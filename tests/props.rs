//! O `<props>` em uso: o que ele passa a pegar, e o que ele deliberadamente
//! não muda.
//!
//! O ponto da feature é o typo no ponto de USO. As props entram como uma camada
//! sobre o contexto de quem usa, e um lookup que falha na camada cai para baixo
//! — então, sem contrato, `<Cartao nomee="Alice" />` renderiza o `nome` que
//! existir no contexto global em vez de falhar. É esse silêncio que estes
//! testes fecham.

use glacier_ui::GlacierUI;

/// Cada teste escreve nos SEUS arquivos: a suíte roda em paralelo, e dois
/// testes gravando o mesmo caminho fazem um ler o arquivo pela metade — o que
/// aparece como um "template sem cabeçalho" que não existe no fonte.
fn escreve(caso: &str, nome: &str, markup: &str) -> String {
    let dir = std::env::temp_dir().join("glacier_props_tests").join(caso);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(nome);
    std::fs::write(&path, markup).unwrap();
    path.to_string_lossy().to_string()
}

/// Um componente com contrato, e uma tela que o usa com `props` no lugar de
/// `{…}` para o teste poder variar só o ponto de uso.
fn motor_com(caso: &str, uso: &str) -> (GlacierUI, String) {
    let cartao = escreve(
        caso,
        "cartao.gv",
        r##"<component>
            <props>
                <prop name="nome" />
                <prop name="cor" default="#89B4FA" />
            </props>
            <Text content="{nome} / {cor}" />
        </component>"##,
    );
    let tela = escreve(
        caso,
        "tela.gv",
        &format!(
            r##"<screen title="T">
                <resources>
                    <link rel="import" href="{cartao}" as="Cartao" />
                </resources>
                <Column>{uso}</Column>
            </screen>"##
        ),
    );
    let mut motor = GlacierUI::new();
    motor.register_component("tela", &tela).unwrap();
    motor.set_initial_screen("tela");
    (motor, tela)
}

#[test]
fn prop_desconhecida_erra_citando_as_declaradas() {
    const CASO: &str = "prop_desconhecida_erra_citando_as_declaradas";
    let (mut motor, _) = motor_com(CASO, r##"<Cartao nomee="Alice" />"##);
    let err = motor
        .reevaluate_all()
        .expect_err("nomee não está no contrato");
    let msg = err.to_string();
    assert!(msg.contains("não aceita a prop 'nomee'"), "{msg}");
    assert!(
        msg.contains("nome"),
        "a mensagem lista as que existem: {msg}"
    );
    assert!(msg.contains("cor"), "{msg}");
}

#[test]
fn prop_sem_default_e_obrigatoria() {
    const CASO: &str = "prop_sem_default_e_obrigatoria";
    let (mut motor, _) = motor_com(CASO, r##"<Cartao cor="#F00" />"##);
    let msg = motor
        .reevaluate_all()
        .expect_err("falta a prop obrigatória")
        .to_string();
    assert!(msg.contains("precisa da prop 'nome'"), "{msg}");
}

#[test]
fn default_entra_quando_a_prop_e_omitida() {
    const CASO: &str = "default_entra_quando_a_prop_e_omitida";
    let (mut motor, _) = motor_com(CASO, r##"<Cartao nome="Alice" />"##);
    motor.reevaluate_all().expect("o default cobre `cor`");
    let textos = format!("{:?}", motor.evaluated("tela").unwrap());
    assert!(textos.contains("Alice / #89B4FA"), "{textos}");
}

/// O default também **fecha a queda** para o contexto de baixo: sem ele, um
/// `{cor}` omitido pegaria a chave global de mesmo nome.
#[test]
fn default_ganha_da_chave_global_de_mesmo_nome() {
    const CASO: &str = "default_ganha_da_chave_global_de_mesmo_nome";
    let (mut motor, _) = motor_com(CASO, r##"<Cartao nome="Alice" />"##);
    motor.define_data("cor", "#DEADBE");
    motor.reevaluate_all().unwrap();
    let textos = format!("{:?}", motor.evaluated("tela").unwrap());
    assert!(
        textos.contains("#89B4FA"),
        "o default do <prop> manda: {textos}"
    );
}

/// Declarar é opcional: um `<component>` sem `<props>` não ganha checagem
/// nenhuma — é o que mantém compatível todo componente que lê o contexto
/// global em vez de receber props (o `perfil_card.gv` dos exemplos é assim).
#[test]
fn componente_sem_props_nao_e_checado() {
    const CASO: &str = "componente_sem_props_nao_e_checado";
    let livre = escreve(
        CASO,
        "livre.gv",
        r##"<component><Text content="{qualquer}" /></component>"##,
    );
    let tela = escreve(
        CASO,
        "tela_livre.gv",
        &format!(
            r##"<screen title="T">
                <resources><link rel="import" href="{livre}" as="Livre" /></resources>
                <Column><Livre inventada="x" /></Column>
            </screen>"##
        ),
    );
    let mut motor = GlacierUI::new();
    motor.register_component("tela", &tela).unwrap();
    motor.set_initial_screen("tela");
    motor
        .reevaluate_all()
        .expect("sem <props> não há contrato a violar");
}

/// Um `<props>` vazio é um contrato, não a ausência de um: ele diz "não aceito
/// prop nenhuma".
#[test]
fn props_vazio_recusa_qualquer_prop() {
    const CASO: &str = "props_vazio_recusa_qualquer_prop";
    let fechado = escreve(
        CASO,
        "fechado.gv",
        r##"<component><props></props><Text content="x" /></component>"##,
    );
    let tela = escreve(
        CASO,
        "tela_fechada.gv",
        &format!(
            r##"<screen title="T">
                <resources><link rel="import" href="{fechado}" as="Fechado" /></resources>
                <Column><Fechado qualquer="x" /></Column>
            </screen>"##
        ),
    );
    let mut motor = GlacierUI::new();
    motor.register_component("tela", &tela).unwrap();
    motor.set_initial_screen("tela");
    let msg = motor.reevaluate_all().unwrap_err().to_string();
    assert!(msg.contains("não aceita a prop 'qualquer'"), "{msg}");
    assert!(msg.contains("nenhuma"), "{msg}");
}

/// `for-each`/`var` chegam no mesmo mapa das props (o parser encaminha todo
/// atributo), mas são diretivas — quem as lê é a expansão dos filhos, antes de
/// o componente ser inlinado. Acusá-las de prop desconhecida quebraria o uso
/// mais comum de um card em lista.
#[test]
fn diretivas_nao_contam_como_prop() {
    const CASO: &str = "diretivas_nao_contam_como_prop";
    let cartao = escreve(
        CASO,
        "cartao_lista.gv",
        r##"<component>
            <props><prop name="nome" /></props>
            <Text content="{nome}" />
        </component>"##,
    );
    let tela = escreve(
        CASO,
        "tela_lista.gv",
        &format!(
            r##"<screen title="T">
                <resources><link rel="import" href="{cartao}" as="Cartao" /></resources>
                <Column><Cartao for-each="itens" var="i" nome="{{i.nome}}" /></Column>
            </screen>"##
        ),
    );
    let mut motor = GlacierUI::new();
    motor.register_component("tela", &tela).unwrap();
    motor.set_initial_screen("tela");
    motor.define_data("itens", r#"[{"nome":"Alice"},{"nome":"Bob"}]"#);
    motor
        .reevaluate_all()
        .expect("for-each/var são diretivas, não props");
    let arvore = format!("{:?}", motor.evaluated("tela").unwrap());
    assert!(
        arvore.contains("Alice") && arvore.contains("Bob"),
        "{arvore}"
    );
}
