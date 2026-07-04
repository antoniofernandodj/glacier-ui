//! Comportamento de componente escrito em **Lua**, interpretado em tempo de
//! execução — sem etapa de compilação.
//!
//! O bloco `<script>` de um template guarda **Lua** (5.4, via [`mlua`]):
//! [`LuaComponent`] o carrega do arquivo e executa as funções quando uma ação
//! chega — nada é compilado, então mudar a lógica não exige recompilar o app.
//!
//! # Acesso ao contexto
//!
//! Cada função Lua enxerga uma tabela global `ctx` espelhando o
//! [`Context`](crate::Context) do motor. Ler `ctx.contador` devolve o valor
//! atual (string); atribuir `ctx.contador = ...` grava de volta. Como Lua
//! coage strings numéricas em aritmética, um contador é só:
//!
//! ```lua
//! function incrementar()
//!     ctx.contador = ctx.contador + 1
//! end
//! ```
//!
//! Depois que a função retorna, toda a tabela `ctx` é copiada de volta ao
//! contexto do motor, então os bindings `{contador}` da markup refletem a
//! mudança na próxima avaliação.
//!
//! Ações de `onChange` (inputs) chegam com o texto digitado: a função recebe
//! esse valor como primeiro argumento **e** na global `value`.
//!
//! ```lua
//! function definir_nome(v)
//!     ctx.nome = v          -- ou: ctx.nome = value
//! end
//! ```

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use crate::component::{Component, Context, FetchResult, PendingFetch, Template};
use mlua::{Function, Lua, MultiValue, Table, Thread, ThreadStatus, Value};

/// Prelúdio Lua injetado antes do `<script>` do usuário. Define `fetch`, que
/// **suspende** a corrotina da ação (via `coroutine.yield`) até o motor concluir
/// a requisição de rede e retomá-la com a resposta — o que dá, no código Lua, a
/// aparência de `async/await`:
///
/// ```lua
/// function carregar()
///     local res = fetch("https://api.exemplo/dados")  -- "await": não bloqueia
///     if res.ok then ctx.dados = res.body end
/// end
/// ```
const PRELUDE: &str = r#"
function fetch(url, opts)
    return coroutine.yield({ __glacier_fetch = true, url = url, opts = opts or {} })
end
"#;

/// Um [`Component`] cujo comportamento vem de um bloco `<script>` em Lua.
///
/// O template (XML ou KDL) é lido do disco; seu `<script>` é extraído e
/// carregado num interpretador Lua próprio. Cada ação (`onClick`, `onChange`,
/// `onSubmit`) roda como uma **corrotina**: chama a função Lua homônima, que
/// lê/escreve o contexto via a tabela global `ctx` e pode chamar `fetch` para
/// rede sem bloquear a UI.
pub struct LuaComponent {
    name: String,
    path: String,
    lua: Lua,
    /// Tabela `ctx` persistente (o mesmo objeto entre chamadas), espelhando o
    /// contexto do motor. Mantida fixa para que corrotinas suspensas que a
    /// referenciam continuem válidas ao serem retomadas.
    ctx_table: Table,
    /// Corrotinas suspensas num `fetch`, aguardando a resposta, por `id`.
    pending: RefCell<HashMap<u64, Thread>>,
    /// Gerador de `id` para requisições de `fetch`.
    next_id: Cell<u64>,
}

impl LuaComponent {
    /// Cria um componente Lua a partir de um arquivo de template.
    ///
    /// O corpo Lua vem de uma de duas fontes:
    /// - **externo**: `<script src="arquivo.lua">` (ou `from="..."`) carrega o
    ///   Lua de outro arquivo, resolvido relativo ao diretório do template;
    /// - **inline**: senão, o corpo do próprio bloco `<script>...</script>`.
    ///
    /// O script é executado uma vez para definir as funções. Erros de I/O ou de
    /// sintaxe Lua viram `Err`.
    pub fn from_file(path: impl Into<String>, name: impl Into<String>) -> Result<Self, String> {
        let path = path.into();
        let name = name.into();
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Falha ao ler template Lua em '{}': {}", path, e))?;
        let script = resolve_script(&content, &path)?;
        Self::from_source(&script, path, name)
    }

