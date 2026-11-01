//! Internacionalização: catálogo por locale, plural e formato de número/data.
//!
//! No markup, uma chamada dentro de `{…}`:
//!
//! ```text
//! text{t("titulo")}
//! text{t("itens", n=contagem)}        <!-- plural: o catálogo escolhe a forma -->
//! text{t("ola", nome=usuario)}         <!-- {nome} dentro da mensagem -->
//! text{num(preco, 2)}                  <!-- 1.234,56 em pt-BR, 1,234.56 em en -->
//! text{date(quando)}                   <!-- 31/10/2026 em pt-BR, 10/31/2026 em en -->
//! ```
//!
//! No `.gv` é igual, dentro de uma string (`text("{t('titulo')}")`).
//!
//! **A locale ativa é uma chave do contexto, `__locale`.** É isso que faz a troca
//! ao vivo funcionar sem reiniciar: toda interpolação que chama `t(…)` lê essa
//! chave pelo [`crate::eval::EvalCtx`], então entra no conjunto de dependências
//! da subárvore e ela é reavaliada quando a chave muda. O catálogo em si é global
//! ao processo (os dados de tradução não mudam a cada quadro).
//!
//! # Catálogo
//!
//! Um JSON por locale: `{ "chave": "texto", "outra": { "one": "{n} item", "other": "{n} itens" } }`.
//! Uma chave pode ser aninhada (`{"menu": {"abrir": "Abrir"}}` é `menu.abrir`) — o objeto
//! é plural só se todas as suas chaves forem categorias (`zero`, `one`, `two`, `few`,
//! `many`, `other`).
//!
//! # Resolução
//!
//! `pt-BR` → `pt` → a locale padrão ([`set_default`]) → a própria chave, com um aviso
//! (uma chave ausente aparece no texto e no log, nunca some em silêncio).
//!
//! # RTL e bidi
//!
//! O `cosmic-text` do `iced` faz o embaralhamento bidirecional **dentro** de uma linha
//! (o texto em árabe ou hebraico sai na ordem visual certa). Este módulo **não** espelha
//! o layout: `row`, alinhamento `start`/`end` e a posição de ícones continuam
//! esquerda→direita. Ver `docs/I18N.md`.

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

/// A chave de contexto que guarda a locale ativa.
pub const CHAVE_LOCALE: &str = "__locale";

#[derive(Debug, Clone)]
enum Msg {
    Texto(String),
    /// categoria (`one`, `other`…) → texto.
    Plural(HashMap<String, String>),
}

#[derive(Default)]
struct Catalogo {
    /// locale (minúscula, `pt-br`) → chave plana → mensagem.
    locales: HashMap<String, HashMap<String, Msg>>,
    padrao: Option<String>,
}

fn catalogo() -> &'static RwLock<Catalogo> {
    static C: OnceLock<RwLock<Catalogo>> = OnceLock::new();
    C.get_or_init(|| RwLock::new(Catalogo::default()))
}

fn norm(locale: &str) -> String {
    locale.trim().replace('_', "-").to_ascii_lowercase()
}

/// O idioma de `pt-br` → `pt`.
fn idioma(locale: &str) -> &str {
    locale.split('-').next().unwrap_or(locale)
}

const CATEGORIAS: &[&str] = &["zero", "one", "two", "few", "many", "other"];

fn achata(prefixo: &str, v: &serde_json::Value, out: &mut HashMap<String, Msg>) {
    use serde_json::Value;
    match v {
        Value::String(s) => {
            out.insert(prefixo.to_string(), Msg::Texto(s.clone()));
        }
        Value::Object(m) => {
            let plural = !m.is_empty()
                && m.keys().all(|k| CATEGORIAS.contains(&k.as_str()))
                && m.values().all(Value::is_string);
            if plural {
                let formas = m
                    .iter()
                    .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string())))
                    .collect();
                out.insert(prefixo.to_string(), Msg::Plural(formas));
            } else {
                for (k, v) in m {
                    let chave = if prefixo.is_empty() { k.clone() } else { format!("{prefixo}.{k}") };
                    achata(&chave, v, out);
                }
            }
        }
        // número, booleano, nulo, lista: não é mensagem.
        _ => {}
    }
}

