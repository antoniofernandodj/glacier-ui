//! Registro de **famílias de fonte** — o habilitador da Onda 10 do
//! `PLANO_WIDGETS.md`.
//!
//! # O buraco que ele tapa
//!
//! Antes desta onda três peças existiam e não se falavam:
//!
//! | Onde | O que havia |
//! |---|---|
//! | [`crate::widget`] (`font_for`) | conhecia **duas** fontes: `mono` e `bold`. Qualquer outro nome caía na fonte padrão, **em silêncio** |
//! | [`crate::stylesheet`] (`font-family`) | aceitava a string, guardava em `rule.font`… e caía no mesmo funil |
//! | [`crate::daemon::GlacierDaemon::font`] | **já carregava** `.ttf`/`.otf` no `iced`, e já sabia definir a padrão |
//!
//! O que faltava era o **nome** entre elas: a lista de bytes que o `iced`
//! carrega não guarda com que família cada um entrou, então um `.gss` não tinha
//! como pedir "Inter" e um `.gv` não tinha como pedir a mono do app. Este
//! módulo é esse nome — um `Vec<(String, Font)>` global, populado no boot por
//! [`GlacierDaemon::font_named`](crate::daemon::GlacierDaemon::font_named) e
//! consultado por `font_for` no render.
//!
//! # A armadilha do `&'static str`
//!
//! O `iced` casa família por [`iced::Font::with_name`], que quer um
//! `&'static str` — o nome precisa viver tanto quanto o app. [`register_family`]
//! resolve com um `Box::leak`: roda **uma vez por família**, no boot, número
//! fixo e pequeno (é o que os próprios exemplos do `iced` fazem). Um vazamento
//! proporcional ao número de fontes embutidas não é vazamento no sentido que
//! importa.
//!
//! # A decisão da §4 (fontes do SO)
//!
//! O `iced` não enumera as fontes do sistema. A escolha — uma crate de índice
//! ou o app declarar o que usa — mora aqui, atrás da feature **`system-fonts`**
//! (desligada por padrão, a forma do `tray-icon` da §2.9). Desligada, [`families_json`]
//! traz só as famílias registradas, que é o que um app empacotado quer mesmo;
//! ligada, acrescenta as do SO via `fontdb`. Ver [`system_families`].

use std::sync::RwLock;

use iced::Font;

/// `(nome como registrado, Font resolvida)`.
///
/// `Vec` e não `HashMap`: a lista é curta (uma família por peso que o app
/// embute, tipicamente menos de dez) e a **ordem de registro** é a ordem em que
/// o `<fontselect>` mostra as opções — um mapa a perderia.
static REGISTRO: RwLock<Vec<(String, Font)>> = RwLock::new(Vec::new());

/// Registra uma família sob `nome`, para que `font="<nome>"` no `.gv` e
/// `font_family: <nome>` no `.gss` passem a resolvê-la pelo caminho que **já
/// existe** (`font_for` → aqui).
///
/// Isto **não carrega bytes** — quem faz isso é
/// [`GlacierDaemon::font`](crate::daemon::GlacierDaemon::font), e
/// [`GlacierDaemon::font_named`](crate::daemon::GlacierDaemon::font_named)
/// chama os dois. Chamar `register_family` sozinho registra o nome de uma fonte
/// que o `iced` já conhece (uma embutida do renderer, ou uma do SO com a
/// feature `system-fonts`).
///
/// Idempotente: registrar o mesmo nome duas vezes (sem diferenciar caixa)
/// atualiza a entrada, não duplica. Devolve a [`Font`] resolvida, para o
/// chamador poder passá-la a
/// [`GlacierDaemon::default_font`](crate::daemon::GlacierDaemon::default_font).
pub fn register_family(nome: &str) -> Font {
    // O `iced` quer `&'static str`; um `Box::leak` por família, no boot, é a
    // saída honesta (ver o doc do módulo).
    let estatico: &'static str = Box::leak(nome.to_string().into_boxed_str());
    let font = Font::with_name(estatico);

    let mut reg = REGISTRO.write().unwrap_or_else(|e| e.into_inner());
    match reg.iter_mut().find(|(n, _)| n.eq_ignore_ascii_case(nome)) {
        Some(slot) => slot.1 = font,
        None => reg.push((nome.to_string(), font)),
    }
    font
}

