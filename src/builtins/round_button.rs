/// QML `RoundButton`: um `<Button>` com `border_radius` total — um círculo em
/// vez de um retângulo de cantos vivos. Para um único glifo ou ícone curto
/// (`+`, `×`, `⋮`), o botão de ação flutuante (FAB) que a maioria dos apps
/// Material/QML tem num canto.
///
/// ```xml
/// <RoundButton text="+" color="#89B4FA" on_click="novo_item" size="48" />
/// ```
///
/// # Por que ele é só isso
///
/// `border_radius` >= metade do lado já arredonda até o círculo — a mesma
/// conta que o círculo de iniciais do `<Avatar>` usa desde a 0.65 (passar o
/// **lado inteiro**, não a metade, dispensa uma prop `radius` à parte). Um
/// `RoundButton` é, byte a byte, `<Button width="N" height="N"
/// border_radius="N">`; a tag existe pela conveniência de não repetir `N` três
/// vezes, não porque haja lógica nova.
///
/// # Por que `color` sempre tem um valor
///
/// No `<Button>` do motor, `border_radius`/`border_width` só entram no
/// `style()` quando `color` (o fundo) está presente — sem fundo custom, o
/// botão cai no tema default e desenha cantos retos, `border_radius` ou não
/// (ver `widget.rs`, o braço de `NodeType::Button`). Por isso `color` aqui tem
/// um default inline (`.roundbutton` não bastaria: um default de CLASSE
/// resolve *depois* de checar se o inline é vazio, e um `color=""` some antes
/// de chegar lá) — sem ele, todo `RoundButton` sem `color` explícito seria só
/// um botão quadrado com a tag errada.
///
/// # Props
///
/// - `text`  — o glifo/ícone (um `<Text>` curto). Obrigatório.
/// - `color` — cor de fundo do círculo. Default `#45475A` (neutro do tema).
/// - `size`  — diâmetro em px. Default `40`.
/// - `on_click` — ação ao soltar.
///
/// # Classes
///
/// `class` no uso pinta o círculo — é a raiz e não há nó de dentro para uma
/// prop `*_class` alcançar (o `<Button>` do motor não abre `<slot/>`, e este
/// widget não precisa: é um `text=` só).
use crate::component::{Component, Context, Template};

pub struct RoundButton;

impl Component for RoundButton {
    fn name(&self) -> &str {
        "RoundButton"
    }

    fn template(&self) -> Template {
        Template::Inline(
            r#"<Button
                    text="{text}"
                    on_click="{on_click}"
                    color="{color|#45475A}"
                    width="{size|40}"
                    height="{size|40}"
                    border_radius="{size|40}"
                    text_align="center"
                />"#
                .to_string(),
        )
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {
        // Sem estado — é um `<Button>` com outra forma, nada mais.
    }
}
