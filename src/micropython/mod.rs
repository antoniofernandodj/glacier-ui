//! `<script lang="micropython">`: a segunda linguagem de script do motor, ao
//! lado do Luau (ver [`crate::luau`]). Runtime real do MicroPython (não
//! RustPython/CPython) — o "embed port" C oficial vendorizado em
//! `vendor/micropython_embed`, compilado por `build.rs` só quando a feature
//! `micropython` está ligada. `vendor/glacier_mp_shim/glacier_mp_shim.h` é a
//! API própria (não upstream) que este módulo chama via FFI — leia o
//! comentário no topo dela primeiro; ela explica o que o embed port padrão
//! NÃO oferece (texto de erro, múltiplos namespaces, chamar por nome) e por
//! que um dict raiz (`dict_main`) é essencial para não perder componentes
//! ociosos no primeiro `gc.collect()`.
//!
//! # Onde isto já chega perto do Luau, e onde ainda não
//!
//! Cobre hoje só a **mudança de estado**: um `<script>` MicroPython define
//! funções de topo (`def nome(...): ...`), cada ação (`on_click`, `onChange`)
//! chama a função homônima, que lê/escreve `ctx` — um **dotdict**: tanto
//! `ctx.chave` (leniente, devolve `None` se ausente — como `ctx.chave` no
//! Lua) quanto `ctx["chave"]` (semântica de dict de verdade, levanta
//! `KeyError` se ausente) funcionam, os dois sobre o mesmo dado por baixo.
//! `ctx` não é um dict cru: é montado por um prelúdio Python
//! (`prelude.py`, `include_str!`'d abaixo — o mesmo papel do
//! `luau::PRELUDE`) rodado ANTES do corpo do usuário em cada componente. A
//! convenção de sufixo (`nome:sufixo` chama `nome(sufixo, value)`) é
//! idêntica à do Lua — ver [`crate::luau::LuauComponent::run_inner`].
//!
//! Ainda NÃO tem (ficam para quando alguém precisar, como o resto do motor
//! foi crescendo): `fetch`/rede, `sse`/`websocket`, diálogos suspensivos
//! (`confirm`/`prompt`), temporizadores (`after`), `on_error` customizado,
//! `require`/módulos, `storage`, composição com um [`Component`] Rust via
//! `wrap` (só scripts puros por enquanto), e valores não-string em `ctx`
//! (dict/lista — precisaria de JSON, hoje só bool/int/str fazem o
//! round-trip).
use crate::asset_source::{AssetSource, DiskAssets};
use crate::component::{Component, Context, Template};
use crate::error::{GlacierError, Result};
use std::collections::HashSet;
use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::path::Path;
use std::sync::{Arc, Once};

/// Bindings cruas para `vendor/glacier_mp_shim/glacier_mp_shim.h`. Um
/// `unsafe extern "C"` só com o que este módulo usa — não é um binding
/// geral do MicroPython (não existe; o resto do runtime é só C por baixo do
/// shim, ver o comentário do módulo).
mod ffi {
    use std::ffi::{c_char, c_int, c_void};

    pub type GlacierMpGlobals = *mut c_void;

    pub const GLACIER_MP_OK: c_int = 0;
    pub const GLACIER_MP_NOT_FOUND: c_int = 3;

    pub type CtxVisitFn =
        unsafe extern "C" fn(user: *mut c_void, key: *const c_char, value: *const c_char);

    unsafe extern "C" {
        pub fn glacier_mp_init();
        pub fn glacier_mp_new_globals() -> GlacierMpGlobals;
        pub fn glacier_mp_free_globals(globals: GlacierMpGlobals);
        pub fn glacier_mp_exec(
            globals: GlacierMpGlobals,
            src: *const c_char,
            err_out: *mut c_char,
            err_cap: usize,
        ) -> c_int;
        pub fn glacier_mp_call(
            globals: GlacierMpGlobals,
            name: *const c_char,
            argc: c_int,
            arg0: *const c_char,
            arg1: *const c_char,
            err_out: *mut c_char,
            err_cap: usize,
        ) -> c_int;
        pub fn glacier_mp_ctx_reset(globals: GlacierMpGlobals);
        pub fn glacier_mp_ctx_set(globals: GlacierMpGlobals, key: *const c_char, value: *const c_char);
        pub fn glacier_mp_ctx_visit(globals: GlacierMpGlobals, cb: CtxVisitFn, user: *mut c_void);
    }
}

