//! Fase 10 do `docs/PLANO_LINGUAGEM_GV_GSS.md`: a armadilha do `Fill` (S8) e o
//! layout sem janela que a detecta (S9).

use glacier_ui::GlacierUI;

fn motor_com(nome: &str, src: &str) -> GlacierUI {
    let dir = std::env::temp_dir().join("glacier_linguagem_layout");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{nome}.gv"));
    std::fs::write(&path, src).unwrap();
    let mut motor = GlacierUI::new();
    motor.register_component(nome, path.to_str().unwrap()).unwrap();
    motor
}

const JANELA: iced::Size = iced::Size::new(800.0, 600.0);

/// O caso real do rustploy (2026-11-19): um `column` SEM `width` cujos filhos são
/// todos `fill`. O glacier forçava `Shrink` na coluna e os filhos ficavam com
/// largura zero; agora a coluna herda o `Fill` deles, como o `iced` calcula.
#[test]
fn coluna_sem_width_com_filhos_fill_nao_colapsa() {
    let mut m = motor_com(
        "form_backup",
        r#"component {
  column(width = fill) {
    column {
      row(width = fill) {
        text("Agendar backup", width = fill)
      }
      textinput(:value = cron, width = fill)
    }
  }
}
"#,
    );
    let c = m.layout_collapses("form_backup", JANELA).unwrap();
    assert!(c.is_empty(), "nada devia colapsar: {c:#?}");
}

/// O mesmo com fundo (o wrap de background/borda era a segunda fonte do
/// `Shrink` forçado).
#[test]
fn coluna_com_fundo_e_filhos_fill_nao_colapsa() {
    let mut m = motor_com(
        "com_fundo",
        r#"component {
  resources {
    style """
    .painel { background: #222222; border_radius: 4; }
    """
  }
  column(width = fill) {
    column(class = painel) {
      textinput(:value = nome, width = fill)
    }
  }
}
"#,
    );
    let c = m.layout_collapses("com_fundo", JANELA).unwrap();
    assert!(c.is_empty(), "nada devia colapsar: {c:#?}");
}

/// O detector acha o que colapsa de verdade, e descreve o caminho até o nó.
#[test]
fn detector_aponta_o_no_sem_area() {
    let mut m = motor_com(
        "espremido",
        r#"component {
  column(width = fill) {
    row(class = barra, width = 0.5) {
      text("título que não cabe")
    }
  }
}
"#,
    );
    let c = m.layout_collapses("espremido", JANELA).unwrap();
    // 0.5px não é zero; este caso tem de passar limpo — o detector não inventa.
    assert!(c.is_empty(), "{c:#?}");

    // `width = shrink` DECLARADO com filhos `fill`: a armadilha antiga, de
    // propósito. O detector tem de achá-la e dizer onde está.
    let mut m = motor_com(
        "zerado",
        r#"component {
  column(width = fill) {
    column(class = form, width = shrink) {
      textinput(:value = cron, width = fill)
    }
  }
}
"#,
    );
    let c = m.layout_collapses("zerado", JANELA).unwrap();
    assert_eq!(c.len(), 1, "{c:#?}");
    assert!(c[0].caminho.contains("column.form"), "{}", c[0]);
    assert!(c[0].to_string().contains("largura zero"), "{}", c[0]);
}

/// Fase 12: `@theme` tira a paleta do próprio `.gss` (um lugar só), resolvendo
/// `var()` contra o `:root` dele.
#[test]
fn theme_do_gss_vira_o_tema_do_app() {
    let dir = std::env::temp_dir().join("glacier_linguagem_layout");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("tema.gss");
    std::fs::write(
        &path,
        ":root { --bg: #0A0E14; --tx: #E6EDF3; }\n@theme {\n  background: var(--bg);\n  text: var(--tx);\n  primary: #58A6FF;\n  success: #3FB950;\n  danger: #F85149;\n}\n",
    )
    .unwrap();
    let mut m = GlacierUI::new();
    m.load_stylesheet(path.to_str().unwrap()).unwrap();
    let pal = m.theme().palette();
    let bg = pal.background;
    assert!((bg.r - 0x0A as f32 / 255.0).abs() < 0.01, "{bg:?}");
    assert!((pal.primary.b - 0xFF as f32 / 255.0).abs() < 0.01, "{:?}", pal.primary);
}
