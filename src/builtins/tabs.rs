/// `Tabs` / `QTabWidget`: a fileira de abas **mais a página** — o widget
/// inteiro, não só a barra.
///
/// ```xml
/// <tabs value="aba" active="{aba}" items="abas">
///     <template slot="geral"> … a página Geral … </template>
///     <template slot="rede">  … a página Rede …  </template>
/// </tabs>
/// ```
///
/// # O 🟡 mais visível da §2.8, e por que ele durou tanto
///
/// A `<tabbar>` saiu na 0.65 e desde então carregava esta ressalva no próprio
/// doc: *"o `QTabWidget` do Qt é a barra **mais** o empilhado de páginas. A
/// barra sai hoje; o empilhado não, porque cada página precisaria do seu
/// próprio buraco no template (`<slot name="geral"/>`, `<slot name="rede"/>`) e
/// o `<slot/>` do motor é único e anônimo."*
///
/// O slot nomeado chegou na 0.67 e a frase envelheceu meio errada: os buracos
/// passaram a existir, mas eram **literais** — um widget não conseguia escolher
/// a região por um valor de contexto. Faltava uma linha, e é o habilitador A da
/// Onda 5: o nome do slot interpola.
///
/// Com ele, o template abaixo é o widget inteiro. `addTab(widget, "Geral")` do
/// Qt vira `<template slot="geral">`, e a tela para de repetir a lista de abas
/// duas vezes — uma no `items` e outra numa escada de `se`/`senão`.
///
/// # As páginas são avaliadas, todas, sempre
///
/// Vale saber, porque é o preço desta forma e ele não é escondido: o conteúdo
/// de **todas** as abas é avaliado na fronteira do componente (é assim que o
/// `<slot/>` funciona desde a 0.65 — a partição acontece uma vez, sobre os
/// filhos crus). O que só a aba ativa faz é **renderizar**.
///
/// É a mesma troca do corpo fechado de um `<accordionitem>`, e a mesma saída:
/// para uma aba com uma lista cara, `virtualize` na coluna lá dentro (ver
/// `PRIMITIVAS.md`). Para uma aba com conteúdo caro de *avaliar* — um
/// `for-each` sobre milhares de linhas —, a `<tabbar>` sozinha com `se`/`senão`
/// continua existindo e continua correta.
///
/// # `value` e `active` andam em par
///
/// A quarta vez que este par aparece na biblioteca, e sempre pelo mesmo motivo:
/// o template precisaria ler o valor da chave cujo *nome* está numa prop — a
/// indireção `{{value}}` que o interpolador não tem. `value` é o **nome** da
/// chave; `active` é o **valor** atual dela.
///
/// Aqui ele paga dobrado: `active` não serve só ao destaque da barra, é ele que
/// escolhe a página (`<slot name="{active}"/>`).
///
/// # Uma aba sem página escrita
///
/// Cai no conteúdo de reserva do `<slot>` — que este template deixa vazio de
/// propósito. Uma aba sem conteúdo mostra a barra e nada abaixo dela, que é
/// melhor do que mostrar a página da aba anterior.
///
/// # Props
///
/// - `items`   — **obrigatória**: nome da chave com o array de `{id, label}`.
/// - `value`   — **obrigatória**: nome da chave que recebe o `id` clicado.
/// - `active`  — o valor atual dessa chave. Sem ela, nenhuma página aparece.
/// - `spacing` — espaço entre a barra e a página. Default `14`.
/// - `padding` — espaço interno da página. Default `0`.
/// - `width`   — largura do conjunto. Default `fill`.
///
/// As props de aparência da barra passam direto para a
/// [`super::tab_bar::TabBar`] embutida: `tab_class`, `tab_active_class`,
/// `label_class`, `tab_padding`, `size`.
///
/// # Aparência
///
/// `.tabs-page` na folha global do template, mais `page_class` para o app
/// alcançar a página sem reescrever o widget — o padrão de classe por nó
/// interno da 0.89.
use crate::component::{Component, Context, Template};

pub struct Tabs;

impl Component for Tabs {
    fn name(&self) -> &str {
        "Tabs"
    }

    fn template(&self) -> Template {
        // A barra é a `<TabBar>` que já existia — inteira, com o `update` dela
        // tratando o clique. Este componente não trata ação nenhuma, e é por
        // isso que o `update` abaixo está vazio: o que ele acrescenta é a
        // página, e página é markup.
        Template::Inline(
            r#"<Column spacing="{spacing|14}" width="{width|fill}">
                    <TabBar
                        value="{value}"
                        active="{active}"
                        items="{items}"
                        padding="{tab_padding|7 14}"
                        size="{size|13}"
                        tab_class="{tab_class}"
                        tab_active_class="{tab_active_class}"
                        label_class="{label_class}"
                    />

                    <!-- O habilitador A da Onda 5, numa linha: o nome do slot
                         interpola contra o contexto do componente, e `active`
                         é uma prop dele. `aba=rede` procura o balde que o uso
                         etiquetou com `slot="rede"`. -->
                    <Column
                        class="tabs-page {page_class}"
                        width="fill"
                        padding="{padding|0}"
                        spacing="{page_spacing|12}"
                    >
                        <slot name="{active}"/>
                    </Column>
                </Column>"#
                .to_string(),
        )
    }

    fn update(&mut self, _a: &str, _v: Option<&str>, _c: &mut Context) {
        // Nada: o clique é da `<TabBar>` embutida, que já grava a chave. Este
        // widget é a composição barra+página, e composição não tem estado.
    }
}
