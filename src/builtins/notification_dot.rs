/// O pontinho de notificação: um badge pequeno sobre o canto de um ícone —
/// "há coisa nova aqui", sem número (para o número, ver [`super::badge::Badge`]
/// combinado com este, ou o `content` deste widget).
///
/// ```xml
/// <NotificationDot>
///     <Svg source="icons/sino.svg" size="24" />
/// </NotificationDot>
/// ```
///
/// # Por que ele esperou a Onda 11
///
/// A nota do `Avatar` (0.65) já dizia: *"o pontinho verde de 'online' no canto
/// pediria sobreposição (`Stack`) dentro de um builtin; hoje se faz por fora"*.
/// A Onda 11 do `PLANO_WIDGETS.md` é exatamente essa sobreposição —
/// `<stack>` + `anchor`, o habilitador A — e este widget é o primeiro
/// consumidor dela: não tinha como nascer antes do `<stack>` existir.
///
/// # Como funciona
///
/// O conteúdo (`<slot/>`, tipicamente um ícone) ocupa a primeira camada do
/// `<stack>`, do tamanho natural dele; o pontinho é a segunda, num
/// `container` `Fill` alinhado ao canto por `anchor` — sem `pin`, porque um
/// canto não precisa saber o tamanho do ícone.
///
/// # Props
///
/// - `color`  — cor do pontinho. Default: `#F38BA8` (vermelho do tema).
/// - `size`   — diâmetro em px. Default: `10`.
/// - `anchor` — qual canto: `top-right` (default, o comum para notificação),
///   `top-left`, `bottom-right`, `bottom-left`, e os outros cinco da grade 3×3
///   (ver `NodeType::Stack`).
/// - `show`   — `"false"` some com o pontinho sem remover o ícone (útil para
///   `show="{tem_notificacao}"`) . Default: mostrado.
///
/// # Classes
///
/// `class` no uso pinta a **raiz** (o `<stack>`, sem efeito visual próprio);
/// `dot_class` alcança o pontinho, que a classe do uso não atinge — o mesmo
/// padrão do `text_class` do `<Badge>`. O conteúdo do `<slot/>` não tem classe
/// própria aqui: já é o widget que quem chama escreveu, com a classe que
/// quiser.
use crate::component::{Component, Context, Template};

pub struct NotificationDot;

impl Component for NotificationDot {
    fn name(&self) -> &str {
        "NotificationDot"
    }

    fn template(&self) -> Template {
        Template::Inline(
            r#"<Stack>
                    <style>
                        .notification-dot { background: #F38BA8; }
                    </style>

                    <slot/>

                    <template if="{show|true}" notEquals="false">
                        <Container
                            class="notification-dot {dot_class}"
                            width="{size|10}"
                            height="{size|10}"
                            border_radius="{size|10}"
                            anchor="{anchor|top-right}"
                        />
                    </template>
                </Stack>"#
                .to_string(),
        )
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {
        // Apresentacional: sem estado, sem comportamento.
    }
}
