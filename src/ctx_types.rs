//! O tipo que um valor do `ctx` tinha quando o script o gravou.
//!
//! O contexto do motor é `String → String` ([`crate::ContextMap`]) e continua
//! sendo: o markup, o avaliador e os widgets leem texto. O que este módulo
//! acrescenta é um **mapa de tags ao lado** — `chave → (tipo, texto)` — para o
//! Luau poder devolver `true` como `true` e `3` como `3`, em vez de `"true"` e
//! `"3"` (e o `false` que voltava como a string `"false"`, que é verdadeira).
//!
//! # A regra que impede um tipo errado
//!
//! A tag guarda o **texto** que acompanhava o valor quando foi gravado
//! (`snapshot`). Ela só vale enquanto o texto atual da chave for igual a esse
//! snapshot. Se qualquer outro caminho sobrescreveu a chave — um widget sem tag,
//! `Context::set` do Rust, o patch de um efeito assíncrono —, o texto mudou, a
//! tag está obsoleta e a chave volta a ser uma string comum. Nunca se ressuscita
//! um tipo que não é mais o do valor.
//!
//! # Canônicos
//!
//! - `Bool`: `"true"` / `"false"`;
//! - `Json`: o JSON compacto de uma tabela Luau (lista de objetos, objeto). Uma
//!   tabela **vazia sem marca** vira `[]`, não `{}`; `json.object({})` é que dá `{}`;
//! - `Number`: um inteiro sem `.0` quando cabe exato num `f64` (até 2^53), senão
//!   a forma mais curta que volta ao mesmo `f64` (`1e300` em notação científica).
//!   `NaN` e `±inf` não têm representação e não são gravados.

use crate::ContextMap;
use crate::eval::Fx;
use std::collections::HashMap;

/// O tipo de um valor do `ctx`. Ausente do mapa = string (o padrão).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tag {
    Bool,
    Number,
    /// Uma tabela de dados: o texto é o JSON compacto dela.
    Json,
}

/// Uma tag e o texto que a validava quando foi gravada.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Tipado {
    pub(crate) tag: Tag,
    pub(crate) snapshot: String,
}

/// `chave → Tipado`. Mora ao lado do [`ContextMap`] de quem o dono é o motor.
pub(crate) type TagMap = HashMap<String, Tipado, Fx>;

/// O mapa de tags de um [`crate::component::Context`]: do motor (emprestado,
/// persiste entre turnos) ou próprio (o `Context::new` dos testes unitários).
pub(crate) enum Tags<'a> {
    #[allow(dead_code)] // só o `Context::new`, dos testes unitários, o constrói
    Proprio(TagMap),
    Emprestado(&'a mut TagMap),
}

impl Tags<'_> {
    pub(crate) fn map(&self) -> &TagMap {
        match self {
            Tags::Proprio(m) => m,
            Tags::Emprestado(m) => m,
        }
    }

    pub(crate) fn map_mut(&mut self) -> &mut TagMap {
        match self {
            Tags::Proprio(m) => m,
            Tags::Emprestado(m) => m,
        }
    }
}

/// A tag de `chave` **se ainda valer**: o texto atual de `data[chave]` tem de ser
/// igual ao snapshot gravado com a tag.
pub(crate) fn tag_valida(tags: &TagMap, data: &ContextMap, chave: &str) -> Option<Tag> {
    let t = tags.get(chave)?;
    (data.get(chave)? == &t.snapshot).then_some(t.tag)
}

/// O texto canônico de um número, ou `None` para `NaN`/`±inf`.
pub(crate) fn numero_para_texto(n: f64) -> Option<String> {
    if !n.is_finite() {
        return None;
    }
    // 2^53: acima disso um f64 inteiro já não representa todo inteiro.
    const EXATO: f64 = 9_007_199_254_740_992.0;
    if n.fract() == 0.0 && n.abs() <= EXATO {
        // `-0.0` vira "0": o Luau não distingue, e "-0" confundiria quem compara texto.
        return Some(format!("{}", n as i64));
    }
    let abs = n.abs();
    Some(if abs >= 1e16 || (abs != 0.0 && abs < 1e-5) {
        format!("{n:e}")
    } else {
        format!("{n}")
    })
}

/// O número que `texto` guarda (o inverso de [`numero_para_texto`]).
pub(crate) fn texto_para_numero(texto: &str) -> Option<f64> {
    texto.parse::<f64>().ok().filter(|n| n.is_finite())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeros_canonicos() {
        assert_eq!(numero_para_texto(3.0).as_deref(), Some("3"));
        assert_eq!(numero_para_texto(-0.0).as_deref(), Some("0"));
        assert_eq!(numero_para_texto(3.5).as_deref(), Some("3.5"));
        assert_eq!(numero_para_texto(0.1).as_deref(), Some("0.1"));
        assert_eq!(numero_para_texto(1e300).as_deref(), Some("1e300"));
        assert_eq!(numero_para_texto(1.5e-9).as_deref(), Some("1.5e-9"));
        assert_eq!(numero_para_texto(f64::NAN), None);
        assert_eq!(numero_para_texto(f64::INFINITY), None);
        // O grande acima de 2^53 não vira `i64::MAX`.
        assert_eq!(numero_para_texto(1e20).as_deref(), Some("1e20"));
    }

    #[test]
    fn numero_volta_ao_mesmo_valor() {
        for n in [0.0, 1.0, -7.0, 3.5, 0.1, 1e300, 1.5e-9, 9_007_199_254_740_992.0] {
            let t = numero_para_texto(n).unwrap();
            assert_eq!(texto_para_numero(&t), Some(n), "{t}");
        }
    }

    #[test]
    fn tag_obsoleta_nao_vale() {
        let mut data = ContextMap::default();
        let mut tags = TagMap::default();
        data.insert("ativo".into(), "true".into());
        tags.insert(
            "ativo".into(),
            Tipado {
                tag: Tag::Bool,
                snapshot: "true".into(),
            },
        );
        assert_eq!(tag_valida(&tags, &data, "ativo"), Some(Tag::Bool));
        // Um widget (ou o Rust) escreveu outro texto: a tag não vale mais.
        data.insert("ativo".into(), "false".into());
        assert_eq!(tag_valida(&tags, &data, "ativo"), None);
        data.remove("ativo");
        assert_eq!(tag_valida(&tags, &data, "ativo"), None);
    }
}
