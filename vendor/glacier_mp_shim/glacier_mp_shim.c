#include "glacier_mp_shim.h"

#include <stdint.h>
#include <string.h>

#include "py/compile.h"
#include "py/gc.h"
#include "py/obj.h"
#include "py/parse.h"
#include "py/runtime.h"
#include "py/stackctrl.h"
#include "shared/runtime/gchelper.h"

// Heap da GC compartilhado por toda a VM (todos os componentes). 1 MiB é
// generoso pra um app desktop — bem acima do que um script de mudar estado
// precisa — e ainda cabe folgado num binário que já embute iced+wgpu.
// Ver o comentário "UMA VM por processo" em glacier_mp_shim.h.
#define GLACIER_MP_HEAP_SIZE (1024 * 1024)
static char glacier_mp_heap[GLACIER_MP_HEAP_SIZE];

static int glacier_mp_initialized = 0;

void glacier_mp_init(void) {
    if (glacier_mp_initialized) {
        return;
    }
    glacier_mp_initialized = 1;
    int stack_top;
    mp_stack_set_top(&stack_top);
    gc_init(&glacier_mp_heap[0], &glacier_mp_heap[0] + GLACIER_MP_HEAP_SIZE);
    mp_init();
}

// Necessária pelo GC conservador (varre a pilha C e os registradores em
// busca de ponteiros vivos) — mesma implementação que o embed_util.c
// vendorizado, mas só um dos dois pode existir no link final. Como não
// compilamos aquele arquivo (ver build.rs), é esta que vale.
void gc_collect(void) {
    gc_collect_start();
    gc_helper_collect_regs_and_stack();
    gc_collect_end();
}

// Chamada se uma exceção escapar de todo nlr_push — não deveria acontecer
// nunca aqui (glacier_mp_exec/glacier_mp_call sempre envolvem a execução do
// script num nlr_push próprio), mas travar em loop é o comportamento padrão
// do próprio MicroPython embed (main.c upstream faz o mesmo) — melhor que UB.
void nlr_jump_fail(void *val) {
    (void)val;
    for (;;) {
    }
}

#ifndef NDEBUG
void __assert_func(const char *file, int line, const char *func, const char *expr) {
    (void)file;
    (void)line;
    (void)func;
    (void)expr;
    for (;;) {
    }
}
#endif

// stdout do MicroPython (usado por `print()`) — manda pro stdout do
// processo host, igual ao mphalport.c vendorizado (não compilado por termos
// nosso próprio `gc_collect`/`nlr_jump_fail` — ver build.rs).
#include <stdio.h>
void mp_hal_stdout_tx_strn_cooked(const char *str, size_t len) {
    fwrite(str, 1, len, stdout);
}

// Troca globals/locals correntes para os do componente, roda `body`, e
// restaura o que havia antes — mesmo em caso de exceção (a restauração
// acontece nos dois braços do nlr_push de quem chama esta função, então ela
// mesma nunca lança).
static void glacier_mp_enter(mp_obj_dict_t *globals, mp_obj_dict_t **out_prev_globals, mp_obj_dict_t **out_prev_locals) {
    *out_prev_globals = mp_globals_get();
    *out_prev_locals = mp_locals_get();
    mp_globals_set(globals);
    mp_locals_set(globals);
}

static void glacier_mp_leave(mp_obj_dict_t *prev_globals, mp_obj_dict_t *prev_locals) {
    mp_globals_set(prev_globals);
    mp_locals_set(prev_locals);
}

// Por que registrar em `MP_STATE_VM(dict_main)`: o GC do MicroPython escaneia
// `mp_state_ctx` como uma região de memória só (ver o comentário em
// `gc_collect_start`, py/gc.c) — isso cobre `dict_globals`/`dict_locals`
// CORRENTES (o componente rodando agora, sempre seguro enquanto executa) e
// `dict_main` (o módulo "__main__", sempre vivo). Mas NÃO cobre o `globals`
// de um componente OCIOSO: nada aponta pra ele fora do `*mut c_void` que o
// `MicropythonComponent` guarda do lado Rust, e o GC não enxerga a heap do
// Rust. Sem isto, o primeiro `gc.collect()` (explícito, ou automático quando
// a heap compartilhada aperta) que rodar durante a chamada de QUALQUER
// componente colapsaria o dict de todo componente ocioso — a próxima chamada
// nele leria um objeto já reciclado. Guardar cada `globals` como um valor
// dentro de `dict_main` (chaveado pelo próprio endereço, único enquanto
// vivo) mantém todos os componentes alcançáveis o tempo todo — até
// `glacier_mp_free_globals` remover a entrada.
glacier_mp_globals_t glacier_mp_new_globals(void) {
    mp_obj_t dict = mp_obj_new_dict(0);
    mp_obj_t key = mp_obj_new_int_from_uint((mp_uint_t)(uintptr_t)MP_OBJ_TO_PTR(dict));
    mp_obj_dict_store(MP_OBJ_FROM_PTR(&MP_STATE_VM(dict_main)), key, dict);
    return (glacier_mp_globals_t)MP_OBJ_TO_PTR(dict);
}