/// Carrega (ou acrescenta a) uma locale a partir do texto JSON. Chaves repetidas
/// de uma carga anterior são substituídas. O primeiro catálogo carregado vira a
/// locale padrão, se nenhuma foi escolhida com [`set_default`].
pub fn load_json(locale: &str, json: &str) -> Result<usize, String> {
    let v: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("i18n `{locale}`: JSON inválido: {e}"))?;
    if !v.is_object() {
        return Err(format!("i18n `{locale}`: a raiz do catálogo tem de ser um objeto"));
    }
    let mut plano = HashMap::new();
    achata("", &v, &mut plano);
    let n = plano.len();
    let mut c = catalogo().write().unwrap_or_else(|e| e.into_inner());
    let l = norm(locale);
    if c.padrao.is_none() {
        c.padrao = Some(l.clone());
    }
    c.locales.entry(l).or_default().extend(plano);
    Ok(n)
}

/// Carrega todo `*.json` de uma pasta: o nome do arquivo (`pt-BR.json`) é a locale.
/// Devolve as locales carregadas. Só no nativo (a web embute com [`load_json`]).
#[cfg(not(target_arch = "wasm32"))]
pub fn load_dir(dir: &std::path::Path) -> Result<Vec<String>, String> {
    let mut achadas = Vec::new();
    let mut arquivos: Vec<_> = std::fs::read_dir(dir)
        .map_err(|e| format!("i18n: não li `{}`: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    arquivos.sort();
    for p in arquivos {
        let Some(nome) = p.file_stem().and_then(|s| s.to_str()) else { continue };
        let txt = std::fs::read_to_string(&p).map_err(|e| format!("i18n: `{}`: {e}", p.display()))?;
        load_json(nome, &txt)?;
        achadas.push(nome.to_string());
    }
    Ok(achadas)
}

/// Escolhe a locale usada quando o contexto não tem `__locale`, e quando uma
/// chave falta na locale pedida.
pub fn set_default(locale: &str) {
    catalogo().write().unwrap_or_else(|e| e.into_inner()).padrao = Some(norm(locale));
}

/// A locale **ativa do processo**: o que uma janela sem `__locale` próprio (uma janela nova,
/// ainda não semeada) usa. É escrita por [`set_active`] — o que o `i18n.set_locale` do Luau
/// faz — e é o que o daemon espalha para todas as janelas quando o idioma muda.
fn ativa() -> &'static RwLock<Option<String>> {
    static A: OnceLock<RwLock<Option<String>>> = OnceLock::new();
    A.get_or_init(|| RwLock::new(None))
}

/// Define a locale ativa do processo (a melhor carregada para `locale`). Devolve a escolhida,
/// ou `None` (sem mudar nada) se nenhum catálogo serve.
pub fn set_active(locale: &str) -> Option<String> {
    let l = best_match(locale)?;
    *ativa().write().unwrap_or_else(|e| e.into_inner()) = Some(l.clone());
    Some(l)
}

/// A locale ativa do processo, se alguma foi escolhida.
pub fn active() -> Option<String> {
    ativa().read().unwrap_or_else(|e| e.into_inner()).clone()
}

/// A locale padrão, se algum catálogo foi carregado.
pub fn default_locale() -> Option<String> {
    catalogo().read().unwrap_or_else(|e| e.into_inner()).padrao.clone()
}

/// As locales carregadas, em ordem alfabética.
pub fn available() -> Vec<String> {
    let c = catalogo().read().unwrap_or_else(|e| e.into_inner());
    let mut v: Vec<String> = c.locales.keys().cloned().collect();
    v.sort();
    v
}

/// A locale do sistema (`LC_ALL`, `LC_MESSAGES`, `LANG`), sem codificação: `pt_BR.UTF-8` → `pt-br`.
pub fn system_locale() -> Option<String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        for k in ["LC_ALL", "LC_MESSAGES", "LANG"] {
            if let Ok(v) = std::env::var(k) {
                let v = v.split(['.', '@']).next().unwrap_or("").trim().to_string();
                if !v.is_empty() && v != "C" && v != "POSIX" {
                    return Some(norm(&v));
                }
            }
        }
    }
    None
}

