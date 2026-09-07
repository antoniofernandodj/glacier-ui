//! Onda 8 — o diálogo que carrega markup.
//!
//! Os testes desta onda cobrem o habilitador (corpo + retorno + a ação
//! `dialog:`) e os widgets que saem dele. O que eles guardam, mais do que a
//! feature, são as três armadilhas previstas no `PLANO_WIDGETS.md`: a colisão
//! por nome de chave, o corpo que precisa receber input apesar do fundo que
//! bloqueia, e o progresso que não pode morar no `DialogSpec`.

use glacier_ui::{DialogIcon, EngineMessage, GlacierUI, NodeType};

/// Escreve um `.gv` temporário e devolve o caminho.
fn escreve(nome: &str, conteudo: &str) -> String {
    std::fs::create_dir_all("templates").ok();
    let caminho = format!("templates/{nome}.gv");
    std::fs::write(&caminho, conteudo).unwrap();
    caminho
}

/// Um `<dialog name="…">` do `<resources>` registra DUAS coisas sob o mesmo
/// nome: o corpo, como template comum, e a moldura, no mapa de diálogos.
///
/// É a decisão central do habilitador A — e é o que faz `render(nome)` montar
/// o corpo sem saber que aquilo é um diálogo.
#[test]
fn dialog_declara_corpo_e_moldura() {
    let mut motor = GlacierUI::new();
    let caminho = escreve(
        "onda8_decl",
        r##"<screen>
                <resources>
                    <dialog name="editar" title="Editar perfil" icon="question"
                            buttons="Cancelar::|Salvar:salvar:accept">
                        <Column spacing="8">
                            <TextInput value="__dialog.nome" placeholder="Nome" />
                        </Column>
                    </dialog>
                </resources>
                <Button text="Editar" on_click="dialog:editar" />
            </screen>"##,
    );
    motor.register_component("tela", &caminho).unwrap();

    // O corpo virou template: o motor o avalia como qualquer outro.
    let corpo = motor.evaluated("editar").expect("corpo registrado");
    assert_eq!(corpo.kind, NodeType::Column);

    // E a moldura abre pelo nome.
    motor.navigate_to("tela");
    let _ = motor.dispatch(&EngineMessage::UiClick("dialog:editar".into()));

    let spec = motor.dialog().expect("diálogo aberto");
    assert_eq!(spec.title, "Editar perfil");
    assert_eq!(spec.icon, DialogIcon::Question);
    assert_eq!(spec.body.as_deref(), Some("editar"));
    assert_eq!(spec.buttons.len(), 2);
    assert_eq!(spec.buttons[0].label, "Cancelar");
    assert_eq!(spec.buttons[1].label, "Salvar");
    assert_eq!(spec.buttons[1].action, "salvar");

    std::fs::remove_file(&caminho).ok();
}

/// `dialog:close` fecha, e um nome que não existe é ignorado em silêncio — a
/// alternativa seria derrubar o app por um erro de digitação num `on_click`.
#[test]
fn dialog_fecha_e_ignora_nome_desconhecido() {
    let mut motor = GlacierUI::new();
    let caminho = escreve(
        "onda8_fecha",
        r##"<screen>
                <resources>
                    <dialog name="aviso" title="Oi"><Text content="olá" /></dialog>
                </resources>
                <Text content="tela" />
            </screen>"##,
    );
    motor.register_component("tela", &caminho).unwrap();
    motor.navigate_to("tela");

    let _ = motor.dispatch(&EngineMessage::UiClick("dialog:aviso".into()));
    assert!(motor.dialog().is_some());

    let _ = motor.dispatch(&EngineMessage::UiClick("dialog:close".into()));
    assert!(motor.dialog().is_none(), "dialog:close fecha");

    let _ = motor.dispatch(&EngineMessage::UiClick("dialog:nao_existe".into()));
    assert!(
        motor.dialog().is_none(),
        "um nome desconhecido não abre nada, e não entra em pânico"
    );

    std::fs::remove_file(&caminho).ok();
}

/// Sem `buttons`, o diálogo nasce com um `Fechar` que só fecha: a ação dele é
/// a sentinela, e clicar nela não roteia ação nenhuma para o componente.
///
/// O teste existe porque a alternativa (despachar uma ação "fechar" de
/// verdade) é uma bomba-relógio: o dia em que o app escrever um `update`
/// chamado `fechar`, ele passaria a disparar sozinho.
#[test]
fn botao_sem_acao_so_fecha() {
    let mut motor = GlacierUI::new();
    let caminho = escreve(
        "onda8_fechar",
        r##"<screen>
                <resources>
                    <dialog name="so_texto"><Text content="nada a fazer" /></dialog>
                </resources>
                <Text content="tela" />
            </screen>"##,
    );
    motor.register_component("tela", &caminho).unwrap();
    motor.navigate_to("tela");
    let _ = motor.dispatch(&EngineMessage::UiClick("dialog:so_texto".into()));

    let spec = motor.dialog().expect("aberto");
    assert_eq!(spec.buttons.len(), 1);
    assert_eq!(spec.buttons[0].label, "Fechar");
    assert_eq!(spec.buttons[0].action, glacier_ui::DIALOG_CLOSE);

    let acao = spec.buttons[0].action.clone();
    let _ = motor.dispatch(&EngineMessage::DialogButton(acao));
    assert!(motor.dialog().is_none());

    std::fs::remove_file(&caminho).ok();
}