    /// Cria um componente Lua a partir do código-fonte já extraído, associando-o
    /// a um `path` de template (para o motor renderizar a UI e manter hot-reload).
    pub fn from_source(
        script: &str,
        path: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<Self, String> {
        let name = name.into();
        let lua = Lua::new();
        lua.load(PRELUDE).set_name("<glacier prelude>").exec().map_err(|e| {
            format!("Erro ao carregar prelúdio Lua: {}", e)
        })?;
        lua.load(script)
            .set_name(format!("<script:{name}>"))
            .exec()
            .map_err(|e| format!("Erro ao carregar <script> Lua de '{}': {}", name, e))?;
        let ctx_table = lua
            .create_table()
            .map_err(|e| format!("Erro ao criar tabela ctx: {}", e))?;
        lua.globals()
            .set("ctx", &ctx_table)
            .map_err(|e| format!("Erro ao registrar ctx: {}", e))?;
        Ok(Self {
            name,
            path: path.into(),
            lua,
            ctx_table,
            pending: RefCell::new(HashMap::new()),
            next_id: Cell::new(1),
        })
    }

    /// Espelha o contexto do motor na tabela Lua `ctx`: limpa a tabela e a
    /// repopula com o estado atual, para que ela reflita o contexto *exatamente*
    /// no início da execução. É o que permite ao `sync_from_lua` detectar o que
    /// o Lua removeu (`ctx.x = nil`). A tabela é limpa in-place (mesmo objeto),
    /// preservando referências de corrotinas suspensas.
    fn sync_to_lua(&self, ctx: &Context) -> mlua::Result<()> {
        self.ctx_table.clear()?;
        for (k, v) in ctx.data.iter() {
            self.ctx_table.set(k.as_str(), v.as_str())?;
        }
        Ok(())
    }

    /// Copia a tabela `ctx` de volta ao contexto do motor, tratando-a como a
    /// fonte da verdade: chaves com valor string-izável são gravadas (novas
    /// incluídas); chaves que o Lua apagou (`ctx.x = nil`) são **removidas** do
    /// contexto — como a tabela começou espelhando o contexto (ver
    /// [`Self::sync_to_lua`]), toda chave do contexto ausente aqui foi
    /// deliberadamente removida pelo script. `nil`/tabelas/funções não são
    /// gravados.
    fn sync_from_lua(&self, ctx: &mut Context) -> mlua::Result<()> {
        let mut present = std::collections::HashSet::new();
        for pair in self.ctx_table.pairs::<String, Value>() {
            let (k, val) = pair?;
            present.insert(k.clone());
            if let Some(s) = lua_value_to_string(&val) {
                ctx.set(&k, s);
            }
        }
        // Chaves que existiam no contexto mas não estão mais na tabela (o Lua as
        // setou para nil) são removidas.
        let removed: Vec<String> =
            ctx.data.keys().filter(|k| !present.contains(*k)).cloned().collect();
        for k in removed {
            ctx.data.remove(&k);
        }
        Ok(())
    }

    /// Roda a função `func` (se existir) como uma corrotina, passando `value`.
    fn run(&self, func: &str, value: Option<&str>, ctx: &mut Context) {
        if let Err(e) = self.run_inner(func, value, ctx) {
            eprintln!("[glacier-ui] erro em <script> Lua '{}::{}': {}", self.name, func, e);
        }
    }

    fn run_inner(&self, func: &str, value: Option<&str>, ctx: &mut Context) -> mlua::Result<()> {
        self.sync_to_lua(ctx)?;
        self.lua.globals().set("value", value)?;

        // Ações sem função correspondente são ignoradas (como o `_ => {}` antigo).
        let Ok(f) = self.lua.globals().get::<Function>(func) else {
            return Ok(());
        };
        let thread = self.lua.create_thread(f)?;
        let args = match value {
            Some(v) => MultiValue::from_iter([Value::String(self.lua.create_string(v)?)]),
            None => MultiValue::new(),
        };
        self.drive(thread, args, ctx)
    }

    /// Retoma a corrotina suspensa `id` com o resultado do `fetch`.
    fn resume_inner(&self, id: u64, result: &FetchResult, ctx: &mut Context) -> mlua::Result<()> {
        let Some(thread) = self.pending.borrow_mut().remove(&id) else {
            return Ok(());
        };
        self.sync_to_lua(ctx)?;
        let res = self.result_to_lua(result)?;
        self.drive(thread, MultiValue::from_iter([Value::Table(res)]), ctx)
    }

    /// Resume a corrotina uma vez com `args`; sincroniza o contexto de volta e,
    /// se ela suspendeu num `fetch`, registra a requisição e guarda a corrotina
    /// para retomada posterior. Se terminou, nada mais a fazer.
    fn drive(&self, thread: Thread, args: MultiValue, ctx: &mut Context) -> mlua::Result<()> {
        let yielded: MultiValue = thread.resume(args)?;
        self.sync_from_lua(ctx)?;

        if thread.status() != ThreadStatus::Resumable {
            return Ok(()); // corrotina terminou
        }

        // Suspendeu: o único yield que o motor entende é um pedido de `fetch`.
        if let Some(Value::Table(req)) = yielded.into_iter().next() {
            if req.get::<bool>("__glacier_fetch").unwrap_or(false) {
                let id = self.next_id.get();
                self.next_id.set(id + 1);
                ctx.fetches.push(self.parse_fetch(id, &req)?);
                self.pending.borrow_mut().insert(id, thread);
            }
        }
        Ok(())
    }

    /// Extrai uma [`PendingFetch`] da tabela `{ url, opts }` que o `fetch` cedeu.
    fn parse_fetch(&self, id: u64, req: &Table) -> mlua::Result<PendingFetch> {
        let url: String = req.get("url")?;
        let opts: Option<Table> = req.get("opts")?;
        let (method, body, headers) = match opts {
            Some(o) => {
                let method = o.get::<Option<String>>("method")?.unwrap_or_else(|| "GET".into());
                let body = o.get::<Option<String>>("body")?;
                let headers = match o.get::<Option<Table>>("headers")? {
                    Some(h) => {
                        let mut v = Vec::new();
                        for pair in h.pairs::<String, String>() {
                            let (k, val) = pair?;
                            v.push((k, val));
                        }
                        v
                    }
                    None => Vec::new(),
                };
                (method, body, headers)
            }
            None => ("GET".into(), None, Vec::new()),
        };
        Ok(PendingFetch::new(id, url, method, body, headers))
    }

    /// Converte um [`FetchResult`] na tabela Lua `{ ok, status, body, error }`.
    fn result_to_lua(&self, r: &FetchResult) -> mlua::Result<Table> {
        let t = self.lua.create_table()?;
        t.set("ok", r.ok)?;
        t.set("status", r.status)?;
        t.set("body", r.body.as_str())?;
        t.set("error", r.error.as_str())?;
        Ok(t)
    }
}

impl Component for LuaComponent {
    fn name(&self) -> &str {
        &self.name
    }

