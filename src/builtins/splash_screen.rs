/// `QSplashScreen`: uma tela de abertura que cobre o conteúdo principal
/// enquanto o app carrega, e some quando ele estiver pronto.
///
/// ```xml
/// <SplashScreen show="{carregando}">
///     <Column align_x="center" align_y="center" spacing="12">
///         <Text content="Minha Ferramenta" size="22" bold="true" />
///         <Spinner />
///     </Column>
///
///     <template slot="splash">
///         <Svg source="logo.svg" width="120" height="120" />
///     </template>
/// </SplashScreen>
/// ```
///
/// Espera, o exemplo trocou os dois? Não: o slot **anônimo** é sempre o
/// conteúdo principal (a tela de verdade, por baixo — como em qualquer outro
/// builtin desta lib), e `slot="splash"` é o que cobre por cima enquanto
/// `show` for verdadeiro. Nomear o de cima, não o de baixo, é a mesma
/// convenção do rodapé do `<Card>`: o comum fica anônimo, o extra ganha nome.
///
/// # Por que ele não anima
///
/// A intenção inicial (`PLANO_WIDGETS.md`, Onda 11) era cobrir com um
/// `<Reveal>` e deixar a splash "enrolar" para revelar o conteúdo por baixo —
/// as três peças (`<stack>`, `<reveal>`, `<container>`) já existiam. Não
/// funciona: um `<Reveal>` interpola a **altura natural** do filho, e o painel
/// da splash precisa ser `height="fill"` para cobrir a tela inteira — as duas
/// coisas juntas são exatamente a armadilha do `Length::Fill` que o
/// `PRIMITIVAS.md` documenta (um `Fill` não tem "altura natural" para
/// interpolar até ela). Descoberto ao escrever o widget, não ao planejá-lo —
/// a mesma classe de correção que este catálogo já fez treze vezes para o
/// lado do estado, agora para o lado da animação.
///
/// A saída honesta é a que o `QSplashScreen` do Qt também usa: aparece,
/// desaparece, sem transição. `show="{carregando}"` troca de instantâneo — o
/// `<stack>` (habilitador A da Onda 11) ainda é o que faz a sobreposição
/// existir, só não com o enfeite que a nota original prometia.
///
/// # Props
///
/// - `show`       — chave/valor truthy: mostra a splash. Default `true` (uma
///   splash que nasce sem `show` começa visível, o caso comum).
/// - `background` — fundo do painel de cima. Sem ele, cai em `.splash-panel`.
///
/// # Classes
///
/// `class` no uso pinta a raiz (o `<stack>`, sem efeito visual próprio);
/// `panel_class` alcança o painel que cobre a tela.
use crate::component::{Component, Context, Template};

pub struct SplashScreen;

impl Component for SplashScreen {
    fn name(&self) -> &str {
        "SplashScreen"
    }

    fn template(&self) -> Template {
        Template::Inline(
            r#"<Stack>
                    <style>
                        .splash-panel { background: #1e1e2e; }
                    </style>

                    <slot/>

                    <template if="{show|true}" notEquals="false">
                        <Container
                            class="splash-panel {panel_class}"
                            background="{background}"
                            width="fill"
                            height="fill"
                            align_x="center"
                            align_y="center"
                        >
                            <slot name="splash"/>
                        </Container>
                    </template>
                </Stack>"#
                .to_string(),
        )
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {
        // Apresentacional: `show` é lido, nunca escrito por aqui — quem
        // decide quando o app terminou de carregar é a tela, não o widget.
    }
}