/// Um `<dialog>` fora do `<resources>`, ou sem `name`, é erro POSICIONADO —
/// não silêncio. É onde o erro de escrita tem de aparecer, já que o dispatch
/// (por decisão) ignora nome desconhecido.
#[test]
fn dialog_sem_nome_e_erro_de_template() {
    let xml = r##"<screen>
            <resources>
                <dialog title="Sem nome"><Text content="x" /></dialog>
            </resources>
            <Text content="tela" />
        </screen>"##;
    let erro = glacier_ui::UiNode::parse_xml(xml).unwrap_err();
    let msg = format!("{erro}");
    assert!(
        msg.contains("sem `name`"),
        "a mensagem deve dizer o que falta, e não só que falhou: {msg}"
    );
}

/// O ciclo inteiro de um `prompt{}`: o motor semeia o rascunho, o corpo é o
/// builtin, o usuário edita a chave, o aceite lê de lá e a corrotina recebe a
/// string.
///
/// Guarda o habilitador B (o retorno deixou de ser `bool`) e, junto com ele, a
/// reclassificação: o `InputDialog` não tem estado por instância nenhum — o
/// valor esteve numa chave de contexto o tempo todo.
#[test]
fn prompt_devolve_o_valor_da_chave() {
    let mut motor = GlacierUI::new();
    let caminho = escreve(
        "onda8_prompt",
        r##"<screen>
                <resources>
                    <script>
                        function pedir()
                            local nome = prompt({ title = "Renomear", label = "Novo nome",
                                                  value = "antes" })
                            ctx.resposta = nome or "CANCELOU"
                        end
                    </script>
                </resources>
                <Button text="Pedir" on_click="pedir" />
            </screen>"##,
    );
    motor.register_component("tela", &caminho).unwrap();
    motor.navigate_to("tela");

    let _ = motor.dispatch(&EngineMessage::UiClick("pedir".into()));

    // O diálogo abriu com corpo, e o rascunho já está semeado — o valor
    // inicial precisa estar lá no PRIMEIRO quadro, senão o campo pisca vazio.
    let spec = motor.dialog().expect("prompt abriu um diálogo");
    assert_eq!(spec.body.as_deref(), Some("__InputDialog"));
    assert_eq!(
        motor.get_data("__dialog.value").map(String::as_str),
        Some("antes")
    );
    assert_eq!(
        motor.get_data("__dialog.label").map(String::as_str),
        Some("Novo nome")
    );
    assert_eq!(
        motor.get_data("__dialog.kind").map(String::as_str),
        Some("text")
    );

    let aceitar = spec.buttons.last().unwrap().action.clone();

    // O usuário digita: é a chave que muda, como em qualquer <textinput>.
    motor.define_data("__dialog.value", "depois");

    let _ = motor.dispatch(&EngineMessage::DialogButton(aceitar));

    assert_eq!(
        motor.get_data("resposta").map(String::as_str),
        Some("depois"),
        "a corrotina recebe a string, não um booleano"
    );
    assert!(motor.dialog().is_none());
    assert_eq!(
        motor.get_data("__dialog.value").map(String::as_str),
        None,
        "o rascunho é apagado no fechamento; sem isso a segunda abertura viria \
         preenchida com a resposta da primeira"
    );

    std::fs::remove_file(&caminho).ok();
}

/// Cancelar um `prompt{}` devolve `nil`, não `false` — a convenção que o
/// `open_file()` cancelado já usava, e o que separa "não respondeu" de
/// "respondeu vazio".
#[test]
fn prompt_cancelado_devolve_nil() {
    let mut motor = GlacierUI::new();
    let caminho = escreve(
        "onda8_prompt_cancela",
        r##"<screen>
                <resources>
                    <script>
                        function pedir()
                            local v = prompt({ title = "Nome" })
                            ctx.tipo = type(v)
                        end
                    </script>
                </resources>
                <Button text="Pedir" on_click="pedir" />
            </screen>"##,
    );
    motor.register_component("tela", &caminho).unwrap();
    motor.navigate_to("tela");
    let _ = motor.dispatch(&EngineMessage::UiClick("pedir".into()));

    let cancelar = motor.dialog().unwrap().buttons[0].action.clone();
    let _ = motor.dispatch(&EngineMessage::DialogButton(cancelar));

    assert_eq!(motor.get_data("tipo").map(String::as_str), Some("nil"));

    std::fs::remove_file(&caminho).ok();
}

