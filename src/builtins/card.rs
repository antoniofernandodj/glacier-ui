/// O cartão: uma superfície elevada que agrupa um assunto — cabeçalho opcional
/// (título e subtítulo) e o conteúdo que quem usa escreve dentro.
///
/// ```xml
/// <Card title="Servidor" subtitle="produção">
///     <Text content="uptime 31 dias" />
///     <Button text="Reiniciar" on_click="reiniciar" />
/// </Card>
/// ```
///
/// # Por que ele só nasce agora
///
/// O `PLANO_WIDGETS.md` listava o `Card` como pronto desde a 0.35, mas o que
/// existia era um **componente do app** (`examples/perfil/perfil_card.gv`) —
/// um cartão de perfil específico, com a imagem e os campos daquele exemplo
/// cravados no template. Cartão de verdade é aquilo que embrulha conteúdo
/// arbitrário, e isso dependia do `<slot/>`. Corrigido na mesma leva.
///
/// # Card, Frame e GroupBox
///
/// Os três desenham uma caixa; o que muda é o que cada um afirma:
///
/// - [`super::frame::Frame`] — só a caixa. Sem cabeçalho, sem opinião.
/// - [`super::group_box::GroupBox`] — *"estes controles são um grupo"*: título
///   discreto e moldura fina, para dentro de um formulário.
/// - `Card` — *"isto é um item"*: superfície com fundo próprio, cantos mais
///   redondos e um cabeçalho com hierarquia (título forte, subtítulo apagado).
///   É a peça de uma lista/grade de itens, não de um formulário.
///
/// # Props
///
/// - `title`    — título do cabeçalho. Vazio = sem cabeçalho.
/// - `subtitle` — linha secundária, sob o título. Vazio = ausente.
/// - `padding`  — espaço interno. Default: `16`.
/// - `spacing`  — espaço entre cabeçalho e corpo, e entre os filhos do corpo.
///   Default: `12`.
/// - `width`    — largura. Default: `fill` (num `<Row>` de cartões, dar
///   `width="280"` a cada um é o caminho para a grade).
/// - `title_size` / `subtitle_size` — corpos. Default: `16` / `13`.
///
/// # Limite conhecido
///
/// **Sem rodapé.** A faixa de ações no pé de um cartão precisaria de um
/// **segundo** buraco no template, e o `<slot/>` de hoje é único e anônimo —
/// dois slots exigiriam slot nomeado (`<slot name="footer"/>`), o degrau
/// seguinte. Enquanto isso, o rodapé é uma `<Row>` no fim do conteúdo.
use crate::component::{Component, Context, Template};

pub struct Card;

impl Component for Card {
    fn name(&self) -> &str {
        "Card"
    }

    fn template(&self) -> Template {
        // Cabeçalho e subtítulo são independentes: um cartão pode ter só
        // título, e um subtítulo sem título é legítimo (uma etiqueta solta).
        // Por isso são dois `<template if>` irmãos, não um aninhado.
        Template::Inline(
            r#"<Container
                    class="card-surface"
                    padding="{padding|16}"
                    width="{width|fill}"
                >
                    <style>
                        .card-surface {
                            background: #8080801f;
                            border-width: 1;
                            border-color: #80808040;
                            border-radius: 10;
                        }
                        .card-subtitle { color: #80868d; }
                    </style>

                    <Column spacing="{spacing|12}" width="{width|fill}">
                        <template if="{title}{subtitle}" notEquals="">
                            <Column spacing="2" width="{width|fill}">
                                <template if="{title}" notEquals="">
                                    <Text
                                        content="{title}"
                                        size="{title_size|16}"
                                        bold="true"
                                    />
                                </template>
                                <template if="{subtitle}" notEquals="">
                                    <Text
                                        class="card-subtitle"
                                        content="{subtitle}"
                                        size="{subtitle_size|13}"
                                    />
                                </template>
                            </Column>
                        </template>

                        <Column spacing="{spacing|12}" width="{width|fill}">
                            <slot/>
                        </Column>
                    </Column>
                </Container>"#
                .to_string(),
        )
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {
        // Recipiente puro.
    }
}
