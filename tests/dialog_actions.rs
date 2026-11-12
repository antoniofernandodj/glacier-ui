//! O bloco `actions { button(…) }` do `dialog` e o `open_dialog()` do Luau.
//!
//! Guardam três coisas: (1) os botões escritos como filhos viram os mesmos
//! `DialogButton` da lista compacta `buttons=`, que continua valendo; (2) o
//! handler abre um `dialog` declarado, e a resposta é a **ação** do botão, nunca
//! um valor de retorno; (3) o `confirm{}` segue devolvendo só booleano.

use glacier_ui::{ButtonRole, DIALOG_CLOSE, EngineMessage, GlacierUI, NodeType};

fn escreve(nome: &str, ext: &str, conteudo: &str) -> String {
    std::fs::create_dir_all("templates").ok();
    let caminho = format!("templates/{nome}.{ext}");
    std::fs::write(&caminho, conteudo).unwrap();
    caminho
}

fn abre(motor: &mut GlacierUI, nome: &str) {
    let _ = motor.dispatch(&EngineMessage::UiClick(format!("dialog:{nome}")));
}

#[test]
fn actions_viram_botoes_na_ordem_com_papeis_e_acoes() {
    let mut motor = GlacierUI::new();
    let c = escreve(
        "da_ordem",
        "xml",
        r##"<screen>
              <resources>
                <dialog name="d" title="T">
                  <text content="corpo" />
                  <actions>
                    <button text="Rejeitar" />
                    <button text="Só o serviço" on_click="so" role="neutral" />
                    <button text="Aceitar" on_click="ok" />
                    <button text="Remover" on_click="rm" role="destructive" />
                    <button text="Voltar" on_click="dialog:outro" role="neutral" />
                  </actions>
                </dialog>
              </resources>
              <text content="tela" />
            </screen>"##,
    );
    motor.register_component("tela", &c).unwrap();
    motor.navigate_to("tela");
    abre(&mut motor, "d");
    let spec = motor.dialog().expect("aberto");
    let b = &spec.buttons;
    assert_eq!(b.len(), 5);
    assert_eq!(b[0].label, "Rejeitar");
    assert_eq!(b[0].action, DIALOG_CLOSE, "sem on_click só fecha");
    assert_eq!(b[0].role, ButtonRole::Neutral);
    assert_eq!((b[1].label.as_str(), b[1].action.as_str()), ("Só o serviço", "so"));
    assert_eq!(b[1].role, ButtonRole::Neutral, "role escrito manda");
    assert_eq!(b[2].action, "ok");
    assert_eq!(b[2].role, ButtonRole::Accept, "com ação, o papel default é accept");
    assert_eq!(b[3].role, ButtonRole::Destructive);
    assert_eq!(b[4].action, "dialog:outro", "encadeia noutro diálogo");
    std::fs::remove_file(&c).ok();
}

#[test]
fn actions_nao_entram_no_corpo() {
    let mut motor = GlacierUI::new();
    let c = escreve(
        "sem_corpo_extra",
        "xml",
        r##"<screen>
              <resources>
                <dialog name="d2" title="T">
                  <column><text content="corpo" /></column>
                  <actions><button text="Ok" /></actions>
                </dialog>
              </resources>
              <text content="tela" />
            </screen>"##,
    );
    motor.register_component("tela", &c).unwrap();
    let corpo = motor.evaluated("d2").expect("corpo registrado");
    assert_eq!(
        corpo.kind,
        NodeType::Column,
        "o `actions` é moldura: se vazasse para o corpo ele viraria um Fragment"
    );
    std::fs::remove_file(&c).ok();
}

#[test]
fn actions_mandam_sobre_buttons_e_vazio_nao_tem_botao() {
    let mut motor = GlacierUI::new();
    let c = escreve(
        "manda",
        "xml",
        r##"<screen>
              <resources>
                <dialog name="a" title="A" buttons="Velho::|Antigo:x:accept">
                  <text content="c" />
                  <actions><button text="Novo" on_click="n" /></actions>
                </dialog>
                <dialog name="b" title="B"><text content="c" /><actions /></dialog>
              </resources>
              <text content="tela" />
            </screen>"##,
    );
    motor.register_component("tela", &c).unwrap();
    motor.navigate_to("tela");
    abre(&mut motor, "a");
    let spec = motor.dialog().unwrap();
    assert_eq!(spec.buttons.len(), 1);
    assert_eq!(spec.buttons[0].label, "Novo");
    abre(&mut motor, "b");
    assert!(motor.dialog().unwrap().buttons.is_empty(), "`actions` vazio = nenhum botão");
    std::fs::remove_file(&c).ok();
}

