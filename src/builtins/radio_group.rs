/// `QButtonGroup` de `QRadioButton`: um grupo de opções mutuamente exclusivas,
/// montado a partir de uma coleção do contexto.
///
/// ```xml
/// <radiogroup value="plano" items="planos" />
/// ```
/// ```rust,ignore
/// ctx.set("planos", r#"[{"id":"free","label":"Grátis"},{"id":"pro","label":"Pro"}]"#);
/// ctx.set("plano", "free");
/// ```
///
/// # Por que ele existe, se a primitiva `<Radio>` já existe
///
/// Duas razões, e as duas são economia de código do lado do app:
///
/// 1. **Ele escreve a chave sozinho.** A primitiva [`crate::parser::NodeType::Radio`]
///    segue a regra do `<Checkbox>`: dispara a ação e quem grava é o app. Este
///    builtin tem `update` próprio, então o clique já grava — nenhum handler, nem
///    em Rust nem em Luau. É o padrão do [`super::spin_box::SpinBox`]: a chave
///    vem por prop e viaja dentro da ação (`pick:plano|pro`).
/// 2. **Uma tag por grupo, não uma por opção.** As opções vêm de uma coleção,
///    do mesmo jeito que as abas do [`super::tab_bar::TabBar`] e os itens de um
///    `<Menu items="…">`.
///
/// Quem quiser as opções escritas à mão, com markup diferente em cada uma, usa
/// `<Radio>` direto e escreve o handler.
///
/// # Por que ele NÃO precisa de um `active`, e o `TabBar` precisa
///
/// O [`super::tab_bar::TabBar`] pede duas props para a mesma coisa (`value="aba"
/// active="{aba}"`) porque quem decide o destaque lá é o **template**, e um
/// template não consegue ler o valor da chave cujo *nome* está numa prop — a
/// indireção `{{value}}` que o interpolador não tem.
///
/// Aqui quem decide é a **primitiva**: o `<Radio>` recebe o nome da chave e faz
/// `ctx.get(chave) == value` na hora do render, em Rust, onde a indireção é uma
/// linha. Por isso o template pode simplesmente repassar `group="{value}"` e
/// uma prop some. É a diferença entre resolver no markup e resolver no motor.
///
/// # Props
///
/// - `items`   — **obrigatória**: nome da chave com o array de `{id, label}`.
/// - `value`   — **obrigatória**: nome da chave que guarda o `id` escolhido —
///   lida para marcar a opção certa, e escrita no clique.
/// - `layout`  — `column` (default) ou `row`, o `Qt::Orientation` do grupo.
/// - `spacing` — espaço entre as opções. Default: `8`.
use crate::component::{Component, Context, Template};

pub struct RadioGroup;

impl Component for RadioGroup {
    fn name(&self) -> &str {
        "RadioGroup"
    }

    fn template(&self) -> Template {
        // `group="{value}"` repassa o NOME da chave para a primitiva, que a lê
        // no render. O `on_change` continua carregando o mesmo nome dentro da
        // ação, porque o `update` abaixo não enxerga as props da instância.
        Template::Inline(
            r#"<Column spacing="{spacing|8}">
                    <template if="{layout|column}" equals="row">
                        <Row spacing="{spacing|16}" align_y="center">
                            <template for-each="{items}" var="opt">
                                <Radio
                                    label="{opt.label}"
                                    value="{opt.id}"
                                    group="{value}"
                                    onChange="pick:{value}|{opt.id}"
                                />
                            </template>
                        </Row>
                    </template>

                    <template else>
                        <template for-each="{items}" var="opt">
                            <Radio
                                label="{opt.label}"
                                value="{opt.id}"
                                group="{value}"
                                onChange="pick:{value}|{opt.id}"
                            />
                        </template>
                    </template>
                </Column>"#
                .to_string(),
        )
    }

    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        // `pick:plano|pro` — a chave que o app nomeou, e o id da opção clicada.
        let Some(("pick", payload)) = action.split_once(':') else {
            return;
        };
        let Some((chave, id)) = payload.split_once('|') else {
            return;
        };
        let chave = chave.trim();
        // Sem `value` não há onde gravar: não faz nada, em vez de inventar uma
        // chave e poluir o contexto do app (mesma regra do `SpinBox`).
        if chave.is_empty() {
            return;
        }
        ctx.set(chave, id.to_string());
    }
}
