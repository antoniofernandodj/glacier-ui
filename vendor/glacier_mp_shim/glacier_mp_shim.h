// API própria (não upstream) sobre o "embed port" do MicroPython, para uso
// via FFI a partir de src/micropython/mod.rs.
//
// Por que não o `port/micropython_embed.h` (mp_embed_init/exec_str/deinit)
// vendorizado ao lado: aquela API não devolve texto de erro (só imprime e
// segue), não dá pra rodar cada componente do glacier-ui num namespace
// próprio (um só `mp_embed_exec_str` usa sempre os globals "correntes"), e
// não tem como chamar uma função já definida pelo nome — as três coisas que
// o bridge de `ctx` e o despacho de ações (ver LuauComponent::run_inner)
// precisam. Este arquivo cobre só isso, deixando o resto do runtime (heap,
// GC, exceções) para o C vendorizado.
//
// UMA VM por processo: o estado do MicroPython (heap, GC, pilha) é global,
// não há como ter duas instâncias independentes coexistindo (ver o
// `mp_state_ctx` upstream). `glacier_mp_init` deve rodar uma única vez; cada
// componente ganha seu próprio dicionário de globals (`glacier_mp_globals_t`)
// para isolar variáveis, mas todos compartilham a mesma VM/heap/GC. E só a
// MESMA THREAD que chamou `glacier_mp_init` pode chamar qualquer função
// daqui depois — o runtime não é thread-safe.
#ifndef GLACIER_MP_SHIM_H
#define GLACIER_MP_SHIM_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// Inicializa a VM (heap + GC + interpretador). Chame uma única vez por
// processo, antes de qualquer outra função deste header — chamadas
// subsequentes são no-op.
void glacier_mp_init(void);

// Um dicionário de globals — o namespace de UM componente. Opaco do lado
// Rust (sempre um `mp_obj_t` por baixo, já com type-punning pra `void*`).
typedef void *glacier_mp_globals_t;

// Cria um dicionário de globals novo e vazio (o namespace de um componente).
// Alocado no heap do MicroPython — vive enquanto o GC não coletar todo
// mundo que aponta pra ele; quem chama deve manter uma referência Rust viva
// (dentro de MicropythonComponent) por toda a vida do componente.
glacier_mp_globals_t glacier_mp_new_globals(void);

// Solta o dicionário de `globals` para o GC coletar — chame quando o
// componente (do lado Rust) for dropado. Sem isso ele vaza: ver o comentário
// "por que dict_main" em glacier_mp_shim.c pra entender por que é PRECISO
// chamar isto (não é só limpeza, é o que impede um `gc.collect()` de outro
// componente derrubar este enquanto ainda vivo, então o desfazer também é
// explícito).
void glacier_mp_free_globals(glacier_mp_globals_t globals);

// Status devolvido por glacier_mp_exec/glacier_mp_call.
#define GLACIER_MP_OK 0
// Uma exceção Python não-capturada; `err_out` tem o texto (repr da exceção).
#define GLACIER_MP_ERROR 1
// Erro nativo sem texto formatado (ex.: um MemoryError bem no início, antes
// do sink de erro conseguir formatar) — `err_out` fica vazio.
#define GLACIER_MP_NATIVE_ERROR 2
// glacier_mp_call: não existe uma global chamável com esse nome.
#define GLACIER_MP_NOT_FOUND 3

// Compila e roda `src` (o corpo do <script>) com `globals` como namespace —
// define as funções/variáveis de topo nele. Erro (exceção Python) devolve
// GLACIER_MP_ERROR e escreve o texto em `err_out` (até `err_cap` bytes,
// sempre terminado em NUL).
int glacier_mp_exec(
    glacier_mp_globals_t globals,
    const char *src,
    char *err_out,
    size_t err_cap
);

// Chama `globals[name](*args)`, com `argc` argumentos (0 a 2) tirados de
// `arg0`/`arg1` — todos strings Python (mesma convenção do bridge Lua: quem
// quiser aritmética faz `int(ctx["x"])` no próprio script). Devolve
// GLACIER_MP_NOT_FOUND se `name` não existe ou não é chamável (quem chama
// decide o fallback, como o `run_inner` do Lua já faz).
int glacier_mp_call(
    glacier_mp_globals_t globals,
    const char *name,
    int argc,
    const char *arg0,
    const char *arg1,
    char *err_out,
    size_t err_cap
);

// Zera (ou cria, na primeira vez) o dict de dados de `globals` como um dict
// vazio — o equivalente a LuauComponent::sync_to_luau limpando a tabela
// antes de repopulá-la. NÃO é `globals["ctx"]`: o `ctx` que o script Python
// vê é um objeto `CtxDot` (definido no prelúdio Rust, não aqui) dando a ele
// `ctx.chave` além de `ctx["chave"]` — as funções deste header leem/escrevem
// o dict de dados por baixo dele (ver o comentário "bridge de ctx" no topo
// de glacier_mp_shim.c pro nome exato do global).
void glacier_mp_ctx_reset(glacier_mp_globals_t globals);

// `<dict de dados de ctx>[key] = value` — `value` sempre uma string Python
// (nunca pré-convertida a int/float; mesma convenção do lado Lua).
void glacier_mp_ctx_set(glacier_mp_globals_t globals, const char *key, const char *value);

// Callback de leitura: chamado uma vez por chave presente em `ctx` depois de
// rodar uma ação, com o valor já formatado como texto — bool vira
// "true"/"false" (minúsculo, pra bater com o resto do motor), int em
// decimal, str como está. Chaves cujo valor não converte pra texto (None,
// incluído — é a forma de "apagar" a chave, espelhando `ctx.x = nil` no
// Lua) simplesmente não são visitadas.
typedef void (*glacier_mp_ctx_visit_fn)(void *user, const char *key, const char *value);

void glacier_mp_ctx_visit(glacier_mp_globals_t globals, glacier_mp_ctx_visit_fn cb, void *user);

#ifdef __cplusplus
}
#endif

#endif // GLACIER_MP_SHIM_H
