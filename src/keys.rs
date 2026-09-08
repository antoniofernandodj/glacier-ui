//! O **teclado** como capacidade — o habilitador B da Onda 9, e o item 9 do §3
//! do `PLANO_WIDGETS.md` (`Shortcut`/`Action` globais), que era o último item
//! daquela lista com consumidor no catálogo.
//!
//! # Ele parecia um item de motor e é uma entrada numa lista
//!
//! O daemon registra hoje cinco `iced::event::listen_with`, cada um uma
//! `fn(Event, Status, window::Id) -> Option<EngineMessage>` — `drag_end`,
//! `tab_focus`, `timeedit_key`, `cursor` e `menu_escape`. Este módulo é o
//! **sexto**, e as duas tags que ele entrega são o mesmo listener em dois
//! modos:
//!
//! - `<shortcut key="ctrl+s" on_press="salvar"/>` — casa a combinação contra os
//!   atalhos que a **árvore avaliada** declara (colhidos em
//!   `collect_tree_bindings`, ao lado dos `<textarea>` e `<comboedit>`);
//! - `<shortcutinput value="chave"/>` — o `QKeySequenceEdit`: em vez de casar, a
//!   combinação é **gravada**.
//!
//! # Por que o atalho mora no markup
//!
//! Porque ele pertence à **tela**. Uma tela que sai de cena leva os atalhos dela
//! junto, sem ninguém desregistrar nada — a árvore avaliada é a lista, e ela é
//! refeita a cada reavaliação. Uma API de Rust (`app.register_shortcut(…)`)
//! exigiria o par registrar/desregistrar e daria o bug clássico do atalho
//! fantasma: a tela antiga fechou e o `Ctrl+S` dela continua salvando.
//!
//! # A tecla de um campo focado não é um atalho
//!
//! `timeedit_key_from_event` já tinha aprendido isso (`if status == Captured {
//! return None }`) e a regra vale igual aqui: digitar `s` num `<textinput>` não
//! pode disparar o `<shortcut key="s">` da tela. O que muda é que uma
//! combinação **com modificador** (`ctrl+s`) atravessa mesmo com campo focado,
//! porque nenhum campo de texto consome `Ctrl+S` — e é justamente para isso que
//! um atalho existe.

use iced::keyboard::{Key, Modifiers, key::Named};

/// Qual `<shortcutinput>` está armado, ou vazio. **Um por tela**, e por isso sem
/// mapa: só um campo captura por vez. O valor é o nome da chave que ele edita —
/// a mesma forma do `__timeedit`, que guarda `"inicio:h"` desde a 0.70.
pub(crate) const CAPTURA_CONTEXT: &str = "__shortcut_cap";

/// Uma combinação de teclas, na grafia canônica deste motor:
/// `ctrl+shift+alt+super+<tecla>`, modificadores nessa ordem, tudo minúsculo.
///
/// A ordem é fixa **na saída** e livre na entrada: `"shift+ctrl+s"` escrito no
/// markup casa com o `ctrl+shift+s` que o teclado produz, porque os dois passam
/// por [`normaliza`] antes de se compararem.
pub fn normaliza(bruto: &str) -> String {
    let mut mods = [false; 4]; // ctrl, shift, alt, super
    let mut tecla = String::new();
    for parte in bruto.split(['+', '-']).map(str::trim) {
        match parte.to_ascii_lowercase().as_str() {
            "" => {}
            "ctrl" | "control" | "controle" => mods[0] = true,
            "shift" | "maiusc" | "maiúsc" => mods[1] = true,
            "alt" | "option" => mods[2] = true,
            "super" | "cmd" | "meta" | "win" | "windows" => mods[3] = true,
            outro => tecla = outro.to_string(),
        }
    }
    monta(mods, &tecla)
}

fn monta(mods: [bool; 4], tecla: &str) -> String {
    let mut s = String::new();
    for (ativo, nome) in mods.iter().zip(["ctrl", "shift", "alt", "super"]) {
        if *ativo {
            s.push_str(nome);
            s.push('+');
        }
    }
    s.push_str(tecla);
    s
}

