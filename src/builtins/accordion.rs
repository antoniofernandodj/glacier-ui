/// `Accordion` / `QToolBox`: seções empilhadas que abrem e fecham.
///
/// São **os dois modos do mesmo widget**, e a diferença cabe numa linha: o
/// accordion guarda um **conjunto** de seções abertas, o tool box guarda **uma**
/// — e por isso o primeiro precisou do `contains` (0.84) e o segundo não
/// precisa de nada.
///
/// ```xml
/// <accordion>
///     <accordionitem title="Rede" value="abertas" open="{abertas}" id="rede">
///         <input value="host" />
///     </accordionitem>
///     <accordionitem title="Disco" value="abertas" open="{abertas}" id="disco">
///         <text>Nada por aqui ainda.</text>
///     </accordionitem>
/// </accordion>
///
/// <toolbox>
///     <toolboxitem title="Ferramentas" value="secao" open="{secao}" id="ferr"> … </toolboxitem>
///     <toolboxitem title="Camadas"     value="secao" open="{secao}" id="cam">  … </toolboxitem>
/// </toolbox>
/// ```
///
/// # Por que uma tag por item, e não uma coleção
///
/// Porque o **conteúdo** de cada seção é diferente, e conteúdo é do app: um
/// `items="secoes"` daria os títulos, mas o corpo teria de sair de um
/// `<slot/>` por seção — e os nomes de slot são fixos no template do
/// componente (0.67), então o widget não pode inventar um por item da coleção.
///
/// A saída é a do próprio Qt: `QToolBox::addItem(widget, "Título")` também
/// recebe uma seção por chamada. Aqui a chamada é uma tag, o widget é o que
/// está escrito entre elas, e o `<slot/>` (0.65) é o que torna isso possível.
///
/// # O `Accordion`/`ToolBox` de fora é só a moldura
///
/// Um `<Column>` com espaçamento e uma borda opcional. Todo o comportamento
/// mora no **item** — que por isso funciona sozinho, sem a moldura, quando o
/// que se quer é uma seção dobrável solta no meio de um formulário.
///
/// # `value` e `open` andam em par
///
/// A terceira vez que este par aparece na biblioteca (`TabBar`, `ListView`,
/// agora aqui), e sempre pelo mesmo motivo: o template precisaria ler o valor
/// da chave cujo *nome* está numa prop — a indireção `{{value}}` que o
/// interpolador não tem. `value` é o **nome** da chave; `open` é o **valor**
/// atual dela.
///
/// # Props do item
///
/// - `title`   — o rótulo do cabeçalho.
/// - `value`   — **nome** da chave que guarda o que está aberto.
/// - `open`    — o valor atual dessa chave.
/// - `id`      — o identificador desta seção dentro da chave.
/// - `sub`     — segunda linha do cabeçalho, menor. Opcional.
/// - `padding` — espaço interno do corpo. Default: `12`.
/// - `spacing` — espaço entre os filhos do corpo. Default: `8`.
///
/// # Aparência
///
/// `.accordion-head` / `.accordion-body` na folha global do template. O
/// indicador é `▾`/`▸` — texto, não ícone: o widget não pode depender de um
/// arquivo `.svg` que o app talvez não tenha.
use crate::component::{Component, Context, Template};

pub struct Accordion;
pub struct AccordionItem;
pub struct ToolBox;
pub struct ToolBoxItem;

/// O corpo comum das duas molduras: uma `<Column>` que só empilha o que
/// escreverem dentro dela. `op` distingue o que o item faz no clique.
fn moldura(largura_padrao: &str) -> Template {
    Template::Inline(format!(
        r#"<Column spacing="{{spacing|4}}" width="{{width|{largura_padrao}}}">
                <slot/>
            </Column>"#
    ))
}

