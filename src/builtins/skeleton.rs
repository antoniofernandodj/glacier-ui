/// O placeholder de carregamento: uma barra cinza do tamanho que o conteúdo
/// real vai ocupar, para a tela não "pular" quando ele chegar.
///
/// ```xml
/// <Skeleton width="200" height="16" />
/// <Skeleton width="fill" height="120" radius="8" />
/// ```
///
/// # Por que ele é só isso
///
/// A §6.3 do `PLANO_WIDGETS.md` sempre foi honesta sobre este: "quatro linhas
/// de markup em qualquer app; vira tag só por conveniência". Não há
/// comportamento — nem a pulsação animada que alguns kits desenham, porque
/// isso pediria um tique de relógio por instância na tela (a família do
/// `Spinner`/`DelayButton`) para uma economia de quatro linhas que não
/// justifica o custo. Um `Skeleton` parado já resolve o problema que ele
/// existe para resolver: reservar o espaço.
///
/// # Props
///
/// - `width`/`height` — dimensões do placeholder. Defaults `fill`/`16`.
/// - `radius`         — `border_radius`. Default `4`.
///
/// # Classes
///
/// `class` no uso pinta o retângulo — é a raiz, e não há nó de dentro.
use crate::component::{Component, Context, Template};

pub struct Skeleton;

impl Component for Skeleton {
    fn name(&self) -> &str {
        "Skeleton"
    }

    fn template(&self) -> Template {
        Template::Inline(
            r#"<Container
                    class="skeleton-placeholder"
                    background="{background}"
                    width="{width|fill}"
                    height="{height|16}"
                    border_radius="{radius|4}"
                >
                    <style>
                        .skeleton-placeholder { background: #8080802f; }
                    </style>
                </Container>"#
                .to_string(),
        )
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {
        // Apresentacional: sem estado, sem comportamento — de propósito (ver
        // a nota acima sobre a pulsação que este widget não tem).
    }
}