/// Regressão: a lista compacta, que existia antes, segue igual.
#[test]
fn buttons_compacto_continua_funcionando() {
    let mut motor = GlacierUI::new();
    let c = escreve(
        "compacto",
        "xml",
        r##"<screen>
              <resources>
                <dialog name="e" title="E" buttons="Cancelar::|Salvar:salvar:accept|Voltar:dialog:x:neutral">
                  <text content="c" />
                </dialog>
                <dialog name="f" title="F"><text content="c" /></dialog>
              </resources>
              <text content="tela" />
            </screen>"##,
    );
    motor.register_component("tela", &c).unwrap();
    motor.navigate_to("tela");
    abre(&mut motor, "e");
    let b = motor.dialog().unwrap().buttons.clone();
    assert_eq!(b.len(), 3);
    assert_eq!(b[0].action, DIALOG_CLOSE);
    assert_eq!(b[1].action, "salvar");
    assert_eq!(b[2].action, "dialog:x");
    abre(&mut motor, "f");
    let b = motor.dialog().unwrap().buttons.clone();
    assert_eq!(b.len(), 1, "sem buttons nem actions: um Fechar");
    assert_eq!(b[0].action, DIALOG_CLOSE);
    std::fs::remove_file(&c).ok();
}

/// A sintaxe `.gv` de verdade (desugar + parser): `button("rótulo", on_click = …)`.
#[test]
fn actions_em_gv() {
    let mut motor = GlacierUI::new();
    let c = escreve(
        "em_gv",
        "gv",
        r#"screen {
  resources {
    dialog(name = d3, title = "Deploy", icon = question) {
      column { text("corpo") }
      actions {
        button("Rejeitar")
        button("Só o serviço", on_click = so)
        button("Aceitar", on_click = ok, role = accept)
      }
    }
  }
  text("tela")
}
"#,
    );
    motor.register_component("tela", &c).unwrap();
    motor.navigate_to("tela");
    abre(&mut motor, "d3");
    let spec = motor.dialog().expect("aberto");
    let rot: Vec<_> = spec.buttons.iter().map(|b| b.label.as_str()).collect();
    assert_eq!(rot, ["Rejeitar", "Só o serviço", "Aceitar"]);
    assert_eq!(spec.buttons[0].action, DIALOG_CLOSE);
    assert_eq!(spec.buttons[1].action, "so");
    assert_eq!(spec.buttons[2].role, ButtonRole::Accept);
    std::fs::remove_file(&c).ok();
}

/// Título, mensagem e rótulos resolvem `{chave}` na abertura.
#[test]
fn rotulos_e_titulo_interpolam_ao_abrir() {
    let mut motor = GlacierUI::new();
    let c = escreve(
        "interpola",
        "xml",
        r##"<screen>
              <resources>
                <dialog name="i" title="Deploy {servico}">
                  <text content="c" />
                  <actions><button text="Subir {servico}" on_click="ok" /></actions>
                </dialog>
              </resources>
              <text content="tela" />
            </screen>"##,
    );
    motor.register_component("tela", &c).unwrap();
    motor.navigate_to("tela");
    motor.define_data("servico", "api");
    abre(&mut motor, "i");
    let spec = motor.dialog().unwrap();
    assert_eq!(spec.title, "Deploy api");
    assert_eq!(spec.buttons[0].label, "Subir api");
    std::fs::remove_file(&c).ok();
}

/// `choice(…, value = v)` vira um botão cuja ação é a sentinela com o valor.
#[test]
fn choice_vira_botao_que_devolve_valor() {
    let mut motor = GlacierUI::new();
    let c = escreve(
        "choice_basico",
        "xml",
        r##"<screen>
              <resources>
                <dialog name="c" title="T">
                  <text content="corpo" />
                  <actions>
                    <button text="Rejeitar" />
                    <choice text="Só este" value="so" role="neutral" />
                    <choice text="Aceitar" value="aceitar" role="accept" />
                  </actions>
                </dialog>
              </resources>
              <text content="tela" />
            </screen>"##,
    );
    motor.register_component("tela", &c).unwrap();
    motor.navigate_to("tela");
    abre(&mut motor, "c");
    let b = motor.dialog().unwrap().buttons.clone();
    assert_eq!(b.len(), 3);
    assert_eq!(b[0].action, DIALOG_CLOSE);
    assert!(b[1].action.ends_with("so") && b[1].action != "so");
    assert!(b[2].action.ends_with("aceitar"));
    assert_eq!(b[2].role, ButtonRole::Accept);
    std::fs::remove_file(&c).ok();
}