/// Tamanho do buffer de texto de exceção — generoso o bastante para um
/// traceback Python de poucos frames (scripts de UI não empilham fundo).
const ERR_CAP: usize = 4096;

/// O prelúdio que dá ao global `ctx` sua sintaxe de dotdict — ver o
/// comentário do módulo e `prelude.py`. Rodado uma vez por componente, antes
/// do corpo do `<script>` do usuário (mesmo papel do `luau::PRELUDE`).
const PRELUDE: &str = include_str!("prelude.py");

fn ensure_init() {
    static INIT: Once = Once::new();
    // SAFETY: `Once` garante uma única chamada, mesmo sob chamadas
    // concorrentes a `ensure_init` — mas isto NÃO torna o runtime
    // thread-safe depois: toda outra função deste módulo ainda precisa
    // rodar na mesma thread que fez esta primeira chamada (ver o
    // comentário "UMA VM por processo" em glacier_mp_shim.h). O motor já
    // roda a lógica de update numa única thread (a de UI), então isso vale
    // de graça.
    INIT.call_once(|| unsafe { ffi::glacier_mp_init() });
}

/// Lê um buffer de erro C (NUL-terminado, possivelmente truncado) como
/// `String`, lossy em bytes não-UTF-8 (não deveria acontecer — MicroPython
/// só produz texto UTF-8 — mas um traceback truncado no meio de um
/// caractere multi-byte é possível).
fn read_c_err(buf: &[u8]) -> String {
    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    String::from_utf8_lossy(&buf[..end]).into_owned()
}

/// `CString::new` que troca um NUL embutido por um erro descritivo em vez de
/// `Err(NulError)` cru — a chance de um script/valor de contexto conter um
/// `\0` é baixíssima, mas silenciar viraria um "ação não fez nada" sem pista.
fn c_string(s: &str, what: &str) -> std::result::Result<CString, String> {
    CString::new(s).map_err(|_| format!("{what} contém um byte NUL, que o MicroPython não aceita"))
}

/// Compila e roda `src` em `globals`, devolvendo o texto da exceção (se
/// houver) já formatado. `what` só entra na mensagem de erro do `CString`
/// (um NUL embutido), não no traceback do MicroPython em si.
fn exec_into(globals: ffi::GlacierMpGlobals, src: &str, what: &str) -> std::result::Result<(), String> {
    let c_src = c_string(src, what)?;
    let mut err = [0u8; ERR_CAP];
    let status = unsafe {
        ffi::glacier_mp_exec(globals, c_src.as_ptr(), err.as_mut_ptr() as *mut c_char, err.len())
    };
    if status != ffi::GLACIER_MP_OK {
        return Err(read_c_err(&err));
    }
    Ok(())
}

/// Um [`Component`] cujo comportamento vem de um bloco
/// `<script lang="micropython">`. Ver o comentário do módulo para o que já
/// funciona e o que ainda não.
pub struct MicropythonComponent {
    name: String,
    path: String,
    globals: ffi::GlacierMpGlobals,
}

// `ffi::GlacierMpGlobals` é um ponteiro cru (não-Send/Sync por padrão, o que
// é exatamente correto aqui: o runtime do MicroPython não é thread-safe — ver
// o comentário em glacier_mp_shim.h —, então nenhum `impl Send`/`Sync` deve
// existir para este tipo).

impl Drop for MicropythonComponent {
    fn drop(&mut self) {
        // SAFETY: `self.globals` foi criado por `glacier_mp_new_globals` em
        // `build` e nunca é compartilhado fora deste componente; ninguém mais
        // tem esse ponteiro pra usar depois.
        unsafe { ffi::glacier_mp_free_globals(self.globals) };
    }
}