/// `confirm{}` não mudou: continua sem corpo e continua devolvendo booleano.
/// É o teste de regressão do habilitador B — generalizar o retorno não podia
/// custar a forma que já existia.
#[test]
fn confirm_continua_booleano_e_sem_corpo() {
    let mut motor = GlacierUI::new();
    let caminho = escreve(
        "onda8_confirm",
        r##"<screen>
                <resources>
                    <script>
                        function perguntar()
                            local ok = confirm({ title = "Remover?", message = "sem volta" })
                            ctx.tipo = type(ok)
                            ctx.valor = tostring(ok)
                        end
                    </script>
                </resources>
                <Button text="Perguntar" on_click="perguntar" />
            </screen>"##,
    );
    motor.register_component("tela", &caminho).unwrap();
    motor.navigate_to("tela");
    let _ = motor.dispatch(&EngineMessage::UiClick("perguntar".into()));

    let spec = motor.dialog().expect("confirm abriu");
    assert_eq!(spec.body, None, "confirm continua sem corpo");

    let aceitar = spec.buttons.last().unwrap().action.clone();
    let _ = motor.dispatch(&EngineMessage::DialogButton(aceitar));

    assert_eq!(motor.get_data("tipo").map(String::as_str), Some("boolean"));
    assert_eq!(motor.get_data("valor").map(String::as_str), Some("true"));

    std::fs::remove_file(&caminho).ok();
}

/// O `ProgressDialog` não suspende a corrotina, e o progresso mora numa chave
/// — as duas coisas que o separam do resto da família.
///
/// A segunda é a armadilha 3 do plano: com o número no `DialogSpec`, cada
/// tique reconstruiria a especificação e o cartão (e `show_dialog`
/// *substitui* o diálogo, então cada 1% seria um diálogo novo). Com ele numa
/// chave, o `spec` é o mesmo objeto do começo ao fim.
#[test]
fn progresso_nao_suspende_e_mora_numa_chave() {
    let mut motor = GlacierUI::new();
    let caminho = escreve(
        "onda8_progresso",
        r##"<screen>
                <resources>
                    <script>
                        function baixar()
                            progress({ title = "Baixando", max = 3, on_cancel = "abortar" })
                            for i = 1, 3 do
                                progress_set(i, "pacote " .. i)
                            end
                            ctx.terminou = "true"
                        end
                        function abortar()
                            ctx.abortado = "true"
                        end
                    </script>
                </resources>
                <Button text="Baixar" on_click="baixar" />
            </screen>"##,
    );
    motor.register_component("tela", &caminho).unwrap();
    motor.navigate_to("tela");

    let _ = motor.dispatch(&EngineMessage::UiClick("baixar".into()));

    // O laço inteiro rodou: o `progress` não suspendeu nada.
    assert_eq!(
        motor.get_data("terminou").map(String::as_str),
        Some("true"),
        "progress() não pode suspender — ele acompanha um trabalho em curso"
    );
    let spec = motor.dialog().expect("o diálogo continua aberto");
    assert_eq!(spec.body.as_deref(), Some("__ProgressDialog"));
    assert!(
        !spec.dismissible,
        "não se dispensa um progresso clicando fora"
    );
    assert_eq!(
        motor.get_data("__dialog.progresso").map(String::as_str),
        Some("3"),
        "o número anda pela chave, não pelo DialogSpec"
    );
    // E a chave que a BARRA lê é a escala 0–100, porque o `min`/`max` de um
    // `<progressbar>` é literal e não aceita interpolação.
    assert_eq!(
        motor.get_data("__dialog.pct").map(String::as_str),
        Some("100"),
        "3 de 3 é 100%"
    );
    assert_eq!(
        motor.get_data("__dialog.label").map(String::as_str),
        Some("pacote 3")
    );

    // O cancelamento é uma AÇÃO comum, não um mecanismo novo.
    let cancelar = spec.buttons[0].action.clone();
    assert_eq!(cancelar, "abortar");
    let _ = motor.dispatch(&EngineMessage::DialogButton(cancelar));
    assert_eq!(motor.get_data("abortado").map(String::as_str), Some("true"));
    assert!(motor.dialog().is_none());

    std::fs::remove_file(&caminho).ok();
}

/// Sem `on_cancel`, o progresso não ganha botão nenhum — um "Cancelar" que não
/// cancela é pior do que botão nenhum.
#[test]
fn progresso_sem_on_cancel_nao_tem_botao() {
    let mut motor = GlacierUI::new();
    let caminho = escreve(
        "onda8_progresso_sem_botao",
        r##"<screen>
                <resources>
                    <script>
                        function ir() progress({ title = "Gravando" }) end
                    </script>
                </resources>
                <Button text="Ir" on_click="ir" />
            </screen>"##,
    );
    motor.register_component("tela", &caminho).unwrap();
    motor.navigate_to("tela");
    let _ = motor.dispatch(&EngineMessage::UiClick("ir".into()));

    let spec = motor.dialog().expect("aberto");
    assert!(spec.buttons.is_empty());
    assert!(
        matches!(
            motor.get_data("__dialog.progresso").map(String::as_str),
            None | Some("")
        ),
        "sem value inicial, o progresso nasce indeterminado (o corpo desenha o spinner)"
    );

    std::fs::remove_file(&caminho).ok();
}