/// A melhor locale **carregada** para o desejo `pedida` (`pt-BR` → `pt-br` → `pt`).
pub fn best_match(pedida: &str) -> Option<String> {
    let c = catalogo().read().unwrap_or_else(|e| e.into_inner());
    let l = norm(pedida);
    if c.locales.contains_key(&l) {
        return Some(l);
    }
    let i = idioma(&l);
    if c.locales.contains_key(i) {
        return Some(i.to_string());
    }
    // `pt` pedido, só `pt-br` carregado.
    let mut candidatos: Vec<&String> =
        c.locales.keys().filter(|k| idioma(k) == i).collect();
    candidatos.sort();
    candidatos.first().map(|s| (*s).clone())
}

fn acha<'a>(c: &'a Catalogo, locale: &str, chave: &str) -> Option<&'a Msg> {
    let l = norm(locale);
    for tentativa in [l.as_str(), idioma(&l)] {
        if let Some(m) = c.locales.get(tentativa).and_then(|m| m.get(chave)) {
            return Some(m);
        }
    }
    // `pt` pedido e só `pt-br` carregado.
    let i = idioma(&l);
    let mut ks: Vec<&String> = c.locales.keys().filter(|k| idioma(k) == i).collect();
    ks.sort();
    for k in ks {
        if let Some(m) = c.locales[k].get(chave) {
            return Some(m);
        }
    }
    let padrao = c.padrao.as_deref()?;
    c.locales.get(padrao).and_then(|m| m.get(chave))
}

// ───────────────────────────── plural ─────────────────────────────

/// A categoria de plural (CLDR, simplificada) de `n` no idioma `lang`.
/// Cobre os idiomas mais usados; os demais caem na regra do inglês.
pub fn categoria_plural(lang: &str, n: f64) -> &'static str {
    let inteiro = n.fract() == 0.0;
    let i = n.abs().trunc();
    match lang {
        // sem plural
        "ja" | "zh" | "ko" | "vi" | "th" | "id" | "ms" => "other",
        // 0 e 1 são singular (e, em francês/português, o intervalo [0, 2))
        "fr" | "pt" => {
            if (0.0..2.0).contains(&n.abs()) && (i == 0.0 || i == 1.0) { "one" } else { "other" }
        }
        "ru" | "uk" | "be" => {
            if !inteiro {
                return "other";
            }
            let (m10, m100) = ((i % 10.0) as u32, (i % 100.0) as u32);
            if m10 == 1 && m100 != 11 {
                "one"
            } else if (2..=4).contains(&m10) && !(12..=14).contains(&m100) {
                "few"
            } else {
                "many"
            }
        }
        "pl" => {
            if !inteiro {
                return "other";
            }
            let (m10, m100) = ((i % 10.0) as u32, (i % 100.0) as u32);
            if i == 1.0 {
                "one"
            } else if (2..=4).contains(&m10) && !(12..=14).contains(&m100) {
                "few"
            } else {
                "many"
            }
        }
        "ar" => {
            if !inteiro {
                return "other";
            }
            let m100 = (i % 100.0) as u32;
            match i as u64 {
                0 => "zero",
                1 => "one",
                2 => "two",
                _ if (3..=10).contains(&m100) => "few",
                _ if (11..=99).contains(&m100) => "many",
                _ => "other",
            }
        }
        // en, de, es, it, nl, sv, …: um é singular.
        _ => {
            if inteiro && i == 1.0 { "one" } else { "other" }
        }
    }
}

// ───────────────────────────── número e data ─────────────────────────────