    fn template(&self) -> Template {
        Template::File(self.path.clone())
    }

    /// Chama uma função Lua opcional `init()` para semear o estado inicial.
    fn init(&mut self, ctx: &mut Context) {
        self.run("init", None, ctx);
    }

    fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context) {
        self.run(action, value, ctx);
    }

    fn on_form_submit(&mut self, action: &str, ctx: &mut Context) {
        self.run(action, None, ctx);
    }

    fn resume_fetch(&mut self, id: u64, result: &FetchResult, ctx: &mut Context) {
        if let Err(e) = self.resume_inner(id, result, ctx) {
            eprintln!("[glacier-ui] erro ao retomar fetch em '{}': {}", self.name, e);
        }
    }
}

/// Converte um [`Value`] Lua na string que o contexto do motor guarda. Números
/// inteiros e floats de valor inteiro viram `"3"` (não `"3.0"`); `nil` devolve
/// `None` para não sobrescrever chaves com valor vazio à toa.
fn lua_value_to_string(v: &Value) -> Option<String> {
    match v {
        Value::Nil => None,
        Value::Boolean(b) => Some(b.to_string()),
        Value::Integer(i) => Some(i.to_string()),
        Value::Number(n) => {
            if n.fract() == 0.0 && n.is_finite() {
                Some((*n as i64).to_string())
            } else {
                Some(n.to_string())
            }
        }
        Value::String(s) => Some(s.to_string_lossy()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Roda `func`/`value` contra um mapa de contexto e devolve o mapa mutado,
    /// exercitando o mesmo caminho de `update`.
    fn drive(comp: &LuaComponent, func: &str, value: Option<&str>, mut data: HashMap<String, String>) -> HashMap<String, String> {
        let mut ctx = Context {
            data: &mut data,
            nav: None,
            effects: Vec::new(),
            dialog: None,
            toasts: Vec::new(),
            fetches: Vec::new(),
        };
        comp.run(func, value, &mut ctx);
        data
    }

    #[test]
    fn incrementa_lendo_e_escrevendo_o_contexto() {
        let comp = LuaComponent::from_source(
            "function incrementar() ctx.contador = ctx.contador + 1 end",
            "t.xml",
            "c",
        )
        .unwrap();
        let mut data = HashMap::new();
        data.insert("contador".into(), "0".into());
        let data = drive(&comp, "incrementar", None, data);
        // Coerção de string numérica + volta a inteiro (não "1.0").
        assert_eq!(data.get("contador").map(String::as_str), Some("1"));
    }

    #[test]
    fn onchange_recebe_o_valor() {
        let comp = LuaComponent::from_source(
            "function set_nome(v) ctx.nome = v end",
            "t.xml",
            "c",
        )
        .unwrap();
        let data = drive(&comp, "set_nome", Some("Ana"), HashMap::new());
        assert_eq!(data.get("nome").map(String::as_str), Some("Ana"));
    }

    #[test]
    fn atribuir_nil_remove_a_chave_no_contexto() {
        let comp = LuaComponent::from_source(
            "function limpar() ctx.temp = nil end",
            "t.xml",
            "c",
        )
        .unwrap();
        let mut data = HashMap::new();
        data.insert("temp".into(), "algo".into());
        data.insert("manter".into(), "ok".into());
        let data = drive(&comp, "limpar", None, data);
        assert_eq!(data.get("temp"), None, "ctx.temp = nil deveria remover a chave");
        // Chaves não tocadas pelo script permanecem.
        assert_eq!(data.get("manter").map(String::as_str), Some("ok"));
    }

    #[test]
    fn acao_sem_funcao_e_ignorada() {
        let comp = LuaComponent::from_source("function a() end", "t.xml", "c").unwrap();
        let mut data = HashMap::new();
        data.insert("x".into(), "keep".into());
        let data = drive(&comp, "inexistente", None, data);
        assert_eq!(data.get("x").map(String::as_str), Some("keep"));
    }

    #[test]
    fn init_semea_default() {
        let comp = LuaComponent::from_source(
            "function init() ctx.contador = ctx.contador or 5 end",
            "t.xml",
            "c",
        )
        .unwrap();
        let data = drive(&comp, "init", None, HashMap::new());
        assert_eq!(data.get("contador").map(String::as_str), Some("5"));
    }

    #[test]
    fn fetch_suspende_a_corrotina_e_retoma_com_a_resposta() {
        let comp = LuaComponent::from_source(
            r#"
            function carregar()
                local res = fetch("http://exemplo/api", { method = "POST", body = "q" })
                if res.ok then ctx.dados = res.body else ctx.erro = res.error end
            end
            "#,
            "t.xml",
            "c",
        )
        .unwrap();
        let mut data = HashMap::new();

        // 1) roda a ação: `fetch` cede, a corrotina suspende e um PendingFetch aparece.
        let id;
        {
            let mut ctx = Context {
                data: &mut data,
                nav: None,
                effects: Vec::new(),
                dialog: None,
                toasts: Vec::new(),
                fetches: Vec::new(),
            };
            comp.run("carregar", None, &mut ctx);
            assert_eq!(ctx.fetches.len(), 1, "deveria ter suspendido num fetch");
            assert_eq!(ctx.fetches[0].url, "http://exemplo/api");
            assert_eq!(ctx.fetches[0].method, "POST");
            assert_eq!(ctx.fetches[0].body.as_deref(), Some("q"));
            id = ctx.fetches[0].id;
        }

        // 2) o motor entrega a resposta: a corrotina retoma no ponto do fetch.
        {
            let mut ctx = Context {
                data: &mut data,
                nav: None,
                effects: Vec::new(),
                dialog: None,
                toasts: Vec::new(),
                fetches: Vec::new(),
            };
            let res = FetchResult { ok: true, status: 200, body: "OLA".into(), error: String::new() };
            comp.resume_inner(id, &res, &mut ctx).unwrap();
        }
        assert_eq!(data.get("dados").map(String::as_str), Some("OLA"));
        assert_eq!(data.get("erro"), None);
    }

    #[test]
    fn detecta_src_externo() {
        assert_eq!(
            extract_script_src(r#"<script src="scripts/c.lua"></script>"#).as_deref(),
            Some("scripts/c.lua")
        );
        assert_eq!(
            extract_script_src(r#"<script from='a.lua' />"#).as_deref(),
            Some("a.lua")
        );
        // Sem src: inline, então None.
        assert_eq!(extract_script_src("<script> a </script>"), None);
    }

    #[test]
    fn carrega_lua_de_arquivo_externo_relativo_ao_template() {
        // Monta template + .lua num diretório temporário e confere que o `src`
        // é resolvido relativo ao diretório do template.
        let dir = std::env::temp_dir().join(format!("glacier_lua_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let tpl = dir.join("t.xml");
        let lua = dir.join("beh.lua");
        std::fs::write(&lua, "function incrementar() ctx.n = ctx.n + 1 end").unwrap();
        std::fs::write(&tpl, r#"<Text/><script src="beh.lua"></script>"#).unwrap();

        let comp = LuaComponent::from_file(tpl.to_str().unwrap(), "c").unwrap();
        let mut data = HashMap::new();
        data.insert("n".into(), "41".into());
        let data = drive(&comp, "incrementar", None, data);
        assert_eq!(data.get("n").map(String::as_str), Some("42"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn extrai_script_de_xml_e_kdl() {
        assert_eq!(
            extract_script("<Text/>\n<script> a </script>").as_deref(),
            Some(" a ")
        );
        // KDL: fecha no `}` de nível 0, respeitando chaves aninhadas do Lua.
        assert_eq!(
            extract_script("Text\nscript {\n if x then y() end\n}").as_deref(),
            Some("\n if x then y() end\n")
        );
    }
}

/// Resolve o corpo Lua de um template: se o `<script>` referencia um arquivo
/// externo via `src="..."` (ou `from="..."`), lê esse arquivo (caminho relativo
/// ao diretório do `template_path`); senão, usa o corpo inline do bloco.
fn resolve_script(markup: &str, template_path: &str) -> Result<String, String> {
    if let Some(src) = extract_script_src(markup) {
        let base = std::path::Path::new(template_path)
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        let lua_path = base.join(&src);
        return std::fs::read_to_string(&lua_path).map_err(|e| {
            format!("Falha ao ler script Lua externo '{}': {}", lua_path.display(), e)
        });
    }
    Ok(extract_script(markup).unwrap_or_default())
}

/// Lê o atributo `src`/`from` da tag de abertura `<script ...>`, se houver — o
/// caminho de um arquivo `.lua` externo.
fn extract_script_src(markup: &str) -> Option<String> {
    let lower = markup.to_ascii_lowercase();
    let open = lower.find("<script")?;
    // Só o texto da tag de abertura (até o primeiro `>`).
    let gt = lower[open..].find('>')? + open;
    let tag = &markup[open..gt];
    let re = regex::Regex::new(r#"(?i)\b(?:src|from)\s*=\s*["']([^"']+)["']"#).ok()?;
    re.captures(tag)
        .map(|c| c.get(1).map_or(String::new(), |m| m.as_str().to_string()))
        .filter(|s| !s.is_empty())
}

/// Extrai o corpo de um bloco `<script>...</script>` (XML) ou `script { ... }`
/// (KDL) de um template. Espelha a lógica de remoção do parser, mas devolve o
/// conteúdo em vez de descartá-lo.
fn extract_script(markup: &str) -> Option<String> {
    let lower = markup.to_ascii_lowercase();
    // XML: <script ...> corpo </script>
    if let Some(open) = lower.find("<script") {
        let gt = lower[open..].find('>')? + open + 1;
        let close = lower[gt..].find("</script>")? + gt;
        return Some(markup[gt..close].to_string());
    }
    // KDL: script { corpo }
    if let Some(rel) = lower.find("script") {
        let after = rel + "script".len();
        if let Some(brace_rel) = lower[after..].find('{') {
            let body_start = after + brace_rel + 1;
            // Fecha no `}` de nível 0 (o corpo Lua pode ter chaves aninhadas).
            let mut depth = 1i32;
            for (i, c) in markup[body_start..].char_indices() {
                match c {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            return Some(markup[body_start..body_start + i].to_string());
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    None
}
