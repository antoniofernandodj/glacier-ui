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
/// # `width`/`height` são PROPS, não a classe do uso — e é por um motivo
///
/// Primeira tentativa: deixar o `<Stack>` raiz sem tamanho nenhum e confiar
/// que `class="minha-classe"` no uso alcançaria `width`/`height` pelo overlay
/// que "a classe do uso pinta a raiz" já dá a qualquer componente. Não
/// funciona para ESTES dois campos: um atributo **inline** no template sempre
/// vence a classe (`eval.rs`, o comentário sobre overlay entrando por
/// último) — então, se o `<Stack>` do template não escreve `width`/`height`,
/// ele cai no padrão do motor (`Shrink`, como `<container>`), mede pela
/// camada BASE (o `<slot/>`, o conteúdo principal do app — quase sempre
/// `width="fill"`), e um `Fill` medido dentro de um `Shrink` colapsa a quase
/// zero (a armadilha do `Length::Fill`, `PRIMITIVAS.md`) — a classe do uso
/// nunca chega a ser lida, porque não há vazio para ela preencher.
///
/// A segunda tentativa foi o oposto — cravar `width="fill" height="fill"`
/// direto no `<Stack>` do template — e furou pelo motivo simétrico: agora o
/// inline SEMPRE vence, e um app que queria um preview contido (`height=
/// "220"`, via classe) não tinha como. Pior: um `<SplashScreen>` inteiro é,
/// ele mesmo, um filho `Fill` dentro de QUALQUER coluna comum (`Shrink`,
/// como a maioria) — a mesma armadilha, só um nível acima.
///
/// A saída: `width`/`height` viram **props de verdade**, com `{prop|fill}` —
/// resolvidas antes da árvore fechar, então tanto o padrão (`fill`, o caso
/// comum: cobrir o app inteiro) quanto uma altura fixa por instância (`
/// <SplashScreen height="220">`, um preview contido) chegam ao `<Stack>` pelo
/// MESMO caminho, sem depender da ordem de precedência entre inline e classe.
///
/// # Props
///
/// - `show`       — chave/valor truthy: mostra a splash. Default `true` (uma
///   splash que nasce sem `show` começa visível, o caso comum).
/// - `width`/`height` — tamanho do `<Stack>` raiz. Default `fill`/`fill` — o
///   caso comum é cobrir o app inteiro. Dê um `height` fixo para um preview
///   contido (é o que `examples/onda11` faz).
/// - `background` — fundo do painel de cima. Sem ele, cai em `.splash-panel`.
///
/// **A `<Column>`/`<Row>` que envolve o uso precisa de `width="fill"`** — não
/// é peculiaridade deste widget, é o `iced`: uma coluna sem `width` fica
/// `Shrink`, e um filho `width="fill"` dentro dela (este widget, no default)
/// não mede pela largura do app — mede pela largura do IRMÃO mais largo na
/// mesma coluna (a "quarta passagem" do algoritmo de flex,
/// `iced_core::layout::flex`, que resolve um item Fixed-no-eixo-principal/
/// Fill-no-cruzado usando o `cross` já medido dos irmãos não-fluidos, nunca o
/// espaço disponível de verdade). O sintoma é sutil: a splash **aparece**,
/// só que espremida na largura de um texto ou botão ao lado — fácil de ler
/// como "não funciona". `examples/onda11` documenta isso na classe
/// `.bloco-fill`.
///
/// # Classes
///
/// `class` no uso pinta a raiz (o `<stack>`, sem efeito visual próprio —
/// mas o `border_radius`/`border_width`/`border_color` de uma classe SIM
/// pintam, porque nenhum dos três tem um inline concorrente no template);
/// `panel_class` alcança o painel que cobre a tela.
use crate::component::{Component, Context, Template};

pub struct SplashScreen;

impl Component for SplashScreen {
    fn name(&self) -> &str {
        "SplashScreen"
    }

    fn template(&self) -> Template {
        Template::Inline(
            r#"<Stack width="{width|fill}" height="{height|fill}">
                    <style>
                        .splash-panel { background: #1e1e2e; }
                    </style>

                    <slot/>

                    <!-- SEM `notEquals="false"`, e o motivo custou quatro
                         rodadas de correção errada: com ele, a condição virava
                         `valor != "false"` — e o jeito idiomático de desligar
                         uma chave neste motor é gravar VAZIO nela (`ctx.set(
                         "carregando", "")`), não a string "false". Vazio passa
                         em `!= "false"`, então a splash abria e NUNCA fechava:
                         o botão rodava a ação, o status mudava, e o painel
                         ficava. Sem comparador nenhum, `eval_condition` cai no
                         `is_truthy` do motor — vazio, "false" e "0" são
                         falsos, "true"/"1"/"yes"/"sim" são verdadeiros —, que
                         é a semântica que todo o resto do motor usa. -->
                    <template if="{show|true}">
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
