/// `QGroupBox`: uma moldura com título ao redor de um grupo de controles.
///
/// É o **primeiro widget do motor que envolve conteúdo** — o que ele renderiza
/// não vem de props, vem do que se escreve entre as tags dele:
///
/// ```xml
/// <GroupBox title="Rede">
///     <Checkbox label="Usar proxy" checked="proxy" />
///     <TextInput value="host" placeholder="127.0.0.1" />
/// </GroupBox>
/// ```
///
/// Isso só existe a partir do `<slot/>` (ver [`crate::parser::NodeType::Slot`]):
/// antes dele, os filhos de um `<Componente>` eram descartados na expansão, e
/// todo widget-recipiente ficava fora do nível Builtin. O `GroupBox` é o caso
/// mais simples possível dessa família, e por isso o primeiro a nascer.
///
/// # O conteúdo é de quem escreveu, não do widget
///
/// A garantia que faz o slot valer a pena: o conteúdo é avaliado no contexto e
/// com o **dono de quem usou** o widget. O `on_click="salvar"` do exemplo
/// abaixo despacha `salvar` para a tela, não `GroupBox::salvar` para o `update`
/// daqui — ao contrário de uma ação escrita no template deste arquivo. Não é
/// preciso escapar nada com `app:` (ver [`crate::eval::APP_ACTION_PREFIX`]);
/// esse prefixo continua sendo para o outro caso, o da ação recebida por prop.
///
/// # As duas formas
///
/// A prop `flat` escolhe o desenho, como o `QGroupBox::flat` do Qt:
///
/// - **caixa** (default) — título acima de uma moldura fechada com o conteúdo
///   dentro. É o agrupador de formulário denso.
/// - `flat="true"` — título, uma linha horizontal e o conteúdo solto abaixo,
///   sem caixa. É o separador de seções de uma tela longa, onde uma moldura por
///   grupo viraria ruído.
///
/// O título é **opcional**: sem `title`, nem o texto nem a linha do `flat`
/// aparecem, e sobra a moldura pura — que é exatamente o [`super::frame::Frame`].
///
/// # Props
///
/// - `title`      — o rótulo do grupo. Vazio = sem cabeçalho.
/// - `flat`       — `true` para a forma sem caixa. Default: `false`.
/// - `padding`    — espaço interno da moldura. Default: `12`.
/// - `spacing`    — espaço entre os filhos. Default: `8`.
/// - `title_size` — corpo do título. Default: `13`.
/// - `width`      — largura do conjunto. Default: `fill` (o `QGroupBox` de um
///   layout vertical acompanha a largura dele; `shrink` encolhe no conteúdo).
///
/// # Aparência
///
/// Mesma escolha do [`super::spin_box::SpinBox`]: as cores saem de um `<style>`
/// **global** declarado no próprio template — instalado em `GlacierUI::new`,
/// portanto antes de qualquer `.gss` do app, que por isso vence por ordem.
/// Redefinir `.groupbox-frame` / `.groupbox-title` numa folha do app é o
/// caminho suportado para repintar.
///
/// O cinza translúcido (`#80808059`) é deliberado: clareia sobre tema escuro e
/// escurece sobre tema claro, então o mesmo default atravessa os quatro estilos
/// embutidos ([`crate::style`]) sem o widget saber qual está ativo.
use crate::component::{Component, Context, Template};

pub struct GroupBox;

impl Component for GroupBox {
    fn name(&self) -> &str {
        "GroupBox"
    }

    fn template(&self) -> Template {
        // O `<slot/>` aparece nos dois braços do `flat`, e isso é seguro: só um
        // sobrevive à expansão, então o conteúdo do uso é resolvido uma vez só.
        //
        // `notEquals=""` é o teste de "tem título?" — `if="{title}"` sozinho
        // aplicaria `is_truthy` ao texto, e um título como "Rede" não é
        // "true"/"1"/"sim".
        Template::Inline(
            r#"<Column spacing="6" width="{width|fill}">
                    <style>
                        .groupbox-frame {
                            border-width: 1;
                            border-color: #80808059;
                            border-radius: 6;
                        }
                        .groupbox-title { color: #80868d; }
                    </style>

                    <template if="{title}" notEquals="">
                        <Text
                            class="groupbox-title"
                            content="{title}"
                            size="{title_size|13}"
                            bold="true"
                        />
                    </template>

                    <template if="{flat|false}" equals="true">
                        <Rule />
                        <Column spacing="{spacing|8}" padding="{padding|8 2}" width="{width|fill}">
                            <slot/>
                        </Column>
                    </template>

                    <template else>
                        <Container
                            class="groupbox-frame"
                            padding="{padding|12}"
                            width="{width|fill}"
                        >
                            <Column spacing="{spacing|8}" width="{width|fill}">
                                <slot/>
                            </Column>
                        </Container>
                    </template>
                </Column>"#
                .to_string(),
        )
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {
        // Recipiente puro: todo comportamento é do conteúdo, e o conteúdo é do
        // app — as ações dele nem passam por aqui (ver a docstring).
    }
}