void glacier_mp_free_globals(glacier_mp_globals_t globals_in) {
    // `mp_obj_dict_delete` levanta KeyError (via nlr_jump) se a chave não
    // estiver lá — não deveria acontecer (só quem chamou `glacier_mp_new_globals`
    // insere essa chave, uma vez), mas sem o nlr_push próprio uma exceção
    // aqui cairia direto em `nlr_jump_fail` (loop infinito) por falta de
    // handler no topo da pilha. Melhor engolir e seguir.
    nlr_buf_t nlr;
    if (nlr_push(&nlr) == 0) {
        mp_obj_t key = mp_obj_new_int_from_uint((mp_uint_t)(uintptr_t)globals_in);
        mp_obj_dict_delete(MP_OBJ_FROM_PTR(&MP_STATE_VM(dict_main)), key);
        nlr_pop();
    }
}

// ---- captura de texto de exceção ----

typedef struct {
    char *buf;
    size_t cap;
    size_t used;
} err_sink_t;

static void err_sink_strn(void *data, const char *str, size_t len) {
    err_sink_t *s = (err_sink_t *)data;
    if (s->cap == 0) {
        return;
    }
    size_t avail = s->cap - 1 - s->used;
    if (avail == 0) {
        return;
    }
    size_t n = len < avail ? len : avail;
    memcpy(s->buf + s->used, str, n);
    s->used += n;
    s->buf[s->used] = '\0';
}

static void format_exception(mp_obj_t exc, char *err_out, size_t err_cap) {
    if (err_cap == 0) {
        return;
    }
    err_out[0] = '\0';
    err_sink_t sink = { .buf = err_out, .cap = err_cap, .used = 0 };
    mp_print_t print = { .data = &sink, .print_strn = err_sink_strn };
    mp_obj_print_exception(&print, exc);
}

int glacier_mp_exec(glacier_mp_globals_t globals_in, const char *src, char *err_out, size_t err_cap) {
    mp_obj_dict_t *globals = (mp_obj_dict_t *)globals_in;
    mp_obj_dict_t *prev_globals, *prev_locals;
    glacier_mp_enter(globals, &prev_globals, &prev_locals);

    nlr_buf_t nlr;
    int status;
    if (nlr_push(&nlr) == 0) {
        mp_lexer_t *lex = mp_lexer_new_from_str_len(MP_QSTR__lt_stdin_gt_, src, strlen(src), 0);
        qstr source_name = lex->source_name;
        mp_parse_tree_t parse_tree = mp_parse(lex, MP_PARSE_FILE_INPUT);
        mp_obj_t module_fun = mp_compile(&parse_tree, source_name, true);
        mp_call_function_0(module_fun);
        nlr_pop();
        status = GLACIER_MP_OK;
    } else {
        format_exception((mp_obj_t)nlr.ret_val, err_out, err_cap);
        status = GLACIER_MP_ERROR;
    }

    glacier_mp_leave(prev_globals, prev_locals);
    return status;
}

int glacier_mp_call(
    glacier_mp_globals_t globals_in,
    const char *name,
    int argc,
    const char *arg0,
    const char *arg1,
    char *err_out,
    size_t err_cap
) {
    mp_obj_dict_t *globals = (mp_obj_dict_t *)globals_in;
    mp_obj_dict_t *prev_globals, *prev_locals;
    glacier_mp_enter(globals, &prev_globals, &prev_locals);

    mp_map_elem_t *elem = mp_map_lookup(
        &globals->map,
        MP_OBJ_NEW_QSTR(qstr_from_str(name)),
        MP_MAP_LOOKUP
    );
    if (elem == NULL || !mp_obj_is_callable(elem->value)) {
        glacier_mp_leave(prev_globals, prev_locals);
        return GLACIER_MP_NOT_FOUND;
    }
    mp_obj_t fun = elem->value;

    nlr_buf_t nlr;
    int status;
    if (nlr_push(&nlr) == 0) {
        mp_obj_t args[2];
        size_t n_args = 0;
        if (argc >= 1) {
            args[n_args++] = mp_obj_new_str(arg0, strlen(arg0));
        }
        if (argc >= 2) {
            args[n_args++] = mp_obj_new_str(arg1, strlen(arg1));
        }
        mp_call_function_n_kw(fun, n_args, 0, args);
        nlr_pop();
        status = GLACIER_MP_OK;
    } else {
        format_exception((mp_obj_t)nlr.ret_val, err_out, err_cap);
        status = GLACIER_MP_ERROR;
    }

    glacier_mp_leave(prev_globals, prev_locals);
    return status;
}