/// `progress_set(nil)` tem de VOLTAR ao indeterminado. Se a escrita vazia
/// sumisse pelo caminho, a chave manteria o último número e a barra ficaria
/// congelada em 40% para sempre — um bug silencioso, do tipo que só aparece
/// numa etapa que volta a "trabalhando…".
#[test]
fn progresso_volta_ao_indeterminado() {
    let mut motor = GlacierUI::new();
    let caminho = escreve(
        "onda8_progresso_nil",
        r##"<screen>
                <resources>
                    <script>
                        function ir()
                            progress({ title = "Etapas", max = 10 })
                            progress_set(4)
                            progress_set(nil, "reindexando…")
                        end
                    </script>
                </resources>
                <Button text="Ir" on_click="ir" />
            </screen>"##,
    );
    motor.register_component("tela", &caminho).unwrap();
    motor.navigate_to("tela");
    let _ = motor.dispatch(&EngineMessage::UiClick("ir".into()));

    for chave in ["__dialog.progresso", "__dialog.pct"] {
        let v = motor.get_data(chave).map(String::as_str);
        assert!(
            matches!(v, None | Some("")),
            "progress_set(nil) volta ao indeterminado, mas {chave} ficou em {v:?}"
        );
    }
    assert_eq!(
        motor.get_data("__dialog.label").map(String::as_str),
        Some("reindexando…")
    );

    std::fs::remove_file(&caminho).ok();
}

/// As quatro regras do `<wizard>`, testadas onde elas moram: a aritmética.
///
/// O `Plano` existe separado do render exatamente para isto — as regras de um
/// wizard são conta, e conta se testa sem montar `Element` nenhum.
#[test]
fn wizard_as_quatro_regras() {
    use glacier_ui::wizard::Plano;
    let passos = ["dados", "pagamento", "revisao"];

    // 1. A chave vazia é o passo zero: a tag funciona sem o app semear nada.
    let p = Plano::novo(&passos, "", "");
    assert_eq!(p.i, 0);
    assert!(p.primeiro, "Voltar fica inerte no primeiro");
    assert!(!p.ultimo);

    // 2. Um id que não está na lista (renomeado, com erro de digitação)
    //    degrada para o começo, não para uma tela vazia.
    assert_eq!(Plano::novo(&passos, "inexistente", "").i, 0);

    // 3. Saturar, nunca dar a volta — nas duas pontas.
    assert_eq!(Plano::novo(&passos, "dados", "").anterior(), 0);
    let ultimo = Plano::novo(&passos, "revisao", "");
    assert!(ultimo.ultimo, "no fim, Avançar vira Finalizar");
    assert_eq!(
        ultimo.proximo(),
        2,
        "avançar no último passo não pode voltar ao primeiro: pareceria ter \
         perdido o que o usuário preencheu"
    );

    // 4. `valid` é o QWizardPage::isComplete(): ausente = sempre válido.
    assert!(Plano::novo(&passos, "dados", "").pode_avancar);
    assert!(Plano::novo(&passos, "dados", "true").pode_avancar);
    assert!(!Plano::novo(&passos, "dados", "false").pode_avancar);
    assert!(!Plano::novo(&passos, "dados", "nao").pode_avancar);

    // Um wizard de um passo só é primeiro E último ao mesmo tempo.
    let unico = Plano::novo(&["so"], "so", "");
    assert!(unico.primeiro && unico.ultimo);
}

/// O `<wizard>` e o `<stackview>` são tags de verdade: o motor as resolve, e a
/// página ativa é a que o `active` escolhe.
#[test]
fn wizard_e_stackview_montam() {
    let mut motor = GlacierUI::new();
    let caminho = escreve(
        "onda8_wizard_tags",
        r##"<screen>
                <wizard value="passo" active="{passo}"
                        steps="dados,revisao" titles="Dados,Revisão"
                        on_finish="salvar">
                    <template slot="dados"><Text content="PAGINA_DADOS" /></template>
                    <template slot="revisao"><Text content="PAGINA_REVISAO" /></template>
                </wizard>
            </screen>"##,
    );
    motor.register_component("tela", &caminho).unwrap();
    motor.define_data("passo", "revisao");
    motor.navigate_to("tela");

    let arvore = format!("{:?}", motor.evaluated("tela").expect("tela avaliada"));
    assert!(
        arvore.contains("PAGINA_REVISAO"),
        "a página do passo ativo tem de estar montada"
    );
    assert!(
        arvore.contains("WizardNav"),
        "o wizard compõe a navegação primitiva"
    );

    std::fs::remove_file(&caminho).ok();
}

/// A conversão HSV↔RGB é a única conta do `<colorwheel>`, e ela tem duas
/// armadilhas que valem um teste: o cinza sem matiz e a ida-e-volta.
#[test]
fn cor_converte_nos_dois_sentidos() {
    use glacier_ui::color_picker::{hsv_para_rgb, para_hex, rgb_para_hsv};

    // Os seis vértices do cubo, que são onde o algoritmo troca de setor.
    for (h, hex) in [
        (0.0, "#ff0000"),
        (60.0, "#ffff00"),
        (120.0, "#00ff00"),
        (180.0, "#00ffff"),
        (240.0, "#0000ff"),
        (300.0, "#ff00ff"),
    ] {
        assert_eq!(para_hex(hsv_para_rgb(h, 1.0, 1.0)), hex, "matiz {h}");
    }

    // Ida e volta em cores arbitrárias: o hex tem de sobreviver.
    for hex in ["#336699", "#ff8800", "#123456", "#ffffff", "#000000"] {
        let c = glacier_ui::color_picker::hex_para_cor(hex).unwrap();
        let (h, s, v) = rgb_para_hsv(c);
        assert_eq!(para_hex(hsv_para_rgb(h, s, v)), hex, "ida e volta de {hex}");
    }

    // Um cinza não tem matiz definido — a função devolve 0, e é por isso que
    // o widget guarda o matiz anterior numa chave irmã: sem ela, arrastar o
    // valor até o preto e voltar devolveria vermelho, não a cor escolhida.
    let (h, s, _) = rgb_para_hsv(glacier_ui::color_picker::hex_para_cor("#808080").unwrap());
    assert_eq!(s, 0.0);
    assert_eq!(h, 0.0);
}