/// O item: um cabeçalho clicável e um corpo que só existe quando aberto.
///
/// `op` é a operação que o `update` executa — `toggle` (conjunto) ou `only`
/// (um por vez) —, e `teste` é a diretiva que decide o destaque: `contains`
/// para o conjunto, `equals` para o único.
fn item(op: &str, aberto: &str, fechado: &str) -> Template {
    Template::Inline(format!(
        r#"<Column spacing="0" width="{{width|fill}}">
                <style>
                    .accordion-head {{
                        color: #8080801f;
                        text-color: #cdd6f4;
                        border-width: 0;
                        border-radius: 5;
                    }}
                    .accordion-head:hover {{ background: #8080803d; }}
                    .accordion-mark {{ color: #80868d; }}
                    .accordion-sub  {{ color: #80868d; }}
                </style>

                {aberto}

                {fechado}
            </Column>"#,
        aberto = aberto.replace("{OP}", op),
        fechado = fechado.replace("{OP}", op),
    ))
}

/// O cabeçalho, com o glifo do estado. Repetido nos dois braços porque a
/// alternativa — um `<slot/>` interno — não existe: o slot é o conteúdo de
/// quem USA o widget, não um mecanismo de reuso dentro do template.
fn cabecalho(marca: &str) -> String {
    format!(
        r#"<Button
                        class="accordion-head"
                        on_click="{{OP}}:{{value}}|{{id}}"
                        padding="{{head_padding|9 12}}"
                        width="fill"
                    >
                        <Row spacing="8" align_y="center" width="fill">
                            <Text class="accordion-mark" content="{marca}" size="12" />
                            <Column spacing="1" width="fill">
                                <Text content="{{title}}" size="{{size|13}}" bold="true" />
                                <template if="{{sub}}" notEquals="">
                                    <Text class="accordion-sub" content="{{sub}}" size="11" />
                                </template>
                            </Column>
                        </Row>
                    </Button>"#
    )
}

impl Component for Accordion {
    fn name(&self) -> &str {
        "Accordion"
    }
    fn template(&self) -> Template {
        moldura("fill")
    }
    fn update(&mut self, _a: &str, _v: Option<&str>, _c: &mut Context) {
        // Moldura pura: o comportamento é do item.
    }
}

impl Component for ToolBox {
    fn name(&self) -> &str {
        "ToolBox"
    }
    fn template(&self) -> Template {
        moldura("fill")
    }
    fn update(&mut self, _a: &str, _v: Option<&str>, _c: &mut Context) {}
}

impl Component for AccordionItem {
    fn name(&self) -> &str {
        "AccordionItem"
    }

    fn template(&self) -> Template {
        // `contains`: a chave guarda o CONJUNTO das seções abertas, e é aqui
        // que o habilitador da 0.84 paga o aluguel — sem ele, cada seção
        // precisaria da sua própria chave, que é exatamente o "estado por
        // instância" que o plano dizia faltar.
        item(
            "toggle",
            &format!(
                r#"<template if="{{open}}" contains="{{id}}">
                    {}
                    <Column
                        class="accordion-body"
                        spacing="{{spacing|8}}"
                        padding="{{padding|12}}"
                        width="fill"
                    >
                        <slot/>
                    </Column>
                </template>"#,
                cabecalho("▾")
            ),
            &format!(
                r#"<template else>
                    {}
                </template>"#,
                cabecalho("▸")
            ),
        )
    }

    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        // `toggle:abertas|rede` — a chave que o app nomeou e o id desta seção.
        let Some(("toggle", payload)) = action.split_once(':') else {
            return;
        };
        let Some((chave, id)) = payload.split_once('|') else {
            return;
        };
        let (chave, id) = (chave.trim(), id.trim());
        if chave.is_empty() || id.is_empty() {
            return;
        }
        let atual = ctx.get(chave).cloned().unwrap_or_default();
        ctx.set(chave, super::list_view::alterna_no_conjunto(&atual, id));
    }
}

impl Component for ToolBoxItem {
    fn name(&self) -> &str {
        "ToolBoxItem"
    }

    fn template(&self) -> Template {
        // `equals`: a chave guarda UMA seção. É o `TabBar` na vertical, e por
        // isso nunca precisou do `contains` — nem esteve bloqueado por nada.
        item(
            "only",
            &format!(
                r#"<template if="{{open}}" equals="{{id}}">
                    {}
                    <Column
                        class="accordion-body"
                        spacing="{{spacing|8}}"
                        padding="{{padding|12}}"
                        width="fill"
                    >
                        <slot/>
                    </Column>
                </template>"#,
                cabecalho("▾")
            ),
            &format!(
                r#"<template else>
                    {}
                </template>"#,
                cabecalho("▸")
            ),
        )
    }

    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        // `only:secao|ferr` — abre esta e fecha as outras. Clicar na que já
        // está aberta **fecha** (a chave vira vazia): é o `QToolBox` sendo
        // gentil, e é o que a pessoa espera de um cabeçalho clicável.
        let Some(("only", payload)) = action.split_once(':') else {
            return;
        };
        let Some((chave, id)) = payload.split_once('|') else {
            return;
        };
        let (chave, id) = (chave.trim(), id.trim());
        if chave.is_empty() || id.is_empty() {
            return;
        }
        let ja_aberta = ctx.get(chave).is_some_and(|v| v.trim() == id);
        ctx.set(
            chave,
            if ja_aberta {
                String::new()
            } else {
                id.to_string()
            },
        );
    }
}
