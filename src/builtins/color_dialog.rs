/// `ColorDialog` / `QColorDialog`: o **corpo** do diálogo de escolher cor.
///
/// ```lua
/// local cor = pick_color({ title = "Cor do rótulo", value = ctx.cor_atual })
/// if cor then ctx.cor_atual = cor end
/// ```
///
/// # O que ele deve à Onda 7
///
/// Este item estava na proposta da Onda 7 e ficou de fora por tamanho. O que
/// mudou é que a onda deixou pronto o que faltava: a roda desenha sobre
/// [`crate::canvas`] (ver [`crate::color_picker`]), e o que sobra aqui são
/// quatro linhas de markup. É o primeiro consumidor do **corpo em markup** que
/// não caberia em Rust sem duplicar meia dúzia de widgets — desenhar a roda, o
/// campo e a amostra numa função de `dialogs.rs` seria escrever o motor de
/// novo, do lado de fora dele.
///
/// # Os três painéis escrevem a MESMA chave
///
/// A roda, o campo hexadecimal e a amostra são três vistas de
/// [`crate::dialogs::DIALOG_VALUE_KEY`]. Nenhum deles guarda cor: mexer na roda
/// reescreve a chave, e o campo e a amostra mudam porque leem dela. É o que faz
/// os três nunca discordarem — não há sincronização a escrever, porque não há
/// duas cópias.
///
/// # O rascunho do campo hexadecimal
///
/// Há **duas** chaves, e a separação existe por uma razão só: enquanto alguém
/// digita `#ff8800`, o texto passa por `#f`, `#ff`, `#ff8` — valores que não
/// são cor. Se o campo escrevesse direto na chave da cor, a roda leria isso
/// como branco e piscaria a cada tecla.
///
/// - `__dialog.value` — a **cor cometida**, em `#rrggbb`. É o que a roda lê e
///   escreve, e o que o aceite devolve à corrotina.
/// - `__dialog.value__hex` — o **texto em digitação**. O campo edita esta, e
///   a roda a reescreve a cada gesto (ver [`crate::color_picker`]), que é o que
///   faz o campo seguir a roda.
///
/// O caminho de volta passa por [`crate::color_picker::hex_completo`]: só um
/// hexadecimal **inteiro** (`#rgb` ou `#rrggbb`) comete a cor. Texto pela
/// metade fica no rascunho e não mexe na roda — que é exatamente o que se quer
/// enquanto a mão ainda está digitando.
///
/// O usuário pode, portanto, deixar o campo num estado inválido e apertar OK: a
/// resposta é a última cor **válida**, não o texto quebrado. É a mesma escolha
/// que um `QColorDialog` faz, e a única que não obriga a validar do lado de
/// quem chamou o `pick_color{}`.
use crate::component::{Component, Context, Template};

/// O nome sob o qual o motor monta este corpo.
pub const COLOR_DIALOG_BODY: &str = "__ColorDialog";

/// A chave do **texto em digitação** do campo hexadecimal — o rascunho que a
/// roda não lê. Segue a convenção de chave irmã do
/// [`crate::color_picker`]: `<chave da cor>__hex`.
pub const HEX_KEY: &str = "__dialog.value__hex";

pub struct ColorDialog;

impl Component for ColorDialog {
    fn name(&self) -> &str {
        COLOR_DIALOG_BODY
    }

    fn template(&self) -> Template {
        Template::Inline(
            r##"<Column spacing="12" width="fill">
                    <Row width="fill">
                        <Space width="fill" />
                        <ColorWheel value="__dialog.value" size="{__dialog.size|220}" />
                        <Space width="fill" />
                    </Row>

                    <Row spacing="10" width="fill">
                        <!-- A amostra: o único lugar onde a cor aparece
                             sozinha, sem a roda em volta dela. -->
                        <Container
                            width="44"
                            height="32"
                            background="{__dialog.value}"
                            border_radius="6"
                            border_width="1"
                        />
                        <TextInput
                            value="__dialog.value__hex"
                            placeholder="#rrggbb"
                            onChange="__ColorDialog::hex"
                            width="fill"
                        />
                    </Row>
                </Column>"##
                .to_string(),
        )
    }

    fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context) {
        // O namespace vem escrito no template pelo mesmo motivo do
        // `__InputDialog`: este corpo é montado como template de topo, não
        // inlinado numa tela, então a avaliação não tem dono para prefixar.
        if action != "hex" {
            return;
        }
        let Some(digitado) = value else { return };

        // O rascunho guarda o que foi digitado, inteiro e sem julgamento — é o
        // que o campo mostra, e apagá-lo ou "corrigi-lo" aqui faria o cursor
        // pular enquanto a pessoa escreve.
        ctx.set(HEX_KEY, digitado);

        // A cor só é cometida quando o texto vira uma cor. Enquanto não vira, a
        // roda continua na última válida — em vez de piscar branco.
        if let Some(hex) = crate::color_picker::hex_completo(digitado) {
            ctx.set(crate::dialogs::DIALOG_VALUE_KEY, hex);
        }
    }
}