/// `pick_color{}` é um `prompt{}` cujo campo é uma roda: mesma porta, mesmo
/// retorno, mesma convenção de desistência.
#[test]
fn pick_color_e_um_prompt_com_roda() {
    let mut motor = GlacierUI::new();
    let caminho = escreve(
        "onda8_cor",
        r##"<screen>
                <resources>
                    <script>
                        function escolher()
                            local c = pick_color({ title = "Cor", value = "#336699" })
                            ctx.escolhida = c or "CANCELOU"
                        end
                    </script>
                </resources>
                <Button text="Cor" on_click="escolher" />
            </screen>"##,
    );
    motor.register_component("tela", &caminho).unwrap();
    motor.navigate_to("tela");
    let _ = motor.dispatch(&EngineMessage::UiClick("escolher".into()));

    let spec = motor.dialog().expect("abriu");
    assert_eq!(spec.body.as_deref(), Some("__ColorDialog"));
    assert_eq!(
        motor.get_data("__dialog.value").map(String::as_str),
        Some("#336699"),
        "a cor inicial é semeada na mesma chave que a roda escreve"
    );

    let aceitar = spec.buttons.last().unwrap().action.clone();
    motor.define_data("__dialog.value", "#ff8800");
    let _ = motor.dispatch(&EngineMessage::DialogButton(aceitar));

    assert_eq!(
        motor.get_data("escolhida").map(String::as_str),
        Some("#ff8800")
    );

    std::fs::remove_file(&caminho).ok();
}

/// As telas dos dois exemplos da onda avaliam de verdade — o que o
/// `exemplos_gv` não cobre, porque ele só parseia. Uma tag escrita errada
/// (`<colorwheel>` com nome trocado, um `slot` que não casa) passa no parse e
/// só aparece aqui.
#[test]
fn exemplos_da_onda_avaliam() {
    for (nome, caminho) in [
        ("onda8", "examples/onda8/app.gv"),
        ("onda8_luau", "examples/onda8_luau/app.gv"),
    ] {
        let mut motor = GlacierUI::new();
        motor
            .register_component(nome, caminho)
            .unwrap_or_else(|e| panic!("{caminho} não registra: {e}"));
        motor.navigate_to(nome);
        motor
            .evaluated(nome)
            .unwrap_or_else(|e| panic!("{caminho} não avalia: {e}"));

        // O `<dialog>` do exemplo Rust tem de ter registrado o corpo dele como
        // um template — é o que faz `dialog:editar_servico` ter o que montar.
        if nome == "onda8" {
            assert!(
                motor.evaluated("editar_servico").is_ok(),
                "o <dialog> do exemplo precisa registrar o corpo sob o nome dele"
            );
        }
    }
}

/// Digitar de verdade no campo de um `prompt{}`.
///
/// O teste anterior (`prompt_devolve_o_valor_da_chave`) escrevia a chave com
/// `define_data`, que é o que o campo *deveria* fazer — e por isso não pegava
/// o buraco: um `<TextInput>` **nunca grava a chave sozinho**; ele despacha
/// `UiInputChanged` com a ação declarada, e sem `onChange` a ação é vazia.
#[test]
fn digitar_no_prompt_escreve_a_chave() {
    let mut motor = GlacierUI::new();
    let caminho = escreve(
        "onda8_digita",
        r##"<screen>
                <resources>
                    <script>
                        function pedir()
                            local n = prompt({ title = "Nome", value = "antes" })
                            ctx.resposta = n or "CANCELOU"
                        end
                    </script>
                </resources>
                <Button text="Pedir" on_click="pedir" />
            </screen>"##,
    );
    motor.register_component("tela", &caminho).unwrap();
    motor.navigate_to("tela");
    let _ = motor.dispatch(&EngineMessage::UiClick("pedir".into()));

    // A ação que o campo do corpo realmente despacha. Achá-la na árvore
    // avaliada é o ponto do teste: se ela for vazia, o campo é decorativo.
    let corpo = motor.evaluated("__InputDialog").expect("corpo montado");
    let acao = acao_do_primeiro_textinput(corpo).expect("o corpo tem um <textinput>");
    assert!(
        !acao.is_empty(),
        "um <TextInput> sem onChange despacha ação vazia e não grava nada"
    );

    let _ = motor.dispatch(&EngineMessage::UiInputChanged {
        action: acao,
        value: "depois".into(),
    });
    assert_eq!(
        motor.get_data("__dialog.value").map(String::as_str),
        Some("depois"),
        "digitar tem de chegar na chave que o aceite lê"
    );

    let aceitar = motor
        .dialog()
        .unwrap()
        .buttons
        .last()
        .unwrap()
        .action
        .clone();
    let _ = motor.dispatch(&EngineMessage::DialogButton(aceitar));
    assert_eq!(
        motor.get_data("resposta").map(String::as_str),
        Some("depois")
    );

    std::fs::remove_file(&caminho).ok();
}