/// `choice` e `button` não se misturam: o markup diz, com a linha, o que está errado.
#[test]
fn choice_e_button_nao_se_misturam() {
    for (nome, actions, trecho) in [
        ("sem_value", r#"<choice text="A" />"#, "choice sem `value`"),
        ("com_on_click", r#"<choice text="A" value="a" on_click="x" />"#, "choice com `on_click`"),
        ("button_com_value", r#"<button text="A" value="a" />"#, "button com `value`"),
        ("tag_estranha", r#"<text content="A" />"#, "dentro do actions"),
    ] {
        let mut motor = GlacierUI::new();
        let c = escreve(
            &format!("mistura_{nome}"),
            "xml",
            &format!(
                r##"<screen><resources><dialog name="d" title="T"><text content="c" />
                    <actions>{actions}</actions></dialog></resources>
                    <text content="tela" /></screen>"##
            ),
        );
        let erro = motor.register_component("tela", &c).expect_err(nome);
        assert!(erro.to_string().contains(trecho), "{nome}: {erro}");
        std::fs::remove_file(&c).ok();
    }
}

/// `open_dialog` suspende e devolve o `value` do `choice`; o handler continua
/// DEPOIS do retorno, com o estado que deixou antes, e lê o que o corpo gravou.
#[test]
fn open_dialog_devolve_o_valor_do_choice_e_o_handler_continua() {
    let mut motor = GlacierUI::new();
    let c = escreve(
        "open_dialog_valor",
        "xml",
        r##"<screen>
              <resources>
                <script>
                  function pedir()
                    local alvo = "svc-1"
                    ctx["__dialog.plano"] = "1. db\n2. api"
                    ctx.antes = "rodou"
                    local r = open_dialog("plano")
                    ctx.tipo = type(r)
                    ctx.lido = ctx["__dialog.nota"] or "vazio"
                    if r == "aceitar" then ctx.resultado = "aceitou:" .. alvo
                    elseif r == "so" then ctx.resultado = "so:" .. alvo
                    else ctx.resultado = "nada" end
                  end
                </script>
                <dialog name="plano" title="Plano">
                  <text content="{__dialog.plano}" />
                  <actions>
                    <button text="Rejeitar" />
                    <choice text="Só este" value="so" role="neutral" />
                    <choice text="Aceitar" value="aceitar" />
                  </actions>
                </dialog>
              </resources>
              <button text="Pedir" on_click="pedir" />
            </screen>"##,
    );
    motor.register_component("tela", &c).unwrap();
    motor.navigate_to("tela");

    let _ = motor.dispatch(&EngineMessage::UiClick("pedir".into()));
    let spec = motor.dialog().expect("open_dialog abriu o diálogo");
    assert_eq!(spec.title, "Plano");
    assert_eq!(motor.get_data("antes").map(String::as_str), Some("rodou"));
    assert_eq!(motor.get_data("resultado"), None, "suspende: ainda não escolheu");
    assert_eq!(
        motor.get_data("__dialog.plano").map(String::as_str),
        Some("1. db\n2. api"),
        "o rascunho semeado antes do open_dialog chega ao corpo"
    );

    // Aceitar (o último botão): devolve "aceitar"; o handler enxerga a variável local.
    let aceitar = spec.buttons[2].action.clone();
    motor.define_data("__dialog.nota", "digitado");
    let _ = motor.dispatch(&EngineMessage::DialogButton(aceitar));
    assert_eq!(motor.get_data("tipo").map(String::as_str), Some("string"));
    assert_eq!(motor.get_data("resultado").map(String::as_str), Some("aceitou:svc-1"));
    assert_eq!(
        motor.get_data("lido").map(String::as_str),
        Some("digitado"),
        "o handler retomado ainda lê o rascunho __dialog.*"
    );
    assert!(motor.dialog().is_none());
    assert_eq!(
        motor.get_data("__dialog.nota"),
        None,
        "e o motor o apaga DEPOIS do handler"
    );

    // Só este.
    let _ = motor.dispatch(&EngineMessage::UiClick("pedir".into()));
    let so = motor.dialog().unwrap().buttons[1].action.clone();
    let _ = motor.dispatch(&EngineMessage::DialogButton(so));
    assert_eq!(motor.get_data("resultado").map(String::as_str), Some("so:svc-1"));

    // Rejeitar (um `button` que só fecha) devolve nil.
    let _ = motor.dispatch(&EngineMessage::UiClick("pedir".into()));
    let rej = motor.dialog().unwrap().buttons[0].action.clone();
    let _ = motor.dispatch(&EngineMessage::DialogButton(rej));
    assert_eq!(motor.get_data("tipo").map(String::as_str), Some("nil"));
    assert_eq!(motor.get_data("resultado").map(String::as_str), Some("nada"));
    std::fs::remove_file(&c).ok();
}

/// Dispensar o diálogo (clicar no fundo) também devolve `nil`, e uma corrotina
/// nunca fica presa: um nome inexistente retoma na hora com `nil`.
#[test]
fn open_dialog_dispensar_e_nome_inexistente_devolvem_nil() {
    let mut motor = GlacierUI::new();
    let c = escreve(
        "open_dialog_nil",
        "xml",
        r##"<screen>
              <resources>
                <script>
                  function dispensavel() ctx.r1 = tostring(open_dialog("d")) end
                  function inexistente() ctx.r2 = tostring(open_dialog("nao_existe")) ctx.fim = "ok" end
                </script>
                <dialog name="d" title="T">
                  <text content="c" />
                  <actions><choice text="A" value="a" /></actions>
                </dialog>
              </resources>
              <button text="x" on_click="dispensavel" />
            </screen>"##,
    );
    motor.register_component("tela", &c).unwrap();
    motor.navigate_to("tela");
    let _ = motor.dispatch(&EngineMessage::UiClick("dispensavel".into()));
    assert!(motor.dialog().is_some());
    let _ = motor.dispatch(&EngineMessage::DialogDismiss);
    assert!(motor.dialog().is_none());
    assert_eq!(motor.get_data("r1").map(String::as_str), Some("nil"));

    let _ = motor.dispatch(&EngineMessage::UiClick("inexistente".into()));
    assert!(motor.dialog().is_none());
    assert_eq!(motor.get_data("r2").map(String::as_str), Some("nil"));
    assert_eq!(motor.get_data("fim").map(String::as_str), Some("ok"));
    std::fs::remove_file(&c).ok();
}

/// Regressão: o `confirm{}` tem dois botões e devolve só booleano — uma opção
/// de terceiro botão que chegou a existir foi retirada de propósito.
#[test]
fn confirm_continua_com_dois_botoes_e_so_booleano() {
    let mut motor = GlacierUI::new();
    let c = escreve(
        "confirm_bool",
        "xml",
        r##"<screen>
              <resources>
                <script>
                  function pedir()
                    local r = confirm({ title = "T", message = "M", extra_label = "ignorado" })
                    ctx.tipo = type(r)
                    ctx.valor = tostring(r)
                  end
                </script>
              </resources>
              <button text="Pedir" on_click="pedir" />
            </screen>"##,
    );
    motor.register_component("tela", &c).unwrap();
    motor.navigate_to("tela");
    let _ = motor.dispatch(&EngineMessage::UiClick("pedir".into()));
    let spec = motor.dialog().expect("confirm abriu");
    assert_eq!(spec.buttons.len(), 2, "extra_label não existe mais");
    let sim = spec.buttons[1].action.clone();
    let _ = motor.dispatch(&EngineMessage::DialogButton(sim));
    assert_eq!(motor.get_data("tipo").map(String::as_str), Some("boolean"));
    assert_eq!(motor.get_data("valor").map(String::as_str), Some("true"));

    let _ = motor.dispatch(&EngineMessage::UiClick("pedir".into()));
    let nao = motor.dialog().unwrap().buttons[0].action.clone();
    let _ = motor.dispatch(&EngineMessage::DialogButton(nao));
    assert_eq!(motor.get_data("tipo").map(String::as_str), Some("boolean"));
    assert_eq!(motor.get_data("valor").map(String::as_str), Some("false"));
    std::fs::remove_file(&c).ok();
}
