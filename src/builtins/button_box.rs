/// `QDialogButtonBox`: a fileira de botões de um formulário, com **papéis** e a
/// ordem decidida pela plataforma.
///
/// Existia desde sempre dentro de `src/dialogs.rs` (é o que desenha os botões
/// de um `QMessageBox`), mas só como parte do diálogo. Com o `<slot/>` (0.65)
/// ele vira widget de tela — que é onde a maior parte dos formulários vive.
///
/// ```xml
/// <buttonbox
///     accept="Salvar"      on_accept="salvar"
///     reject="Cancelar"    on_reject="cancelar"
///     destructive="Excluir" on_destructive="excluir"
/// >
///     <!-- o slot vai à ESQUERDA, longe das ações: é o "Ajuda"/"Restaurar
///          padrões" do Qt, que não deve encostar no botão principal -->
///     <button text="Ajuda" on_click="ajuda" padding="8 16" />
/// </buttonbox>
/// ```
///
/// # A ordem é da plataforma, não da tela
///
/// É a razão de o widget existir, e a razão de o Qt ter um `QDialogButtonBox`
/// em vez de um `QHBoxLayout` com dois botões: **onde** fica o botão principal
/// é convenção do sistema, não decisão de quem escreve a tela.
///
/// - **GNOME/macOS** (o default aqui): `[Excluir] … [Cancelar] [Salvar]` — o
///   afirmativo por último, encostado na borda direita.
/// - **Windows**: `[Excluir] … [Salvar] [Cancelar]` — o afirmativo primeiro.
///
/// O widget escolhe por `cfg!(target_os = "windows")` **em Rust**, no
/// [`Component::template`] — que é uma função, não uma constante, e portanto
/// pode montar markup diferente por alvo de compilação. Uma prop `order`
/// (`gnome` / `windows`) força uma das duas quando a tela tem motivo para isso
/// (um app que imita outro, uma captura de tela para documentação).
///
/// # Os papéis
///
/// Os mesmos três do [`crate::dialogs::ButtonRole`], e com o mesmo significado
/// visual — o que muda é só de onde sai a cor (lá, da paleta em Rust; aqui, de
/// uma folha global que o app pode repintar):
///
/// | papel | quando | aparência |
/// |---|---|---|
/// | `accept` | OK, Salvar, Sim | destaque (cor primária) |
/// | `reject` | Cancelar, Não, Fechar | discreto |
/// | `destructive` | Excluir, Descartar | perigo (vermelho), e **longe** dos outros |
///
/// O destrutivo fica na ponta oposta em qualquer ordem: é a única separação
/// que impede um clique errado de ser irreversível.
///
/// # Props
///
/// - `accept` / `reject` / `destructive` — os rótulos. **Um botão sem rótulo
///   não aparece**, então uma caixa só com `accept` é um botão só.
/// - `on_accept` / `on_reject` / `on_destructive` — as ações, entregues ao app
///   pelo prefixo `app:` (ver [`crate::eval::APP_ACTION_PREFIX`]).
/// - `order` — `gnome` ou `windows`. Default: o alvo de compilação.
/// - `spacing` — espaço entre os botões. Default: `8`.
/// - `padding` — área de clique de cada botão. Default: `8 16`.
use crate::component::{Component, Context, Template};

pub struct ButtonBox;

/// Os três botões, cada um só existindo quando tem rótulo. Separados numa
/// função porque as duas ordens usam exatamente os mesmos três blocos — a
/// única diferença entre as plataformas é a sequência em que eles entram.
fn botao(papel: &str, classe: &str) -> String {
    format!(
        r#"<template if="{{{papel}}}" notEquals="">
                    <Button
                        class="{classe}"
                        on_click="app:{{on_{papel}}}"
                        padding="{{padding|8 16}}"
                    >
                        <Text content="{{{papel}}}" size="13" bold="true" />
                    </Button>
                </template>"#
    )
}

impl Component for ButtonBox {
    fn name(&self) -> &str {
        "ButtonBox"
    }

    fn template(&self) -> Template {
        let aceitar = botao("accept", "bbox-accept");
        let recusar = botao("reject", "bbox-reject");
        let destrutivo = botao("destructive", "bbox-destructive");

        // A ordem por plataforma, decidida aqui e não na tela. `template()` é
        // uma função Rust, então o `cfg!` funciona — é o mesmo mecanismo que
        // faria um builtin variar por feature, e o único ponto da biblioteca
        // onde markup depende do alvo.
        let windows_por_padrao = cfg!(target_os = "windows");
        let (primeiro, segundo) = (&aceitar, &recusar);
        let (win_a, win_b) = (primeiro, segundo);
        let (gnome_a, gnome_b) = (&recusar, &aceitar);

        let ordem_win = format!("{win_a}\n{win_b}");
        let ordem_gnome = format!("{gnome_a}\n{gnome_b}");
        let (padrao, alternativa, forca_alternativa) = if windows_por_padrao {
            (&ordem_win, &ordem_gnome, "gnome")
        } else {
            (&ordem_gnome, &ordem_win, "windows")
        };

        Template::Inline(format!(
            r#"<Row spacing="{{spacing|8}}" align_y="center" width="{{width|fill}}">
                    <style>
                        .bbox-accept {{
                            color: #89b4fa;
                            text-color: #11111b;
                            border-width: 0;
                            border-radius: 6;
                        }}
                        .bbox-accept:hover {{ background: #a6c8ff; }}
                        .bbox-reject {{
                            color: #8080803d;
                            text-color: #cdd6f4;
                            border-width: 0;
                            border-radius: 6;
                        }}
                        .bbox-reject:hover {{ background: #80808066; }}
                        .bbox-destructive {{
                            color: #f38ba8;
                            text-color: #11111b;
                            border-width: 0;
                            border-radius: 6;
                        }}
                        .bbox-destructive:hover {{ background: #ff9fb8; }}
                    </style>

                    <!-- O destrutivo e o conteúdo do slot ficam na esquerda,
                         separados das ações por um <Space/>: é a única
                         separação que impede um clique errado de ser
                         irreversível. -->
                    {destrutivo}
                    <slot/>
                    <Space />

                    <template if="{{order|}}" equals="{forca_alternativa}">
                        {alternativa}
                    </template>
                    <template else>
                        {padrao}
                    </template>
                </Row>"#
        ))
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {
        // Delegante: os três cliques são do app, entregues pelo prefixo `app:`.
        // O widget não tem estado nenhum — o que ele decide é layout.
    }
}