/// A ação de `onChange` do primeiro `<TextInput>` de uma árvore avaliada.
fn acao_do_primeiro_textinput(no: &glacier_ui::UiNode) -> Option<String> {
    if let NodeType::TextInput { on_change, .. } = &no.kind {
        return Some(on_change.clone());
    }
    no.children.iter().find_map(acao_do_primeiro_textinput)
}

/// O torto que a onda deixou e esta correção fecha: digitar no campo
/// hexadecimal não pode fazer a roda piscar.
///
/// Enquanto o texto não é uma cor inteira, ele fica no rascunho e a cor
/// cometida não se mexe. É o que a roda lê — então ela continua na última cor
/// válida em vez de cair para branco a cada tecla.
#[test]
fn hex_parcial_nao_mexe_na_cor() {
    let mut motor = GlacierUI::new();
    let caminho = escreve(
        "onda8_hex",
        r##"<screen>
                <resources>
                    <script>
                        function escolher()
                            ctx.escolhida = pick_color({ title = "Cor", value = "#336699" }) or "X"
                        end
                    </script>
                </resources>
                <Button text="Cor" on_click="escolher" />
            </screen>"##,
    );
    motor.register_component("tela", &caminho).unwrap();
    motor.navigate_to("tela");
    let _ = motor.dispatch(&EngineMessage::UiClick("escolher".into()));

    // As duas chaves nascem iguais — senão o campo apareceria vazio.
    assert_eq!(
        motor.get_data("__dialog.value__hex").map(String::as_str),
        Some("#336699")
    );

    // Digitando `#ff8800` uma tecla por vez. Só a última é uma cor.
    for parcial in ["", "#", "#f", "#ff", "#ff8", "#ff88", "#ff880"] {
        let _ = motor.dispatch(&EngineMessage::UiInputChanged {
            action: "__ColorDialog::hex".into(),
            value: parcial.into(),
        });
        assert_eq!(
            motor.get_data("__dialog.value").map(String::as_str),
            Some("#336699"),
            "{parcial:?} não é uma cor: a roda tem de continuar em #336699"
        );
        assert_eq!(
            motor.get_data("__dialog.value__hex").map(String::as_str),
            Some(parcial),
            "o rascunho mostra o que foi digitado, inteiro e sem correção"
        );
    }

    let _ = motor.dispatch(&EngineMessage::UiInputChanged {
        action: "__ColorDialog::hex".into(),
        value: "#ff8800".into(),
    });
    assert_eq!(
        motor.get_data("__dialog.value").map(String::as_str),
        Some("#ff8800"),
        "completou: agora sim a cor é cometida"
    );

    // A caixa alta comete, normalizada — uma chave que ora tem `#FF00AA` ora
    // `#ff00aa` faz duas comparações de igualdade discordarem.
    let _ = motor.dispatch(&EngineMessage::UiInputChanged {
        action: "__ColorDialog::hex".into(),
        value: "FF00AA".into(),
    });
    assert_eq!(
        motor.get_data("__dialog.value").map(String::as_str),
        Some("#ff00aa")
    );

    // Aceitar com o campo num estado inválido devolve a última cor VÁLIDA, não
    // o texto quebrado — quem chamou `pick_color{}` não precisa validar.
    let _ = motor.dispatch(&EngineMessage::UiInputChanged {
        action: "__ColorDialog::hex".into(),
        value: "#zz".into(),
    });
    let aceitar = motor
        .dialog()
        .unwrap()
        .buttons
        .last()
        .unwrap()
        .action
        .clone();
    let _ = motor.dispatch(&EngineMessage::DialogButton(aceitar));
    assert_eq!(
        motor.get_data("escolhida").map(String::as_str),
        Some("#ff00aa")
    );

    std::fs::remove_file(&caminho).ok();
}

/// `hex_completo` aceita o que um humano digita e recusa o que está pela
/// metade — é a função inteira do conserto.
#[test]
fn hex_completo_recusa_o_que_esta_pela_metade() {
    use glacier_ui::color_picker::hex_completo;

    assert_eq!(hex_completo("#ff8800").as_deref(), Some("#ff8800"));
    assert_eq!(
        hex_completo("ff8800").as_deref(),
        Some("#ff8800"),
        "# opcional"
    );
    assert_eq!(
        hex_completo("#FF8800").as_deref(),
        Some("#ff8800"),
        "normaliza"
    );
    assert_eq!(
        hex_completo("  #ff8800  ").as_deref(),
        Some("#ff8800"),
        "apara"
    );

    for pela_metade in ["", "#", "#f", "#ff", "#ff8", "#ff88", "#ff880", "#ff88001"] {
        assert_eq!(hex_completo(pela_metade), None, "{pela_metade:?}");
    }
    // A forma curta fica DE FORA de propósito: no caminho de `#ff8800` o texto
    // passa por `#ff8`, e lê-la como cor cometeria um amarelo que ninguém
    // pediu — o mesmo defeito que esta função conserta, de outra cor.
    assert_eq!(hex_completo("#f0a"), None, "forma curta não comete");
    for lixo in ["#zzzzzz", "azul", "#ff 800"] {
        assert_eq!(hex_completo(lixo), None, "{lixo:?}");
    }
}