/// A combinação que um evento de tecla produz, ou `None` quando o evento não é
/// uma combinação **completa**.
///
/// Apertar só `Ctrl` não é um atalho — é o começo de um. Um `<shortcutinput>`
/// que gravasse `"ctrl+"` no primeiro modificador nunca chegaria a gravar
/// `ctrl+s`, porque o modificador chega antes da letra.
pub fn combinacao(key: &Key, modifiers: Modifiers) -> Option<String> {
    let mods = [
        modifiers.control(),
        modifiers.shift(),
        modifiers.alt(),
        modifiers.logo(),
    ];
    let tecla = match key {
        Key::Character(c) => {
            let c = c.trim();
            if c.is_empty() {
                return None;
            }
            // O `shift` já está no modificador; deixar a letra maiúscula faria
            // `shift+S` e `shift+s` serem atalhos diferentes.
            c.to_lowercase()
        }
        Key::Named(n) => match nome_de(*n) {
            Some(nome) => nome.to_string(),
            // Um modificador sozinho não é combinação.
            None => return None,
        },
        Key::Unidentified => return None,
    };
    Some(monta(mods, &tecla))
}

/// O nome de uma tecla nomeada na grafia do markup. `None` para as que são só
/// modificador — elas nunca formam uma combinação sozinhas.
fn nome_de(n: Named) -> Option<&'static str> {
    Some(match n {
        Named::Enter => "enter",
        Named::Escape => "escape",
        Named::Tab => "tab",
        Named::Space => "space",
        Named::Backspace => "backspace",
        Named::Delete => "delete",
        Named::ArrowUp => "up",
        Named::ArrowDown => "down",
        Named::ArrowLeft => "left",
        Named::ArrowRight => "right",
        Named::Home => "home",
        Named::End => "end",
        Named::PageUp => "pageup",
        Named::PageDown => "pagedown",
        Named::Insert => "insert",
        Named::F1 => "f1",
        Named::F2 => "f2",
        Named::F3 => "f3",
        Named::F4 => "f4",
        Named::F5 => "f5",
        Named::F6 => "f6",
        Named::F7 => "f7",
        Named::F8 => "f8",
        Named::F9 => "f9",
        Named::F10 => "f10",
        Named::F11 => "f11",
        Named::F12 => "f12",
        _ => return None,
    })
}

/// Um atalho declarado só vale se tiver **as duas** metades. Um
/// `<shortcut key="ctrl+s"/>` sem ação é um nó decorativo, e um sem `key` casaria
/// com a combinação vazia — que é o que um modificador solto produziria se
/// [`combinacao`] não o filtrasse.
pub fn declarado(key: &str, action: &str) -> bool {
    !key.trim().is_empty() && !action.trim().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ordem_dos_modificadores_nao_importa_na_entrada() {
        assert_eq!(normaliza("shift+Ctrl+S"), "ctrl+shift+s");
        assert_eq!(normaliza("ctrl+shift+s"), "ctrl+shift+s");
        assert_eq!(normaliza("F5"), "f5");
        assert_eq!(normaliza(" ctrl + p "), "ctrl+p");
    }

    #[test]
    fn o_teclado_e_o_markup_chegam_na_mesma_grafia() {
        let m = Modifiers::CTRL | Modifiers::SHIFT;
        let k = Key::Character("S".into());
        assert_eq!(combinacao(&k, m).as_deref(), Some("ctrl+shift+s"));
        assert_eq!(combinacao(&k, m).unwrap(), normaliza("Shift+Ctrl+s"));
    }

    #[test]
    fn um_modificador_sozinho_nao_e_combinacao() {
        // Senão um `<shortcutinput>` gravaria `"ctrl+"` e nunca chegaria ao
        // `ctrl+s` — o modificador chega antes da letra.
        assert_eq!(
            combinacao(&Key::Named(Named::Control), Modifiers::CTRL),
            None
        );
        assert_eq!(combinacao(&Key::Unidentified, Modifiers::empty()), None);
        assert_eq!(
            combinacao(&Key::Character(" ".into()), Modifiers::empty()),
            None
        );
    }

    #[test]
    fn teclas_nomeadas_tem_nome_e_nao_debug() {
        assert_eq!(
            combinacao(&Key::Named(Named::F5), Modifiers::empty()).as_deref(),
            Some("f5")
        );
        assert_eq!(
            combinacao(&Key::Named(Named::ArrowUp), Modifiers::ALT).as_deref(),
            Some("alt+up")
        );
    }

    #[test]
    fn um_atalho_pela_metade_nao_e_atalho() {
        assert!(declarado("ctrl+s", "salvar"));
        assert!(!declarado("ctrl+s", "  "));
        assert!(!declarado("", "salvar"));
    }
}