// ---- bridge de ctx ----
//
// O global Python `ctx` que o script vê NÃO é o dict que estas funções
// manipulam: é um objeto `CtxDot` (definido no prelúdio, `src/micropython/
// prelude.py`) que dá a ele a sintaxe `ctx.chave` (dotdict), além de
// `ctx["chave"]` — os dois lendo/escrevendo o MESMO dict por baixo. Esse
// dict de verdade mora num global próprio, `__glacier_ctx__` (nome
// improvável de colidir com algo que o usuário escreva — mesma convenção do
// `__glacier_viewport` do lado Luau); é ele que as funções abaixo tratam
// como "o ctx". `ctx` em si é criado pelo prelúdio, roda ANTES do script do
// usuário (ver `MicropythonComponent::build`) e nunca é tocado por aqui.
#define GLACIER_CTX_DATA_KEY "__glacier_ctx__"

void glacier_mp_ctx_reset(glacier_mp_globals_t globals_in) {
    mp_obj_dict_t *globals = (mp_obj_dict_t *)globals_in;
    mp_obj_t data = mp_obj_new_dict(0);
    mp_obj_dict_store(
        MP_OBJ_FROM_PTR(globals),
        MP_OBJ_NEW_QSTR(qstr_from_str(GLACIER_CTX_DATA_KEY)),
        data
    );
}

// Devolve o dict `__glacier_ctx__` de `globals`, criando-o vazio se por
// algum motivo tiver sumido (não deveria — só o prelúdio e estas funções
// tocam esse nome).
static mp_obj_t glacier_mp_get_ctx_dict(mp_obj_dict_t *globals) {
    mp_map_elem_t *elem = mp_map_lookup(
        &globals->map,
        MP_OBJ_NEW_QSTR(qstr_from_str(GLACIER_CTX_DATA_KEY)),
        MP_MAP_LOOKUP
    );
    if (elem != NULL && mp_obj_is_type(elem->value, &mp_type_dict)) {
        return elem->value;
    }
    mp_obj_t data = mp_obj_new_dict(0);
    mp_obj_dict_store(
        MP_OBJ_FROM_PTR(globals),
        MP_OBJ_NEW_QSTR(qstr_from_str(GLACIER_CTX_DATA_KEY)),
        data
    );
    return data;
}

void glacier_mp_ctx_set(glacier_mp_globals_t globals_in, const char *key, const char *value) {
    mp_obj_dict_t *globals = (mp_obj_dict_t *)globals_in;
    mp_obj_t ctx = glacier_mp_get_ctx_dict(globals);
    mp_obj_dict_store(
        ctx,
        mp_obj_new_str(key, strlen(key)),
        mp_obj_new_str(value, strlen(value))
    );
}

// Formata um valor Python de volta pra texto, na mesma convenção que
// luau_value_to_string usa do lado Lua (src/luau/mod.rs): bool em
// minúsculo, inteiro em decimal, float sem ".0" à toa quando é um valor
// inteiro. Devolve 0 (e não escreve nada) para None e para qualquer tipo
// ainda não suportado (dict/list/tuple — ver o comentário em
// glacier_mp_shim.h sobre "outras funcionalidades" futuras).
static int format_ctx_value(mp_obj_t value, char *out, size_t cap) {
    if (value == mp_const_none) {
        return 0;
    }
    if (mp_obj_is_bool(value)) {
        snprintf(out, cap, "%s", value == mp_const_true ? "true" : "false");
        return 1;
    }
    if (mp_obj_is_int(value)) {
        snprintf(out, cap, "%lld", (long long)mp_obj_get_int(value));
        return 1;
    }
#if MICROPY_PY_BUILTINS_FLOAT
    if (mp_obj_is_float(value)) {
        mp_float_t f = mp_obj_get_float(value);
        if (f == (long long)f) {
            snprintf(out, cap, "%lld", (long long)f);
        } else {
            snprintf(out, cap, "%.17g", (double)f);
        }
        return 1;
    }
#endif
    if (mp_obj_is_str(value)) {
        snprintf(out, cap, "%s", mp_obj_str_get_str(value));
        return 1;
    }
    return 0;
}

void glacier_mp_ctx_visit(glacier_mp_globals_t globals_in, glacier_mp_ctx_visit_fn cb, void *user) {
    mp_obj_dict_t *globals = (mp_obj_dict_t *)globals_in;
    mp_obj_t ctx = glacier_mp_get_ctx_dict(globals);
    mp_map_t *map = mp_obj_dict_get_map(ctx);
    char value_buf[4096];
    for (size_t i = 0; i < map->alloc; i++) {
        if (!mp_map_slot_is_filled(map, i)) {
            continue;
        }
        mp_map_elem_t *elem = &map->table[i];
        if (!mp_obj_is_str(elem->key)) {
            continue;
        }
        if (format_ctx_value(elem->value, value_buf, sizeof(value_buf))) {
            cb(user, mp_obj_str_get_str(elem->key), value_buf);
        }
    }
}