/// **O teste que faltava.** Todos os outros verificam o `DialogSpec` e as
/// chaves; nenhum verificava que o corpo **chega à tela**.
///
/// Ele não chegava. O motor só mantém avaliada a **tela ativa** (uma economia
/// deliberada do `reevaluate_all`), e o corpo de um diálogo não é a tela ativa:
/// `render` falhava com "registrado mas não avaliado", o `render_current`
/// engolia o erro — de propósito, para um `<dialog>` com nome errado não
/// derrubar a tela — e o cartão aparecia com título, botões e um buraco no
/// lugar do campo.
///
/// O sintoma era mudo, e é por isso que este teste cobra o `render` do corpo em
/// vez de só a existência do `spec.body`.
#[test]
fn o_corpo_do_dialogo_chega_a_tela() {
    // Uma tela de verdade, com mais de um template registrado — é a condição
    // em que a economia do motor descarta o que não está em uso.
    for (acao, corpo) in [
        ("pedir_texto", "__InputDialog"),
        ("escolher_cor", "__ColorDialog"),
    ] {
        let mut motor = GlacierUI::new();
        motor
            .register_component("onda8_luau", "examples/onda8_luau/app.gv")
            .unwrap();
        motor.navigate_to("onda8_luau");
        let _ = motor.dispatch(&EngineMessage::UiClick(acao.into()));

        let spec = motor.dialog().unwrap_or_else(|| panic!("{acao} não abriu"));
        assert_eq!(spec.body.as_deref(), Some(corpo));
        if let Err(e) = motor.render(corpo) {
            panic!("{acao}: o corpo não chega à tela — {e}");
        }
        assert!(motor.render_current().is_ok());

        // E o corpo tem de ter CONTEÚDO. Um `render` que devolve `Ok` prova só
        // que o template foi avaliado — se a escada de `se`/`senão` não casasse
        // com nada, o cartão apareceria com um container vazio no lugar do
        // campo, que é exatamente o sintoma que este teste existe para pegar.
        let arvore = motor.evaluated(corpo).expect("avaliado").clone();
        assert!(
            conta_folhas(&arvore) > 0,
            "{acao}: o corpo avaliou vazio — o cartão sairia com um buraco"
        );
    }
}

/// O mesmo, pelo caminho declarativo: um `<dialog name="…">` do `.gv` também
/// não é a tela ativa, e também precisava ser fixado.
#[test]
fn o_corpo_declarativo_chega_a_tela() {
    let mut motor = GlacierUI::new();
    motor
        .register_component("onda8", "examples/onda8/app.gv")
        .unwrap();
    motor.navigate_to("onda8");

    let _ = motor.dispatch(&EngineMessage::UiClick("dialog:editar_servico".into()));
    let spec = motor.dialog().expect("abriu");
    assert_eq!(spec.body.as_deref(), Some("editar_servico"));
    if let Err(e) = motor.render("editar_servico") {
        panic!("o corpo declarativo não chega à tela — {e}");
    }

    // Um `<dialog … />` auto-fechado é uma caixa de MENSAGEM, não um QDialog:
    // ele não declara corpo, e por isso o cartão não ganha um container vazio
    // no meio nem o motor mantém avaliado um template sem conteúdo.
    let _ = motor.dispatch(&EngineMessage::UiClick("dialog:remover_servico".into()));
    assert_eq!(
        motor.dialog().and_then(|d| d.body.as_deref()),
        None,
        "um <dialog> sem conteúdo não pode reivindicar um corpo"
    );
    assert!(
        motor.render_current().is_ok(),
        "e o encadeamento (um botão de diálogo abrindo outro) continua montando"
    );
}

/// Fechado o diálogo, o corpo é **solto**: sem isso, todo corpo já exibido
/// seguiria sendo reavaliado a cada quadro pelo resto da vida do processo.
#[test]
fn o_corpo_e_solto_no_fechamento() {
    let mut motor = GlacierUI::new();
    motor
        .register_component("onda8_luau", "examples/onda8_luau/app.gv")
        .unwrap();
    motor.navigate_to("onda8_luau");

    let _ = motor.dispatch(&EngineMessage::UiClick("pedir_texto".into()));
    assert!(motor.render("__InputDialog").is_ok());

    let cancelar = motor.dialog().unwrap().buttons[0].action.clone();
    let _ = motor.dispatch(&EngineMessage::DialogButton(cancelar));

    // Solto: volta a ser um template como outro qualquer, avaliado sob demanda
    // (`evaluated`) mas fora do trabalho de todo quadro.
    assert!(motor.dialog().is_none());
    assert!(
        motor.render("__InputDialog").is_err(),
        "o corpo continuou fixado depois de o diálogo fechar"
    );
}

