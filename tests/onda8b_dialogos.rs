//! Onda 8b — `<dialog>` e `component` compostos nos dois sentidos, e os dois
//! bugs de motor que o exemplo `onda8b_luau` destravou:
//!
//!   A. um `<button on_click="dialog:x">` dentro de um `<component>` **com
//!      `<script>`** virava `Comp::dialog:x` (namespace de dono), e aí o
//!      `strip_prefix("dialog:")` do dispatch não casava → o modal não abria.
//!   B. o rascunho `__dialog.*` era apagado **antes** de o handler do botão
//!      rodar, então um `salvar()` que lia `ctx["__dialog.nome"]` no aceite
//!      recebia string vazia (latente desde a Onda 8).
//!
//! Os dois testes olham o efeito observável — a ação avaliada na árvore, o
//! `dialog()` aberto, a chave que o handler escreveu — não uma ida-e-volta de
//! contexto (a checagem que o `CLAUDE.md` manda fazer).

use glacier_ui::{EngineMessage, GlacierUI, NodeType};

fn escreve(nome: &str, conteudo: &str) -> String {
    std::fs::create_dir_all("templates").ok();
    let caminho = format!("templates/{nome}");
    std::fs::write(&caminho, conteudo).unwrap();
    caminho
}

fn acha<'a>(node: &'a glacier_ui::UiNode, tag: &str) -> Option<&'a glacier_ui::UiNode> {
    if node.kind.tag_name() == Some(tag) {
        return Some(node);
    }
    node.children.iter().find_map(|c| acha(c, tag))
}

fn on_click_de(node: &glacier_ui::UiNode) -> Option<String> {
    match &node.kind {
        NodeType::Button { on_click, .. } => on_click.clone(),
        _ => None,
    }
}

/// Bug B: o rascunho `__dialog.*` tem de estar no contexto quando o handler do
/// botão roda, e ser apagado só **depois**.
#[test]
fn rascunho_do_dialogo_sobrevive_ate_o_handler_ler() {
    let gv = escreve(
        "onda8b_bugb.gv",
        r#"<screen>
  <resources>
    <script>
      function salvar()
        ctx.resultado = ctx["__dialog.nome"] or "VAZIO"
      end
    </script>
    <dialog name="editar" title="Editar" buttons="Cancelar::|Salvar:salvar:accept">
      <column>
        <textinput value="__dialog.nome" on_change="__dialog.nome" />
      </column>
    </dialog>
  </resources>
  <button text="abrir" on_click="dialog:editar" />
</screen>"#,
    );

    let mut motor = GlacierUI::new();
    motor.register_component("tela", &gv).unwrap();
    motor.navigate_to("tela");

    let _ = motor.dispatch(&EngineMessage::UiClick("dialog:editar".into()));
    assert!(motor.dialog().is_some(), "o <dialog> abriu");

    // O que o usuário digitou mora numa chave comum, como em qualquer campo.
    let _ = motor.dispatch(&EngineMessage::ContextPatch(vec![(
        "__dialog.nome".into(),
        "api-gateway".into(),
    )]));

    // Salvar: fecha o diálogo e roteia `salvar` como ação comum.
    let _ = motor.dispatch(&EngineMessage::DialogButton("salvar".into()));

    assert_eq!(
        motor.context().get("resultado").map(String::as_str),
        Some("api-gateway"),
        "o handler leu __dialog.nome ANTES da limpeza (bug B)"
    );
    assert_eq!(
        motor.context().get("__dialog.nome"),
        None,
        "e o rascunho foi apagado DEPOIS do aceite"
    );
}

/// Bug A: um `<dialog>` declarado dentro de um `<component>` com `<script>` —
/// ele é registrado, o `dialog:` do botão **não** é namespaceado, e o botão
/// `Salvar:Comp::salvar:accept` volta para o handler do componente, que lê o
/// rascunho.
#[test]
fn dialog_dentro_de_componente_com_script_abre_e_salva() {
    escreve(
        "onda8b_buga_comp.luau",
        "function salvar()\n  ctx.res = ctx[\"__dialog.v\"] or \"VAZIO\"\nend\n",
    );
    escreve(
        "onda8b_buga_comp.gv",
        r#"<component>
  <resources>
    <script src="onda8b_buga_comp.luau"></script>
    <dialog name="dlg_do_comp" title="X"
            buttons="Cancelar::|Salvar:CompX::salvar:accept">
      <column>
        <textinput value="__dialog.v" on_change="__dialog.v" />
      </column>
    </dialog>
  </resources>
  <button text="editar" on_click="dialog:dlg_do_comp" />
</component>"#,
    );
    let gv = escreve(
        "onda8b_buga.gv",
        r#"<screen>
  <resources>
    <import name="CompX" from="onda8b_buga_comp.gv" />
  </resources>
  <CompX />
</screen>"#,
    );

    let mut motor = GlacierUI::new();
    motor.register_component("tela", &gv).unwrap();
    motor.navigate_to("tela");

    // A ação do botão do componente, JÁ avaliada (o namespacing já rodou): ela
    // tem de ter ficado `dialog:dlg_do_comp`, não `CompX::dialog:dlg_do_comp`.
    let botao = acha(motor.evaluated("tela").unwrap(), "button").expect("o botão do componente");
    let acao = on_click_de(botao).expect("on_click");
    assert_eq!(
        acao, "dialog:dlg_do_comp",
        "o prefixo dialog: não pode ser namespaceado para o dono (bug A)"
    );

    // E ele abre o modal declarado DENTRO do componente.
    let _ = motor.dispatch(&EngineMessage::UiClick(acao));
    assert!(
        motor.dialog().is_some(),
        "o <dialog> declarado dentro do <component> foi registrado e abriu"
    );

    // Salvar: `CompX::salvar` volta para o componente (o `::` de dono), e o
    // rascunho ainda está lá.
    let _ = motor.dispatch(&EngineMessage::ContextPatch(vec![(
        "__dialog.v".into(),
        "42".into(),
    )]));
    let _ = motor.dispatch(&EngineMessage::DialogButton("CompX::salvar".into()));
    assert_eq!(
        motor.context().get("res").map(String::as_str),
        Some("42"),
        "o handler do COMPONENTE leu __dialog.v"
    );
}