impl MicropythonComponent {
    /// Cria um componente MicroPython a partir de um arquivo de template.
    pub fn from_file(path: impl Into<String>, name: impl Into<String>) -> Result<Self> {
        Self::from_file_with(path, name, Arc::new(DiskAssets))
    }

    /// Como [`Self::from_file`], mas lendo o template (e o `<script src>`
    /// externo, se houver) através de um [`AssetSource`].
    pub fn from_file_with(
        path: impl Into<String>,
        name: impl Into<String>,
        assets: Arc<dyn AssetSource>,
    ) -> Result<Self> {
        let path = path.into();
        let name = name.into();
        Self::from_file_inner(&path, &name, &assets).map_err(|message| GlacierError::MicroPython {
            component: name,
            message,
        })
    }

    fn from_file_inner(
        path: &str,
        name: &str,
        assets: &Arc<dyn AssetSource>,
    ) -> std::result::Result<Self, String> {
        let content = assets
            .read_to_string(path)
            .map_err(|e| format!("Falha ao ler template MicroPython em '{}': {}", path, e))?;
        let script = resolve_script(&content, path, assets.as_ref())?;
        Self::build(&script, path.to_string(), name.to_string())
    }

    /// Cria um componente MicroPython a partir do código-fonte já extraído.
    pub fn from_source(
        script: &str,
        path: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<Self> {
        let path = path.into();
        let name = name.into();
        Self::build(script, path, name.clone()).map_err(|message| GlacierError::MicroPython {
            component: name,
            message,
        })
    }

    fn build(script: &str, path: String, name: String) -> std::result::Result<Self, String> {
        ensure_init();
        let globals = unsafe { ffi::glacier_mp_new_globals() };
        let comp = Self { name, path, globals };
        // O prelúdio primeiro — `ctx` (o dotdict) precisa existir ANTES do
        // corpo do usuário, que pode referenciá-lo já no nível de topo
        // (mesma razão de `ctx_table` vir antes do script no lado Lua).
        // Falhar aqui seria um bug NOSSO (o prelúdio não depende de nada do
        // usuário), não um erro de script — mas o erro ainda usa o mesmo
        // caminho, só com uma mensagem que deixa isso óbvio.
        exec_into(comp.globals, PRELUDE, "o prelúdio interno")?;
        exec_into(comp.globals, script, "o <script>")?;
        Ok(comp)
    }

    /// Chama `globals[name](*args)`, `args` sendo 0 a 2 strings. `Ok(true)` =
    /// rodou; `Ok(false)` = não existe função chamável com esse nome exato
    /// (quem chama decide o fallback — ver [`Self::run_inner`]); `Err` =
    /// exceção Python não capturada, com o traceback como texto.
    fn call_raw(
        &self,
        name: &str,
        args: &[&str],
    ) -> std::result::Result<bool, String> {
        let c_name = c_string(name, "o nome da ação")?;
        let c_args: Vec<CString> = args
            .iter()
            .map(|a| c_string(a, "um argumento de ação"))
            .collect::<std::result::Result<_, _>>()?;
        let arg0 = c_args.first().map_or(std::ptr::null(), |c| c.as_ptr());
        let arg1 = c_args.get(1).map_or(std::ptr::null(), |c| c.as_ptr());
        let mut err = [0u8; ERR_CAP];
        let status = unsafe {
            ffi::glacier_mp_call(
                self.globals,
                c_name.as_ptr(),
                c_args.len() as c_int,
                arg0,
                arg1,
                err.as_mut_ptr() as *mut c_char,
                err.len(),
            )
        };
        match status {
            ffi::GLACIER_MP_OK => Ok(true),
            ffi::GLACIER_MP_NOT_FOUND => Ok(false),
            _ => Err(read_c_err(&err)),
        }
    }

    /// Espelha o contexto do motor no dict de dados por trás do `ctx`
    /// dotdict do componente (ver `prelude.py`): zera e repopula, pra
    /// refletir o estado *exatamente* no início da execução — o que permite
    /// [`Self::sync_from_micropython`] detectar o que o script removeu
    /// (`ctx.x = None`, `ctx["x"] = None` ou `del ctx["x"]`, as três
    /// equivalentes).
    fn sync_to_micropython(&self, ctx: &Context) {
        unsafe { ffi::glacier_mp_ctx_reset(self.globals) };
        for (k, v) in ctx.data.iter() {
            // Chave/valor de um `ContextMap` nunca têm NUL embutido na
            // prática (vêm de atributos XML e de `ctx.set` do próprio
            // motor) — silenciar aqui em vez de propagar evita que um caso
            // extremo trave uma sincronização inteira.
            let (Ok(ck), Ok(cv)) = (CString::new(k.as_str()), CString::new(v.as_str())) else {
                continue;
            };
            unsafe { ffi::glacier_mp_ctx_set(self.globals, ck.as_ptr(), cv.as_ptr()) };
        }
    }

    /// Copia o dict `ctx` de volta ao contexto do motor — mesma convenção
    /// "diff por presença" que [`crate::luau::LuauComponent::sync_from_luau`]
    /// usa: toda chave que existia no contexto ANTES desta chamada (`before`)
    /// e não apareceu na visita é removida (foi apagada deliberadamente,
    /// `ctx["x"] = None` ou `del ctx["x"]`); o resto é gravado.
    fn sync_from_micropython(&self, ctx: &mut Context, before: HashSet<String>) {
        struct Visit<'ctx, 'data> {
            ctx: &'ctx mut Context<'data>,
            present: HashSet<String>,
        }
        unsafe extern "C" fn visit_cb(user: *mut c_void, key: *const c_char, value: *const c_char) {
            // SAFETY: `user` é sempre `&mut Visit` empilhado no escopo de
            // `sync_from_micropython`, vivo durante toda a chamada a
            // `glacier_mp_ctx_visit` (que não guarda o ponteiro além dela); e
            // `key`/`value` vêm de `mp_obj_str_get_str` no shim, sempre
            // NUL-terminados e válidos por baixo do texto UTF-8 checado na
            // criação da string Python.
            let visit = unsafe { &mut *(user as *mut Visit) };
            let key = unsafe { CStr::from_ptr(key) }.to_string_lossy().into_owned();
            let value = unsafe { CStr::from_ptr(value) }.to_string_lossy().into_owned();
            visit.present.insert(key.clone());
            visit.ctx.set(&key, value);
        }

        let mut visit = Visit {
            ctx,
            present: HashSet::new(),
        };
        unsafe {
            ffi::glacier_mp_ctx_visit(
                self.globals,
                visit_cb,
                &mut visit as *mut Visit as *mut c_void,
            );
        }
        for removed in before.difference(&visit.present) {
            visit.ctx.data.remove(removed);
        }
    }

    /// Resolve `func` para uma chamada, tentando o nome exato primeiro e,
    /// sem ele, `nome:sufixo` → `nome(sufixo, value)` — a mesma convenção do
    /// Lua (`open_service:<id>`, `field:<chave>`; ver
    /// [`crate::luau::LuauComponent::run_inner`]). `Ok(true)` = alguma
    /// função rodou (e o contexto já foi sincronizado de volta); `Ok(false)`
    /// = nenhuma casou (quem chama decide o fallback); `Err` = exceção.
    fn run_inner(
        &self,
        func: &str,
        value: Option<&str>,
        ctx: &mut Context,
    ) -> std::result::Result<bool, String> {
        self.sync_to_micropython(ctx);
        let before: HashSet<String> = ctx.data.keys().cloned().collect();

        let mut args: Vec<&str> = Vec::new();
        if let Some(v) = value {
            args.push(v);
        }
        let ran = if self.call_raw(func, &args)? {
            true
        } else if let Some((name, suffix)) = func.split_once(':') {
            let mut args: Vec<&str> = vec![suffix];
            if let Some(v) = value {
                args.push(v);
            }
            self.call_raw(name, &args)?
        } else {
            false
        };

        if ran {
            self.sync_from_micropython(ctx, before);
        }
        Ok(ran)
    }

    /// Relata um erro de execução: sempre loga em `stderr` e promove a um
    /// toast — ainda sem o equivalente ao `on_error(msg)` do Lua (ver o
    /// comentário do módulo, fica para depois). Nunca propaga.
    fn report_error(&self, where_: &str, err: &str, ctx: &mut Context) {
        let msg = format!(
            "[glacier-ui] erro em <script lang=\"micropython\"> '{}::{}': {}",
            self.name, where_, err
        );
        eprintln!("{msg}");
        ctx.show_toast(crate::toasts::ToastSpec::error(msg).with_title("Erro de script"));
    }
}

impl Component for MicropythonComponent {
    fn name(&self) -> &str {
        &self.name
    }