/// (separador de milhar, separador decimal) por idioma.
fn separadores(lang: &str) -> (&'static str, &'static str) {
    match lang {
        "pt" | "es" | "it" | "de" | "nl" | "id" | "tr" | "da" | "el" => (".", ","),
        "fr" => ("\u{202F}", ","),
        "ru" | "uk" | "pl" | "cs" | "sv" | "fi" | "nb" | "no" | "bg" => ("\u{A0}", ","),
        _ => (",", "."),
    }
}

/// Formata `valor` com `casas` decimais e os separadores da locale.
pub fn formata_numero(locale: &str, valor: f64, casas: usize) -> String {
    let l = norm(locale);
    let (mil, dec) = separadores(idioma(&l));
    let bruto = format!("{:.*}", casas, valor.abs());
    let (int_part, frac) = bruto.split_once('.').unwrap_or((&bruto, ""));
    let mut agrupado = String::new();
    let n = int_part.len();
    for (i, ch) in int_part.chars().enumerate() {
        if i > 0 && (n - i) % 3 == 0 {
            agrupado.push_str(mil);
        }
        agrupado.push(ch);
    }
    let sinal = if valor < 0.0 && bruto.chars().any(|c| c != '0' && c != '.') { "-" } else { "" };
    if frac.is_empty() {
        format!("{sinal}{agrupado}")
    } else {
        format!("{sinal}{agrupado}{dec}{frac}")
    }
}

