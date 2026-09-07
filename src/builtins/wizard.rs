/// `Wizard` / `QWizard`: os passos com voltar, avançar e finalizar.
///
/// ```xml
/// <wizard value="passo" active="{passo}"
///         steps="dados,pagamento,revisao"
///         titles="Seus dados,Pagamento,Revisão"
///         valid="{pode_avancar}"
///         on_finish="salvar_pedido">
///     <template slot="dados">     … </template>
///     <template slot="pagamento"> … </template>
///     <template slot="revisao">   … </template>
/// </wizard>
/// ```
///
/// # Ele é a soma desta onda
///
/// No Qt o `QWizard` **é** um `QDialog` — e aqui isso deixou de ser trivia e
/// virou o desenho: com o corpo em markup (habilitador A), um wizard é o
/// `<dialog>` do item 1 com este widget dentro, e nada mais precisou existir.
/// As peças são todas anteriores:
///
/// - as páginas são o [`super::stack_view::StackView`] (item 5), que por sua
///   vez é o nome dinâmico de slot da Onda 5;
/// - a fileira de botões é o `<buttonbox>` da Onda 4, com a ordem por
///   plataforma que ele já resolve;
/// - o passo atual mora numa **chave nomeada**, o padrão do `SpinBox` (0.85).
///
/// O catálogo marcava o `Wizard` como `Comp ●`, "precisa de estado por
/// instância". Não precisa — mas repare que aqui o argumento é o *outro*: não é
/// que o widget seja singleton (ele não é, dá para ter dois na mesma tela), é
/// que o passo é **o valor**, e valor sempre coube numa chave que o app nomeia.
/// É a mesma correção do `<tabbar>`, do `<pagination>` e do `<dial>`.
///
/// # A lógica que ele carrega
///
/// É o item desta onda que tem `update` de verdade, e são quatro regras que
/// todo wizard do mundo tem e que ninguém quer reescrever:
///
/// 1. **Voltar fica inerte no primeiro passo** — não some, fica inerte: um
///    botão que aparece e desaparece faz a fileira dançar a cada passo.
/// 2. **Avançar vira Finalizar no último**, e é o `on_finish` que ele dispara.
/// 3. **Avançar trava enquanto a página não valida** (`valid="{...}"`), que é o
///    `QWizardPage::isComplete()`. Sem a prop, todo passo é válido.
/// 4. **Andar não pula o fim nem o começo**: saturar é sempre melhor do que
///    dar a volta, porque um wizard que volta do último para o primeiro parece
///    ter perdido o trabalho do usuário.
///
/// # Props
///
/// - `value`     — **obrigatória**: o nome da chave que guarda o passo atual.
/// - `active`    — o valor atual dessa chave (o par `value`/`active` de novo).
/// - `steps`     — **obrigatória**: os ids dos passos, separados por vírgula.
///   São eles que casam com o `slot=` de cada `<template>`.
/// - `titles`    — os rótulos do cabeçalho, na mesma ordem. Sem eles, o
///   cabeçalho mostra os ids.
/// - `valid`     — enquanto for falso, `Avançar`/`Finalizar` fica inerte.
/// - `on_finish` — a ação disparada pelo botão do último passo.
/// - `on_cancel` — a ação do botão Cancelar. Sem ela, não há Cancelar.
/// - `back_label` / `next_label` / `finish_label` — os rótulos, para trocar de
///   idioma sem reescrever o widget.
use crate::component::{Component, Context, Template};

pub struct Wizard;

impl Component for Wizard {
    fn name(&self) -> &str {
        "Wizard"
    }

    fn template(&self) -> Template {
        // As páginas primeiro, a navegação depois — a ordem do `QWizard`, e a
        // única em que os botões são a última coisa que a leitura encontra.
        //
        // O cabeçalho ("Passo 2 de 3") vem da primitiva junto com os botões, e
        // não daqui, porque ele depende da MESMA conta: achar `active` dentro
        // de `steps`. Fazer a conta duas vezes seria dois lugares para ela
        // divergir.
        //
        // # Por que a página é montada aqui, e não por um `<StackView>`
        //
        // Seria a composição óbvia — e não funciona, por uma regra do
        // `<slot/>` que vale a pena ter escrita em algum lugar: **um slot não
        // atravessa a fronteira de um componente aninhado**. A partição
        // acontece uma vez, sobre os filhos crus de QUEM ESCREVEU a tag; um
        // `<StackView><slot name="dados"/></StackView>` entrega ao StackView um
        // filho já resolvido, sem etiqueta, e o `<slot name="{active}"/>` do
        // template dele não acha etiqueta nenhuma para casar. O resultado é uma
        // página em branco, sem erro.
        //
        // Então o wizard monta a coluna da página ele mesmo — as mesmas quatro
        // linhas do `<StackView>`, que continua existindo para quem troca de
        // página sem ser um wizard.
        Template::Inline(
            r#"<Column spacing="{spacing|14}" width="{width|fill}">
                    <Column
                        class="stackview-page {page_class}"
                        width="fill"
                        padding="{padding|0}"
                        spacing="{page_spacing|12}"
                    >
                        <slot name="{active}"/>
                    </Column>

                    <WizardNav
                        value="{value}"
                        steps="{steps}"
                        titles="{titles}"
                        valid="{valid}"
                        on_finish="{on_finish}"
                        on_cancel="{on_cancel}"
                        back_label="{back_label}"
                        next_label="{next_label}"
                        finish_label="{finish_label}"
                        cancel_label="{cancel_label}"
                        header="{header|true}"
                    />
                </Column>"#
                .to_string(),
        )
    }

    fn update(&mut self, _a: &str, _v: Option<&str>, _c: &mut Context) {
        // Nada: quem anda entre os passos é a `<WizardNav>`, e ela escreve a
        // chave sozinha pelo `ContextPatch` — o mesmo caminho da
        // `<pagination>`. Este componente é a composição páginas+navegação, e
        // composição não tem estado.
    }
}
