//! Ler um campo de uma tabela do `ctx` pelo caminho: `@msg.ola`, `{cfg.db.host}`,
//! `@itens.0.nome`.
//!
//! O contexto é um mapa plano `chave → texto`. Um script que grava uma tabela
//! (`ctx.msg = { ola = "mundo" }`) grava **uma** chave, `msg`, com o JSON dela; a
//! chave `msg.ola` nunca existe. Sem este módulo o markup lia `msg.ola` como uma
//! chave plana ausente e interpolava texto vazio — sem erro nem aviso.
//!
//! # A regra
//!
//! 1. A chave **exata** vence, sempre (é o comportamento de antes, e uma chave
//!    que tem ponto no próprio nome continua funcionando). Isto só roda quando a
//!    chave exata **não** existe.
//! 2. Procura-se o **maior prefixo** (cortado em pontos) que existe como chave. Se o
//!    valor dele é um JSON de objeto ou de lista, desce-se pelos segmentos que
//!    sobram: nome do campo num objeto; número (base 0) num array.
//! 3. A folha sai como o contexto guarda um valor: string crua; número e booleano
//!    pelo texto canônico; objeto e lista como o JSON em texto (o que `spread=` e
//!    `for-each` esperam receber de volta). `null`, campo ou índice inexistente =
//!    ausente — e então vale o `{msg.x|padrão}`.
//!
//! Só **leitura**. Não existe "escrever num caminho": `ctx["msg.ola"] = x` grava uma
//! chave plana de nome `msg.ola`, que passa a **sombrear** o campo da tabela (a
//! chave exata vence).
//!
//! # Custo
//!
//! O JSON de uma chave é interpretado uma vez e reaproveitado enquanto o texto
//! dela for o mesmo ([`MEMO`]); ler dez campos do mesmo objeto numa avaliação
//! interpreta o objeto uma vez.

use crate::eval::Fx;
use serde_json::Value;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// Quantas chaves JSON o memo guarda. Passou disso, esvazia (um app com tantos
/// objetos lidos por caminho é raro, e recomeçar é barato).
const MEMO_MAX: usize = 64;

thread_local! {
    /// `chave → (texto, valor interpretado)`. Válido enquanto o texto for igual.
    static MEMO: RefCell<HashMap<String, (String, Rc<Value>), Fx>> = RefCell::new(HashMap::default());
    /// Chaves que já geraram o aviso de "o prefixo não é JSON", para avisar uma vez.
    static AVISADAS: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
}

/// O JSON de `texto` se for um objeto ou uma lista, reaproveitando o parse
/// anterior da mesma `chave` quando o texto não mudou.
fn container(chave: &str, texto: &str) -> Option<Rc<Value>> {
    // Barato de descartar: um texto que não começa como container nunca é.
    let primeiro = texto.trim_start().as_bytes().first()?;
    if *primeiro != b'{' && *primeiro != b'[' {
        return None;
    }
    MEMO.with(|m| {
        let mut m = m.borrow_mut();
        if let Some((t, v)) = m.get(chave)
            && t == texto
        {
            return Some(Rc::clone(v));
        }
        let v: Value = serde_json::from_str(texto).ok()?;
        if !v.is_object() && !v.is_array() {
            return None;
        }
        if m.len() >= MEMO_MAX {
            m.clear();
        }
        let v = Rc::new(v);
        m.insert(chave.to_string(), (texto.to_string(), Rc::clone(&v)));
        Some(v)
    })
}

/// O texto de uma folha como o contexto o guarda; `None` para `null`.
fn folha(v: &Value) -> Option<String> {
    match v {
        Value::Null => None,
        Value::String(s) => Some(s.clone()),
        outro => Some(outro.to_string()),
    }
}

/// Desce por `segmentos` a partir de `v`.
fn descer<'v>(mut v: &'v Value, segmentos: &str) -> Option<&'v Value> {
    for seg in segmentos.split('.') {
        v = match v {
            Value::Object(m) => m.get(seg)?,
            Value::Array(a) => a.get(seg.parse::<usize>().ok()?)?,
            _ => return None,
        };
    }
    Some(v)
}

/// Avisa (uma vez por chave) que o prefixo existe mas não é um objeto/lista JSON:
/// quem escreve `@msg.ola` quase sempre gravou `msg` como texto e espera campos.
fn avisa_prefixo_nao_json(chave: &str, prefixo: &str) {
    AVISADAS.with(|a| {
        if a.borrow_mut().insert(chave.to_string()) {
            eprintln!(
                "[glacier-ui] aviso: '{chave}' não resolve: a chave '{prefixo}' existe, mas \
                 não guarda um JSON de objeto ou de lista (grave uma tabela: \
                 `ctx.{prefixo} = {{ ... }}`)"
            );
        }
    });
}

/// A folha em `caminho` dentro de um valor JSON que o chamador já tem interpretado
/// (o item de um `for-each`): desce sem reserializar nem reinterpretar.
pub(crate) fn folha_em(v: &Value, caminho: &str) -> Option<String> {
    descer(v, caminho).and_then(folha)
}

