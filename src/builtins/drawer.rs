/// `Drawer`: o painel lateral que desliza para dentro e para fora da tela.
///
/// ```xml
/// <row width="fill" height="fill">
///     <drawer value="menu" open="{menu}" size="260">
///         <button text="Painel"   on_click="ir:painel" width="fill" />
///         <button text="Serviços" on_click="ir:servicos" width="fill" />
///     </drawer>
///
///     <column width="fill"> … o conteúdo da tela … </column>
/// </row>
/// ```
///
/// # Por que ele está na Onda 5 sem precisar da Onda 5
///
/// Este é o item 6 da onda, e o plano já o marcava assim: *"não precisa de
/// nenhum dos dois habilitadores — é `<slot/>` (0.65) + uma chave nomeada + a
/// animação que o motor já tem; entra aqui por parentesco, não por bloqueio."*
///
/// O parentesco é a pergunta da onda inteira — *de quem é este conteúdo?* — e a
/// resposta aqui é a mesma do `<popover>`: do widget, não da tela. A diferença
/// é que a gaveta **empurra** em vez de flutuar, e por isso ela não é um
/// overlay: é um filho de verdade da `<row>`, com largura animada.
///
/// A única peça que faltava era o eixo: o `<Reveal>` (0.90) animava só a
/// altura. `axis="x"` (Onda 5) é o mesmo mecanismo na largura — ver
/// [`crate::reveal`].
///
/// # Empurra, não cobre
///
/// Escolha deliberada, e a que distingue esta gaveta de um `<popover>`: o
/// conteúdo ao lado **encolhe** quando ela abre, como o painel lateral de um
/// IDE. Uma gaveta que cobre a tela é um `<popover>` colado na borda, e o
/// motor já tem um.
///
/// Consequência prática: a `<row>` que segura a gaveta e o conteúdo é do APP,
/// não do widget. Sem ela, a gaveta abre empurrando para baixo em vez de para
/// o lado — e é também por isso que **não há prop `side`**: a gaveta fica do
/// lado em que o markup a escreveu, antes ou depois do conteúdo na `<row>`.
/// Uma prop que duplicasse essa escolha só poderia contradizê-la.
///
/// # `value` e `open` andam em par
///
/// Como no `<accordionitem>`, e pela mesma razão: `value` é o **nome** da chave
/// que guarda o estado; `open` é o valor atual dela. Clicar no botão de
/// fechar (ou em qualquer `<button on_click="app:...">` que alterne a chave)
/// abre e fecha.
///
/// O widget **não** desenha o gatilho: um botão de menu mora na barra de
/// título, no cabeçalho ou num `<toolbutton>`, e nenhum desses lugares é
/// dentro da gaveta. Quem abre escreve `on_click="toggle:menu"` — a ação que o
/// `update` abaixo trata, e que funciona de qualquer lugar da tela porque o
/// nome da chave viaja com ela.
///
/// # Props
///
/// - `value`    — **obrigatória**: nome da chave com o aberto/fechado.
/// - `open`     — o valor atual dessa chave.
/// - `size`     — largura da gaveta aberta, em pixels. Default `240`.
/// - `duration` — duração do deslize em ms. Default `180`; `0` desliga.
/// - `padding`  — espaço interno. Default `12`.
/// - `spacing`  — espaço entre os filhos. Default `8`.
///
/// # Aparência
///
/// `.drawer-panel` na folha global do template, mais `panel_class` para o app
/// alcançar o painel — o padrão de classe por nó interno da 0.89.
use crate::component::{Component, Context, Template};

pub struct Drawer;

impl Component for Drawer {
    fn name(&self) -> &str {
        "Drawer"
    }

    fn template(&self) -> Template {
        // A largura fixa mora no filho do `<Reveal>`, não no `<Reveal>`: é a
        // largura NATURAL que a animação interpola de 0 até ela. Um `Reveal`
        // com `width` declarada mediria a si mesmo em vez do filho, e a gaveta
        // abriria de estalo na largura toda.
        //
        // `height="fill"` no painel é o que faz a gaveta ir do topo ao rodapé
        // da `<row>` do app — sem isso ela teria a altura do conteúdo dela, e
        // uma gaveta de dois botões seria uma tira de 80px.
        Template::Inline(
            r#"<Reveal
                    open="{open}"
                    duration="{duration|180}"
                    axis="x"
                    height="fill"
                >
                    <style>
                        .drawer-panel { border-color: #8080803D; }
                    </style>

                    <Column
                        class="drawer-panel {panel_class}"
                        width="{size|240}"
                        height="fill"
                        padding="{padding|12}"
                        spacing="{spacing|8}"
                    >
                        <slot/>
                    </Column>
                </Reveal>"#
                .to_string(),
        )
    }

    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        // `toggle:menu` — a chave que o app nomeou. Escrita de QUALQUER lugar
        // da tela (o botão que abre a gaveta quase nunca está dentro dela), e é
        // por isso que a ação carrega a chave em vez de o widget guardá-la: é o
        // padrão do `SpinBox`, e o que faz duas gavetas na mesma tela não
        // colidirem.
        let Some(("toggle", chave)) = action.split_once(':') else {
            return;
        };
        let chave = chave.trim();
        if chave.is_empty() {
            return;
        }
        let aberta = ctx
            .get(chave)
            .is_some_and(|v| crate::widget::is_truthy(v.trim()));
        ctx.set(
            chave,
            if aberta {
                String::new()
            } else {
                "true".to_string()
            },
        );
    }
}