/// Um laço **síncrono** não mostra progresso: o `update` inteiro roda num turno
/// só, a UI não pinta um quadro no meio dele, e o diálogo abre e fecha entre
/// dois quadros — o usuário não vê nada.
///
/// O exemplo cedia a vez errado e este teste guarda a correção: o progresso
/// tem de **continuar aberto** quando o `update` que o iniciou retorna.
#[test]
fn progresso_sobrevive_ao_turno_que_o_abriu() {
    let mut motor = GlacierUI::new();
    motor
        .register_component("onda8_luau", "examples/onda8_luau/app.gv")
        .unwrap();
    motor.navigate_to("onda8_luau");

    let _ = motor.dispatch(&EngineMessage::UiClick("baixar".into()));

    let spec = motor
        .dialog()
        .expect("o progresso tem de continuar aberto depois do turno");
    assert_eq!(spec.body.as_deref(), Some("__ProgressDialog"));
    assert!(
        motor.render("__ProgressDialog").is_ok(),
        "e o corpo dele tem de chegar à tela"
    );
    assert_eq!(
        motor.get_data("__dialog.progresso").map(String::as_str),
        Some("1"),
        "o primeiro passo já andou; os outros vêm por `after`"
    );
}

/// Quantos nós **desenháveis** a árvore tem — texto, campo, botão, o que for.
/// Containers e fragmentos não contam: uma coluna vazia é o buraco que estes
/// testes procuram, não conteúdo.
fn conta_folhas(no: &glacier_ui::UiNode) -> usize {
    let proprio = usize::from(!matches!(
        no.kind,
        NodeType::Container
            | NodeType::Column
            | NodeType::Row
            | NodeType::Fragment
            | NodeType::If { .. }
            | NodeType::Else
            | NodeType::ElseIf { .. }
    ));
    proprio + no.children.iter().map(conta_folhas).sum::<usize>()
}

/// O mesmo para o progresso: o corpo dele tem de ter a barra, não um container
/// vazio. O `<progressbar>` só aparece quando `__dialog.progresso` está
/// preenchida — o ramo vazio desenha o `<spinner>`, e os dois contam.
#[test]
fn o_corpo_do_progresso_tem_conteudo() {
    let mut motor = GlacierUI::new();
    motor
        .register_component("onda8_luau", "examples/onda8_luau/app.gv")
        .unwrap();
    motor.navigate_to("onda8_luau");
    let _ = motor.dispatch(&EngineMessage::UiClick("baixar".into()));

    let arvore = motor
        .evaluated("__ProgressDialog")
        .expect("avaliado")
        .clone();
    assert!(
        conta_folhas(&arvore) > 0,
        "o corpo do progresso avaliou vazio"
    );
    let tem_barra = procura(&arvore, |n| matches!(n.kind, NodeType::ProgressBar { .. }));
    assert!(
        tem_barra,
        "com `__dialog.progresso` preenchida o corpo tem de trazer a barra"
    );
}

/// Anda a árvore procurando um nó que satisfaça o predicado.
fn procura(no: &glacier_ui::UiNode, f: impl Fn(&glacier_ui::UiNode) -> bool + Copy) -> bool {
    f(no) || no.children.iter().any(|c| procura(c, f))
}

/// As quatro variantes do `QInputDialog` desenham quatro campos diferentes.
///
/// É onde dois erros de condicional se escondiam: `one_of="int,double"` separa
/// por ESPAÇO, então era um token só e nunca casava — os dois numéricos caíam
/// no `<senão>` e viravam campo de texto, sem erro nenhum.
#[test]
fn as_quatro_variantes_do_prompt_desenham_campos_diferentes() {
    for (kind, esperado) in [
        ("text", "TextInput"),
        ("int", "SpinBox"),
        ("double", "SpinBox"),
        ("item", "Select"),
    ] {
        let mut motor = GlacierUI::new();
        let caminho = escreve(
            &format!("onda8_kind_{kind}"),
            &format!(
                r##"<screen>
                    <resources>
                        <script>
                            function pedir()
                                prompt({{ title = "T", kind = "{kind}", value = "1",
                                          items = {{ {{ label = "A", value = "a" }} }} }})
                            end
                        </script>
                    </resources>
                    <Button text="P" on_click="pedir" />
                </screen>"##
            ),
        );
        motor.register_component("tela", &caminho).unwrap();
        motor.navigate_to("tela");
        let _ = motor.dispatch(&EngineMessage::UiClick("pedir".into()));

        let arvore = motor.evaluated("__InputDialog").expect("avaliado").clone();
        let achou = match esperado {
            "TextInput" => procura(&arvore, |n| matches!(n.kind, NodeType::TextInput { .. })),
            "Select" => procura(&arvore, |n| matches!(n.kind, NodeType::Select { .. })),
            // O `<SpinBox>` é builtin: no template avaliado ele já foi inlinado,
            // e o que sobra é o campo mais os dois degraus. O que o separa do
            // ramo de texto é justamente ter BOTÕES ao lado do campo.
            _ => procura(&arvore, |n| matches!(n.kind, NodeType::Button { .. })),
        };
        assert!(achou, "kind={kind} não desenhou um {esperado}");

        std::fs::remove_file(&caminho).ok();
    }
}
