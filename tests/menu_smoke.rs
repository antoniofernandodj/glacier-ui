//! Exercita o caminho open→hover-submenu (2 níveis)→click→dismiss de
//! `crate::menu` sem precisar de mouse real: abre o `examples/menus`
//! component, dispara `OpenMenuBarDropdown`/`OpenContextMenu`/
//! `MenuHoverSubmenu`/`MenuItemClick`/`MenuDismiss` diretamente e confirma
//! que `render_current` não entra em pânico em nenhum estado intermediário.
use glacier_ui::widget::EngineMessage;
use glacier_ui::GlacierUI;
use std::sync::Arc;

#[test]
fn menu_open_hover_render_smoke() {
    let mut ui = GlacierUI::new();
    ui.register_component("menus", "examples/menus/menus.gv")
        .expect("register menus component");
    ui.set_initial_screen("menus");

    // Renderiza sem nenhum menu aberto.
    ui.render_current().expect("render sem menu aberto");

    let _ = ui.dispatch(&EngineMessage::CursorMoved(iced::Point::new(120.0, 40.0)));
    let _ = ui.dispatch(&EngineMessage::OpenMenuBarDropdown {
        tree: Arc::new(vec![
            glacier_ui::menu::MenuNode {
                label: "Ação simples".into(),
                icon: None,
                on_click: Some("acao_simples".into()),
                checked: None,
                disabled: false,
                separator: false,
                children: vec![],
            },
            glacier_ui::menu::MenuNode {
                label: "Com submenu".into(),
                icon: None,
                on_click: None,
                checked: None,
                disabled: false,
                separator: false,
                children: vec![glacier_ui::menu::MenuNode {
                    label: "Filho".into(),
                    icon: None,
                    on_click: Some("acao_filho".into()),
                    checked: None,
                    disabled: false,
                    separator: false,
                    children: vec![],
                }],
            },
        ]),
    });
    // Um menu está aberto: render_current precisa incluir a camada de overlay.
    ui.render_current().expect("render com menu aberto (raiz)");

    let _ = ui.dispatch(&EngineMessage::MenuHoverSubmenu { path: vec![1] });
    ui.render_current().expect("render com submenu aberto");

    let _ = ui.dispatch(&EngineMessage::MenuHoverSubmenu { path: vec![1, 0] });
    ui.render_current().expect("render com sub-submenu aberto (2 níveis)");

    let _ = ui.dispatch(&EngineMessage::MenuItemClick("acao_filho".into()));
    ui.render_current().expect("render após clique (menu fechado)");

    let _ = ui.dispatch(&EngineMessage::OpenContextMenu {
        tree: Arc::new(vec![glacier_ui::menu::MenuNode {
            label: "Contexto".into(),
            icon: None,
            on_click: Some("ctx_acao".into()),
            checked: None,
            disabled: false,
            separator: false,
            children: vec![],
        }]),
    });
    ui.render_current().expect("render com context menu aberto");

    let _ = ui.dispatch(&EngineMessage::MenuDismiss);
    ui.render_current().expect("render após dismiss");
}