/// (ano, mês, dia) civil a partir de dias desde 1970-01-01 (algoritmo de Hinnant).
fn civil(dias: i64) -> (i64, u32, u32) {
    let z = dias + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Lê uma data: segundos Unix (`1793000000`) ou `AAAA-MM-DD` (com hora opcional, ignorada).
fn le_data(s: &str) -> Option<(i64, u32, u32)> {
    let s = s.trim();
    if let Ok(secs) = s.parse::<i64>() {
        return Some(civil(secs.div_euclid(86_400)));
    }
    if let Ok(secs) = s.parse::<f64>() {
        return Some(civil((secs as i64).div_euclid(86_400)));
    }
    let dia = s.split(['T', ' ']).next()?;
    let mut p = dia.split('-');
    let (a, m, d) = (p.next()?.parse().ok()?, p.next()?.parse().ok()?, p.next()?.parse().ok()?);
    ((1..=12).contains(&m) && (1..=31).contains(&d)).then_some((a, m, d))
}

/// Formata uma data (segundos Unix ou `AAAA-MM-DD`, em UTC) no padrão da locale.
pub fn formata_data(locale: &str, valor: &str) -> Option<String> {
    let (a, m, d) = le_data(valor)?;
    let l = norm(locale);
    Some(match idioma(&l) {
        "en" if l == "en-gb" || l == "en-au" || l == "en-nz" || l == "en-in" => format!("{d:02}/{m:02}/{a:04}"),
        "en" => format!("{m:02}/{d:02}/{a:04}"),
        "pt" | "es" | "fr" | "it" | "el" | "vi" | "id" => format!("{d:02}/{m:02}/{a:04}"),
        "de" | "ru" | "pl" | "cs" | "uk" | "tr" | "fi" | "nb" | "no" | "da" => format!("{d:02}.{m:02}.{a:04}"),
        "ja" | "zh" => format!("{a:04}/{m:02}/{d:02}"),
        "ko" | "sv" | "lt" => format!("{a:04}-{m:02}-{d:02}"),
        "nl" => format!("{d:02}-{m:02}-{a:04}"),
        _ => format!("{a:04}-{m:02}-{d:02}"),
    })
}

// ───────────────────────────── tradução ─────────────────────────────

fn subst(msg: &str, args: &HashMap<String, String>) -> String {
    let mut out = String::with_capacity(msg.len());
    let mut rest = msg;
    while let Some(i) = rest.find('{') {
        out.push_str(&rest[..i]);
        let Some(j) = rest[i..].find('}') else {
            out.push_str(&rest[i..]);
            return out;
        };
        let nome = rest[i + 1..i + j].trim();
        match args.get(nome) {
            Some(v) => out.push_str(v),
            None => out.push_str(&rest[i..i + j + 1]),
        }
        rest = &rest[i + j + 1..];
    }
    out.push_str(rest);
    out
}

/// Traduz `chave` na `locale`, com `args` para os `{nome}` da mensagem e para o
/// plural (o argumento `n`, ou `count`, ou o primeiro numérico).
/// Chave ausente → a própria chave (e um aviso no log).
pub fn translate(locale: &str, chave: &str, args: &HashMap<String, String>) -> String {
    let c = catalogo().read().unwrap_or_else(|e| e.into_inner());
    let loc = if locale.trim().is_empty() {
        c.padrao.clone().unwrap_or_default()
    } else {
        locale.to_string()
    };
    let Some(msg) = acha(&c, &loc, chave) else {
        eprintln!("[glacier i18n] chave `{chave}` não existe na locale `{loc}` nem na padrão");
        return chave.to_string();
    };
    match msg {
        Msg::Texto(t) => subst(t, args),
        Msg::Plural(formas) => {
            let n = ["n", "count"]
                .iter()
                .find_map(|k| args.get(*k))
                .or_else(|| args.values().find(|v| v.trim().parse::<f64>().is_ok()))
                .and_then(|v| v.trim().parse::<f64>().ok());
            let Some(n) = n else {
                eprintln!("[glacier i18n] `{chave}` é plural e a chamada não passou `n=`");
                return formas.get("other").map(|f| subst(f, args)).unwrap_or_else(|| chave.to_string());
            };
            let l = norm(&loc);
            let cat = categoria_plural(idioma(&l), n);
            let forma = formas
                .get(cat)
                .or_else(|| formas.get("other"))
                .or_else(|| formas.values().next());
            // `{n}` formatado pela locale (1.234 em pt, 1,234 em en), a menos que a chamada já o tenha formatado.
            let mut a = args.clone();
            for k in ["n", "count"] {
                if let Some(v) = a.get(k).cloned() {
                    if let Ok(x) = v.trim().parse::<f64>() {
                        let casas = if x.fract() == 0.0 { 0 } else { v.split('.').nth(1).map_or(0, str::len) };
                        a.insert(k.to_string(), formata_numero(&l, x, casas));
                    }
                }
            }
            forma.map(|f| subst(f, &a)).unwrap_or_else(|| chave.to_string())
        }
    }
}

/// Traduz pelo **texto de origem** (estilo gettext): o próprio texto escrito no código é a
/// chave do catálogo (`"Remover tag?"` → `"Remove tag?"`). Sem entrada na locale (nem na
/// padrão), devolve o texto de origem **sem aviso** — um texto ainda não traduzido não é um
/// erro, só aparece no idioma em que foi escrito. É o `i18n.tr` do Luau.
pub fn translate_source(locale: &str, source: &str) -> String {
    let c = catalogo().read().unwrap_or_else(|e| e.into_inner());
    let loc = if locale.trim().is_empty() {
        c.padrao.clone().unwrap_or_default()
    } else {
        locale.to_string()
    };
    match acha(&c, &loc, source) {
        Some(Msg::Texto(t)) => t.clone(),
        Some(Msg::Plural(f)) => f.get("other").cloned().unwrap_or_else(|| source.to_string()),
        None => source.to_string(),
    }
}

// ───────────────────────────── a chamada `{t(…)}` ─────────────────────────────

/// Os argumentos de uma chamada: `("chave", n=contagem, nome="lit")`.
/// Devolve (primeiro argumento, `nome → valor bruto`).
fn args_da_chamada(inside: &str) -> Option<(String, Vec<(String, Arg)>)> {
    let mut partes = Vec::new();
    let mut atual = String::new();
    let mut aspas: Option<char> = None;
    for ch in inside.chars() {
        match aspas {
            Some(q) => {
                atual.push(ch);
                if ch == q {
                    aspas = None;
                }
            }
            None => match ch {
                '"' | '\'' => {
                    aspas = Some(ch);
                    atual.push(ch);
                }
                ',' => partes.push(std::mem::take(&mut atual)),
                _ => atual.push(ch),
            },
        }
    }
    partes.push(atual);
    let mut it = partes.into_iter();
    let primeiro = it.next()?.trim().to_string();
    let mut resto = Vec::new();
    for p in it {
        let p = p.trim();
        if p.is_empty() {
            continue;
        }
        match p.split_once('=') {
            Some((k, v)) => resto.push((k.trim().to_string(), Arg::de(v.trim()))),
            None => resto.push((String::new(), Arg::de(p))),
        }
    }
    Some((primeiro, resto))
}

/// Um valor de argumento: literal (`"x"`, `'x'`, `42`) ou nome de chave do contexto.
#[derive(Debug, Clone)]
enum Arg {
    Literal(String),
    Chave(String),
}

impl Arg {
    fn de(s: &str) -> Arg {
        let b = s.as_bytes();
        if b.len() >= 2 && (b[0] == b'"' || b[0] == b'\'') && b[b.len() - 1] == b[0] {
            return Arg::Literal(s[1..s.len() - 1].to_string());
        }
        if s.parse::<f64>().is_ok() {
            return Arg::Literal(s.to_string());
        }
        Arg::Chave(s.trim_start_matches('@').to_string())
    }
}

/// Se `expr` (o texto entre `{` e `}`) é uma chamada `t(...)`, `num(...)` ou
/// `date(...)`, avalia-a. `leitura` consulta o contexto — e, por ser o
/// [`crate::eval::EvalCtx::get`], registra a dependência. `None` = não é uma chamada.
pub fn avalia_chamada(expr: &str, leitura: &mut dyn FnMut(&str) -> Option<String>) -> Option<String> {
    let e = expr.trim();
    let abre = e.find('(')?;
    let fn_nome = &e[..abre];
    if !matches!(fn_nome, "t" | "num" | "date") || !e.ends_with(')') {
        return None;
    }
    let (primeiro, resto) = args_da_chamada(&e[abre + 1..e.len() - 1])?;
    let loc = leitura(CHAVE_LOCALE)
        .filter(|l| !l.trim().is_empty())
        .or_else(active)
        .or_else(default_locale)
        .unwrap_or_else(|| "en".to_string());
    fn resolve(a: &Arg, leitura: &mut dyn FnMut(&str) -> Option<String>) -> String {
        match a {
            Arg::Literal(s) => s.clone(),
            Arg::Chave(k) => leitura(k).unwrap_or_default(),
        }
    }
    match fn_nome {
        "t" => {
            let mut args = HashMap::new();
            for (k, v) in &resto {
                if !k.is_empty() {
                    args.insert(k.clone(), resolve(v, leitura));
                }
            }
            let chave = match Arg::de(&primeiro) {
                Arg::Literal(chave) => chave,
                // `t(chave)` sem aspas: o nome de uma chave de contexto que guarda a chave de
                // tradução — e os argumentos (`t(estado, n=contagem)`) valem do mesmo jeito.
                Arg::Chave(k) => leitura(&k).unwrap_or_default(),
            };
            Some(translate(&loc, &chave, &args))
        }
        "num" => {
            let valor = resolve(&Arg::de(&primeiro), leitura);
            let casas = resto.first().map(|(_, a)| resolve(a, leitura)).and_then(|s| s.trim().parse().ok()).unwrap_or(0);
            Some(match valor.trim().parse::<f64>() {
                Ok(x) => formata_numero(&loc, x, casas),
                Err(_) => valor,
            })
        }
        "date" => {
            let valor = resolve(&Arg::de(&primeiro), leitura);
            Some(formata_data(&loc, &valor).unwrap_or(valor))
        }
        _ => None,
    }
}