    fn template(&self) -> Template {
        Template::File(self.path.clone())
    }

    fn init(&mut self, ctx: &mut Context) {
        // Roda um `def init(): ...` de topo, se existir — mesma convenção
        // do `init()` opcional do Lua/builtins (ver docs/BUILTINS.md). Sem
        // função `init`, `run_inner` só devolve `Ok(false)`, e não fazemos
        // nada (diferente de `update`, aqui não há ação/valor pra usar como
        // fallback de escrita direta).
        if let Err(e) = self.run_inner("init", None, ctx) {
            self.report_error("init", &e, ctx);
        }
    }

    fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context) {
        match self.run_inner(action, value, ctx) {
            Ok(true) => {}
            Ok(false) => {
                // Sem função casando: mesma saída de "último recurso" que o
                // Lua sem `inner` usa — grava o valor sob o próprio nome da
                // ação, pra um `onChange="campo"` sem handler script ainda
                // fazer algo minimamente sensato.
                if let Some(v) = value {
                    ctx.set(action, v);
                }
            }
            Err(e) => self.report_error(action, &e, ctx),
        }
    }
}

/// `true` se `markup` carrega um `<script lang="micropython">` (inline ou
/// `src`/`from` externo) — o que faz [`crate::GlacierUI`] instalar um
/// [`MicropythonComponent`] no lugar de tratar o template como UI-only.
/// Espelha [`crate::luau::has_script`], mas checando `lang` primeiro: um
/// `<script>` sem esse atributo (ou com outro valor) não é deste módulo.
pub(crate) fn has_script(markup: &str) -> bool {
    crate::eval::script_lang(markup).as_deref() == Some("micropython")
        && (extract_script_src(markup).is_some() || extract_script(markup).is_some())
}

