/// `StackView` / `QStackedWidget`: as páginas sem a barra.
///
/// ```xml
/// <stackview active="{passo}">
///     <template slot="dados">   … a página Dados …   </template>
///     <template slot="revisao"> … a página Revisão … </template>
/// </stackview>
/// ```
///
/// # O 🟡 mais antigo da §2.8, e por que ele sobreviveu à Onda 5
///
/// O catálogo marca o `QStackedWidget` como "já dá com `se`; formalizar — sai
/// junto do `Tabs` completo, mesmo mecanismo". A Onda 5 construiu o `<tabs>` e
/// não construiu este, porque o mecanismo que faltava (o nome dinâmico de slot)
/// nasceu ali e o consumidor óbvio dele era a barra com página.
///
/// Ele é `<tabs>` **sem a barra**, e a diferença não é cosmética: um
/// `QStackedWidget` existe justamente para os casos em que quem troca de página
/// **não** é uma fileira de abas — um wizard (item 6 desta onda), um menu
/// lateral, um passo de formulário, uma máquina de estados. Escrever `<tabs>` e
/// esconder a barra por `.gss` funcionaria e seria mentira: o widget se
/// chamaria abas e não teria abas.
///
/// # Comparado com a escada de `se`/`senao`
///
/// A escada continua existindo e continua correta. O que muda:
///
/// - a página é escolhida por **nome**, não por posição numa cadeia de
///   condicionais — trocar a ordem de dois `<template slot>` não muda nada;
/// - o conteúdo de todas as páginas é **avaliado** (a partição do `<slot/>`
///   acontece uma vez, sobre os filhos crus), e só a ativa **renderiza**. É a
///   mesma troca do `<tabs>`, com a mesma saída: para uma página cara de
///   *avaliar*, a escada de `se` continua sendo a ferramenta certa.
///
/// # Props
///
/// - `active`  — o **valor** que escolhe a página (o `slot` que ela etiquetou).
///   Sem ela, nenhuma página aparece.
/// - `width` / `padding` / `spacing` — do container da página.
///
/// Note que aqui **não** existe o par `value`/`active` do `<tabs>`: este widget
/// não grava nada, porque não há nele nada em que clicar. Quem grava é quem
/// troca de página.
use crate::component::{Component, Context, Template};

pub struct StackView;

impl Component for StackView {
    fn name(&self) -> &str {
        "StackView"
    }

    fn template(&self) -> Template {
        Template::Inline(
            r#"<Column
                    class="stackview-page {page_class}"
                    width="{width|fill}"
                    padding="{padding|0}"
                    spacing="{spacing|12}"
                >
                    <slot name="{active}"/>
                </Column>"#
                .to_string(),
        )
    }

    fn update(&mut self, _a: &str, _v: Option<&str>, _c: &mut Context) {
        // Nada, e desta vez literalmente: não há nada em que clicar.
    }
}