/// Resolve `chave` como um caminho dentro de um valor JSON do contexto, **depois** de
/// a chave exata ter falhado. `exata(k)` devolve o texto de `k` e a profundidade da
/// camada que o deu (0 = base); a profundidade do resultado é a do prefixo achado,
/// o que mantém o rastreamento de dependências do avaliador correto.
pub(crate) fn resolver<'a>(
    chave: &str,
    mut exata: impl FnMut(&str) -> Option<(Cow<'a, str>, u32)>,
) -> Option<(Cow<'a, str>, u32)> {
    // Posições dos pontos, do último para o primeiro: o maior prefixo primeiro.
    let pontos: Vec<usize> = chave.match_indices('.').map(|(i, _)| i).collect();
    for &p in pontos.iter().rev() {
        let prefixo = &chave[..p];
        if prefixo.is_empty() {
            continue;
        }
        let Some((texto, prof)) = exata(prefixo) else {
            continue;
        };
        let Some(raiz) = container(prefixo, &texto) else {
            // Existe, mas não é um container: não há campo a descer. Um prefixo mais
            // curto ainda pode ser (uma chave plana de nome `a.b` ao lado de `a`).
            avisa_prefixo_nao_json(chave, prefixo);
            continue;
        };
        let resto = &chave[p + 1..];
        // O primeiro prefixo que é container decide: achou a folha ou é ausente.
        return descer(&raiz, resto)
            .and_then(folha)
            .map(|t| (Cow::Owned(t), prof));
    }
    None
}

/// O mesmo para quem só tem o `ContextMap` (os widgets): a chave exata, ou o caminho.
pub(crate) fn get<'a>(ctx: &'a crate::ContextMap, chave: &str) -> Option<Cow<'a, str>> {
    if let Some(v) = ctx.get(chave) {
        return Some(Cow::Borrowed(v.as_str()));
    }
    if !chave.contains('.') {
        return None;
    }
    resolver(chave, |k| {
        ctx.get(k).map(|v| (Cow::Borrowed(v.as_str()), 0))
    })
    .map(|(v, _)| v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ContextMap;

    fn ctx(pares: &[(&str, &str)]) -> ContextMap {
        pares
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn lê(c: &ContextMap, k: &str) -> Option<String> {
        get(c, k).map(|v| v.into_owned())
    }

    #[test]
    fn campo_de_objeto_e_aninhado() {
        let c = ctx(&[("msg", r#"{"ola":"mundo","a":{"b":{"c":"fundo"}}}"#)]);
        assert_eq!(lê(&c, "msg.ola").as_deref(), Some("mundo"));
        assert_eq!(lê(&c, "msg.a.b.c").as_deref(), Some("fundo"));
    }

    #[test]
    fn indice_de_array_base_zero() {
        let c = ctx(&[("itens", r#"[{"nome":"x"},{"nome":"y"}]"#)]);
        assert_eq!(lê(&c, "itens.0.nome").as_deref(), Some("x"));
        assert_eq!(lê(&c, "itens.1.nome").as_deref(), Some("y"));
        assert_eq!(lê(&c, "itens.2.nome"), None);
    }

    #[test]
    fn folhas_numero_booleano_e_container() {
        let c = ctx(&[("d", r#"{"n":3,"f":1.5,"b":true,"o":{"k":1},"l":[1,2],"z":null}"#)]);
        assert_eq!(lê(&c, "d.n").as_deref(), Some("3"));
        assert_eq!(lê(&c, "d.f").as_deref(), Some("1.5"));
        assert_eq!(lê(&c, "d.b").as_deref(), Some("true"));
        assert_eq!(lê(&c, "d.o").as_deref(), Some(r#"{"k":1}"#));
        assert_eq!(lê(&c, "d.l").as_deref(), Some("[1,2]"));
        assert_eq!(lê(&c, "d.z"), None); // null = ausente
        assert_eq!(lê(&c, "d.nao_existe"), None);
    }

    #[test]
    fn a_chave_exata_vence_e_sombreia_o_campo() {
        let c = ctx(&[("msg", r#"{"ola":"da tabela"}"#), ("msg.ola", "plana")]);
        assert_eq!(lê(&c, "msg.ola").as_deref(), Some("plana"));
    }

    #[test]
    fn prefixo_que_nao_e_json_nao_resolve() {
        let c = ctx(&[("msg", "texto qualquer")]);
        assert_eq!(lê(&c, "msg.ola"), None);
        let c = ctx(&[("n", "42")]);
        assert_eq!(lê(&c, "n.x"), None);
    }

    #[test]
    fn prefixo_ausente_e_chave_sem_ponto() {
        let c = ctx(&[("a", r#"{"b":1}"#)]);
        assert_eq!(lê(&c, "x.b"), None);
        assert_eq!(lê(&c, "a").as_deref(), Some(r#"{"b":1}"#)); // a exata, como sempre
        assert_eq!(lê(&c, "sem_ponto"), None);
    }

    #[test]
    fn maior_prefixo_existente_vence() {
        // `a.b` existe como chave plana (um container) e `a` também.
        let c = ctx(&[("a", r#"{"b":{"c":"de a"}}"#), ("a.b", r#"{"c":"de a.b"}"#)]);
        assert_eq!(lê(&c, "a.b.c").as_deref(), Some("de a.b"));
    }

    #[test]
    fn muda_o_texto_muda_o_resultado() {
        let mut c = ctx(&[("m", r#"{"x":"um"}"#)]);
        assert_eq!(lê(&c, "m.x").as_deref(), Some("um"));
        c.insert("m".into(), r#"{"x":"dois"}"#.into());
        assert_eq!(lê(&c, "m.x").as_deref(), Some("dois"));
    }
}