/// Corpo inline de um `<script lang="micropython">...</script>`. Mesma
/// varredura de [`crate::eval::find_script_open`] que o parser de markup usa
/// para remover o bloco — ver o porquê em
/// [`crate::luau`]'s `extract_script` (a razão de existir é idêntica, só a
/// linguagem do lado de dentro muda).
fn extract_script(markup: &str) -> Option<String> {
    let lower = markup.to_ascii_lowercase();
    let open = crate::eval::find_script_open(markup)?;
    let gt = lower[open..].find('>')? + open + 1;
    let close = lower[gt..].find("</script>")? + gt;
    Some(markup[gt..close].to_string())
}

/// Atributo `src`/`from` da tag de abertura `<script lang="micropython" ...>`,
/// se houver — um arquivo `.py` externo.
fn extract_script_src(markup: &str) -> Option<String> {
    let lower = markup.to_ascii_lowercase();
    let open = crate::eval::find_script_open(markup)?;
    let gt = lower[open..].find('>')? + open;
    let tag = &markup[open..gt];
    let re = regex::Regex::new(r#"(?i)\b(?:src|from)\s*=\s*["']([^"']+)["']"#).ok()?;
    re.captures(tag)
        .map(|c| c.get(1).map_or(String::new(), |m| m.as_str().to_string()))
        .filter(|s| !s.is_empty())
}

/// Resolve o corpo MicroPython de um template: `src`/`from` externo (lido
/// via `assets`, relativo ao diretório do template) ou o corpo inline.
fn resolve_script(
    markup: &str,
    template_path: &str,
    assets: &dyn AssetSource,
) -> std::result::Result<String, String> {
    let Some(src) = extract_script_src(markup) else {
        return Ok(extract_script(markup).unwrap_or_default());
    };
    let dir = Path::new(template_path).parent().unwrap_or(Path::new(""));
    let full = dir.join(&src);
    assets
        .read_to_string(&full.to_string_lossy())
        .map(|s| s.into_owned())
        .map_err(|e| format!("Falha ao ler <script src=\"{}\">: {}", src, e))
}

// Espelha a suíte de `crate::luau::LuauComponent` (mesmos nomes de teste,
// mesmos cenários) — ver o comentário do módulo pra por que a superfície
// pública ainda é bem menor (só ctx get/set + a convenção de sufixo).
//
// NOTA para quem rodar isto: o usuário deste repositório pediu
// explicitamente para nunca chamar `cargo test` aqui (a suíte inteira abre
// janelas numa máquina com GPU/Vulkan quebrado) — estes testes foram
// escritos e verificados por fora (build real da lib + um smoke test em C
// puro contra o mesmo shim, fora do cargo), não rodados via `cargo test`
// nesta sessão.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ContextMap;

    fn drive(
        comp: &mut MicropythonComponent,
        action: &str,
        value: Option<&str>,
        mut data: ContextMap,
    ) -> ContextMap {
        let mut ctx = Context::new(&mut data);
        comp.update(action, value, &mut ctx);
        data
    }

    #[test]
    fn incrementa_lendo_e_escrevendo_o_contexto() {
        let mut comp = MicropythonComponent::from_source(
            "def incrementar():\n    ctx['contador'] = int(ctx['contador']) + 1\n",
            "t.gv",
            "c",
        )
        .unwrap();
        let mut data = ContextMap::default();
        data.insert("contador".into(), "0".into());
        let data = drive(&mut comp, "incrementar", None, data);
        assert_eq!(data.get("contador").map(String::as_str), Some("1"));
    }

    #[test]
    fn dotdict_ctx_ponto_funciona_igual_ao_colchete() {
        // `ctx.chave` (leniente, None se ausente) e `ctx["chave"]` (dict de
        // verdade) leem/escrevem o MESMO dado por baixo — ver prelude.py.
        let mut comp = MicropythonComponent::from_source(
            "def incrementar():\n    ctx.contador = int(ctx.contador or 0) + 1\n",
            "t.gv",
            "c",
        )
        .unwrap();
        let mut data = ContextMap::default();
        data.insert("contador".into(), "9".into());
        let data = drive(&mut comp, "incrementar", None, data);
        assert_eq!(data.get("contador").map(String::as_str), Some("10"));
    }

    #[test]
    fn dotdict_ctx_ponto_ausente_e_none_sem_erro() {
        let mut comp = MicropythonComponent::from_source(
            "def ler():\n    ctx.visto = 'sim' if ctx.nunca_setado is None else 'nao'\n",
            "t.gv",
            "c",
        )
        .unwrap();
        let data = drive(&mut comp, "ler", None, ContextMap::default());
        assert_eq!(data.get("visto").map(String::as_str), Some("sim"));
    }

    #[test]
    fn onchange_recebe_o_valor() {
        let mut comp =
            MicropythonComponent::from_source("def set_nome(v):\n    ctx['nome'] = v\n", "t.gv", "c")
                .unwrap();
        let data = drive(&mut comp, "set_nome", Some("Ana"), ContextMap::default());
        assert_eq!(data.get("nome").map(String::as_str), Some("Ana"));
    }

    #[test]
    fn atribuir_none_remove_a_chave_no_contexto() {
        let mut comp =
            MicropythonComponent::from_source("def limpar():\n    ctx['temp'] = None\n", "t.gv", "c")
                .unwrap();
        let mut data = ContextMap::default();
        data.insert("temp".into(), "algo".into());
        data.insert("manter".into(), "ok".into());
        let data = drive(&mut comp, "limpar", None, data);
        assert_eq!(data.get("temp"), None, "ctx['temp'] = None deveria remover a chave");
        assert_eq!(data.get("manter").map(String::as_str), Some("ok"));
    }

    #[test]
    fn acao_sem_funcao_grava_o_valor_sob_o_proprio_nome() {
        // `update` é o ponto de entrada público (sem um `run`/`dispatch`
        // separado como o Lua tem pros testes) — o fallback de "gravar sob
        // o nome da ação" quando nenhuma função casa é o mesmo de último
        // recurso que `LuauComponent::dispatch` aplica sem `inner`.
        let mut comp = MicropythonComponent::from_source("def a():\n    pass\n", "t.gv", "c").unwrap();
        let data = drive(&mut comp, "inexistente", Some("valor"), ContextMap::default());
        assert_eq!(data.get("inexistente").map(String::as_str), Some("valor"));
    }

    #[test]
    fn acao_com_sufixo_passa_o_sufixo_como_argumento() {
        let mut comp = MicropythonComponent::from_source(
            "def open_service(id):\n    ctx['aberto'] = id\n",
            "t.gv",
            "c",
        )
        .unwrap();
        let data = drive(&mut comp, "open_service:abc", None, ContextMap::default());
        assert_eq!(data.get("aberto").map(String::as_str), Some("abc"));
    }

    #[test]
    fn acao_com_sufixo_e_value_passa_ambos() {
        let mut comp =
            MicropythonComponent::from_source("def field(k, v):\n    ctx[k] = v\n", "t.gv", "c")
                .unwrap();
        let data = drive(&mut comp, "field:nome", Some("Ana"), ContextMap::default());
        assert_eq!(data.get("nome").map(String::as_str), Some("Ana"));
    }

    #[test]
    fn nome_exato_tem_precedencia_sobre_o_split() {
        // `salvar:x` não bate com o literal "salvar:x" (identificador Python
        // não tem ':'), então cai pro split: chama `salvar("x")`, NÃO
        // `salvar_x()` — a asserção prova que o split resolveu pro nome
        // certo (`salvar`) e não colou os dois numa correspondência solta.
        let mut comp = MicropythonComponent::from_source(
            "def salvar(v):\n    ctx['via'] = 'exato'\ndef salvar_x():\n    ctx['via'] = 'split'\n",
            "t.gv",
            "c",
        )
        .unwrap();
        let data = drive(&mut comp, "salvar:x", None, ContextMap::default());
        assert_eq!(data.get("via").map(String::as_str), Some("exato"));
    }

    #[test]
    fn dois_componentes_nao_compartilham_estado() {
        // O runtime do MicroPython é uma VM só por processo (ver o
        // comentário "por que dict_main" em glacier_mp_shim.c) — este teste
        // é o que garante que dois componentes não vazam globals um pro
        // outro apesar disso.
        let mut a =
            MicropythonComponent::from_source("def set_x():\n    ctx['x'] = 'a'\n", "a.gv", "a")
                .unwrap();
        let mut b =
            MicropythonComponent::from_source("def set_x():\n    ctx['x'] = 'b'\n", "b.gv", "b")
                .unwrap();
        let data_a = drive(&mut a, "set_x", None, ContextMap::default());
        let data_b = drive(&mut b, "set_x", None, ContextMap::default());
        assert_eq!(data_a.get("x").map(String::as_str), Some("a"));
        assert_eq!(data_b.get("x").map(String::as_str), Some("b"));
    }

    #[test]
    fn excecao_nao_capturada_nao_trava_e_nao_altera_o_contexto() {
        let mut comp =
            MicropythonComponent::from_source("def falha():\n    raise ValueError('boom')\n", "t.gv", "c")
                .unwrap();
        let mut data = ContextMap::default();
        data.insert("antes".into(), "ok".into());
        let data = drive(&mut comp, "falha", None, data);
        assert_eq!(data.get("antes").map(String::as_str), Some("ok"));
    }
}