/// Resolve um `font="..."`/`font-family` contra o registro, **sem diferenciar
/// caixa**.
///
/// `None` significa "não registrado" — e o chamador (`font_for`) cai na fonte
/// padrão, a **mesma degradação silenciosa de antes**, agora restrita a nomes
/// que ninguém registrou. Apelidos do motor (`mono`/`bold`) são tratados antes
/// desta chamada, em `font_for`, e não passam por aqui.
pub fn resolve(nome: &str) -> Option<Font> {
    let alvo = nome.trim();
    if alvo.is_empty() {
        return None;
    }
    let reg = REGISTRO.read().unwrap_or_else(|e| e.into_inner());
    reg.iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(alvo))
        .map(|(_, f)| *f)
}

/// As famílias registradas, na ordem de registro.
pub fn registered_families() -> Vec<String> {
    let reg = REGISTRO.read().unwrap_or_else(|e| e.into_inner());
    reg.iter().map(|(n, _)| n.clone()).collect()
}

/// As famílias disponíveis como um **array JSON de strings** — o formato que
/// `<combo items="__fonts">` e `<listview items="__fonts">` já consomem, sem
/// API nova (a mesma convenção do `<menu items="chave">`).
///
/// [`crate::GlacierUI::set_initial_screen`] semeia isto na chave `__fonts` do
/// contexto (a família do `__cal_hover`/`__band`/`__colgrip`).
///
/// Com a feature `system-fonts` ligada, as famílias do SO entram **depois** das
/// registradas, sem repetir uma que o app já tenha registrado com o mesmo nome.
pub fn families_json() -> String {
    // `mut` só é usado com a feature `system-fonts` ligada.
    #[cfg_attr(not(feature = "system-fonts"), allow(unused_mut))]
    let mut nomes = registered_families();

    #[cfg(feature = "system-fonts")]
    for familia in system_families() {
        if !nomes.iter().any(|n| n.eq_ignore_ascii_case(&familia)) {
            nomes.push(familia);
        }
    }

    serde_json::to_string(&nomes).unwrap_or_else(|_| "[]".to_string())
}

/// As famílias instaladas no sistema, em ordem alfabética e sem repetição.
///
/// Só existe com a feature **`system-fonts`**. O resultado é cacheado no
/// primeiro uso — `load_system_fonts` do `fontdb` varre diretórios e leva
/// dezenas de milissegundos, e a lista não muda durante a vida do processo.
///
/// **Nota honesta:** isto popula o *seletor*. Se uma família do SO **renderiza**
/// depende do renderer do `iced`/`cosmic-text` já tê-la carregado — este módulo
/// não injeta os bytes dela no `iced`. Para garantir o render de uma fonte
/// específica, o caminho continua sendo embuti-la com `font_named`.
#[cfg(feature = "system-fonts")]
pub fn system_families() -> Vec<String> {
    use std::sync::OnceLock;
    static CACHE: OnceLock<Vec<String>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let mut db = fontdb::Database::new();
            db.load_system_fonts();
            let mut fams: Vec<String> = db
                .faces()
                .filter_map(|face| face.families.first().map(|(nome, _)| nome.clone()))
                .collect();
            fams.sort_unstable();
            fams.dedup();
            fams
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registra_resolve_e_dedup_sem_caixa() {
        register_family("Prova Onda 10 Sans");
        assert!(resolve("prova onda 10 sans").is_some());
        assert!(resolve("PROVA ONDA 10 SANS").is_some());
        assert!(resolve("Inexistente Onda 10").is_none());
        assert!(resolve("   ").is_none());

        // Idempotente: o segundo registro do mesmo nome não duplica.
        register_family("prova onda 10 sans");
        let n = registered_families()
            .iter()
            .filter(|n| n.eq_ignore_ascii_case("prova onda 10 sans"))
            .count();
        assert_eq!(n, 1);
    }

    #[test]
    fn families_json_e_um_array() {
        let j = families_json();
        assert!(j.starts_with('['));
        assert!(j.ends_with(']'));
        let _: Vec<String> = serde_json::from_str(&j).expect("array de strings");
    }
}
