/// `ProgressDialog` / `QProgressDialog`: o **corpo** do diálogo que acompanha
/// uma tarefa longa.
///
/// ```lua
/// progress({ title = "Baixando", label = "conectando…", max = 100,
///            cancel_label = "Cancelar", on_cancel = "cancelar_download" })
/// -- … no meio do trabalho:
/// progress_set(42, "baixando pacote 42 de 100")
/// -- … no fim:
/// progress_close()
/// ```
///
/// # O único da família que é atualizado enquanto está aberto
///
/// Os outros diálogos da Onda 8 são perguntas: aparecem, esperam, somem. Este
/// é uma janela sobre um trabalho em curso, e muda dezenas de vezes por
/// segundo — o que muda o desenho dele por completo.
///
/// A tentação é pôr o progresso no [`crate::dialogs::DialogSpec`] e reexibir o
/// diálogo a cada tique. Isso reconstrói a especificação e o cartão inteiros a
/// cada 1% — e `GlacierUI::show_dialog` **substitui** o diálogo, então cada
/// atualização é um diálogo novo no lugar do anterior. Com o progresso numa
/// **chave**, o `spec` nunca muda: o número anda pelo mesmo caminho de qualquer
/// outro valor do motor, e o `<progressbar>` o lê como leria o de uma tela.
///
/// É o argumento A da onda pagando de novo — o corpo em markup não é só
/// conveniência de escrita, é o que faz este widget não ser um caso especial.
///
/// # O cancelamento é uma ação, não um mecanismo novo
///
/// O `QProgressDialog::wasCanceled()` é um `bool` que o laço consulta. Aqui o
/// botão despacha uma **ação** como qualquer outro botão (`on_cancel`), e o
/// handler dela faz o que o app quiser — parar o laço, fechar o stream, marcar
/// uma chave. Sem `on_cancel`, o diálogo não tem botão nenhum e só sai pelo
/// `progress_close()`: é o `QProgressDialog` sem cancelamento, e é o certo para
/// uma etapa que não dá para interromper.
///
/// # Props (por chave, como no [`super::input_dialog::InputDialog`])
///
/// - `__dialog.progresso` — o valor **cru**, como o app o escreveu. Vazio =
///   indeterminado. É o que o rótulo e os testes leem.
/// - `__dialog.pct`       — o mesmo valor na escala 0–100, que é o que a barra
///   desenha (ver o comentário no template para o porquê).
/// - `__dialog.max`       — o total. Default `100`.
/// - `__dialog.label`     — a linha de texto acima da barra.
///
/// Um progresso **indeterminado** (sem `__dialog.progresso`) mostra o
/// `<spinner>` da 0.66 no lugar da barra — as duas metades do
/// `QProgressBar`/`QProgressDialog`, e a razão de o `Spinner` já existir.
use crate::component::{Component, Context, Template};

/// O nome sob o qual o motor monta este corpo. `__` porque é peça interna, não
/// tag para escrever numa tela.
pub const PROGRESS_DIALOG_BODY: &str = "__ProgressDialog";

pub struct ProgressDialog;

impl Component for ProgressDialog {
    fn name(&self) -> &str {
        PROGRESS_DIALOG_BODY
    }

    fn template(&self) -> Template {
        Template::Inline(
            r#"<Column spacing="10" width="fill">
                    <!-- `not_equals=""` e NÃO `not_empty`: no motor, `empty`/
                         `not_empty` perguntam se o valor é um **array JSON
                         vazio** (é o teste que o `<listview>` usa), não se a
                         string está em branco. Um texto comum nunca é um array
                         válido, então `not_empty` daria sempre falso — e o
                         ramo simplesmente não apareceria, sem erro nenhum. -->
                    <se cond="{__dialog.label}" not_equals="">
                        <Text content="{__dialog.label}" size="13" />
                    </se>

                    <!-- Duas coisas que o `<progressbar>` impõe, e as duas
                         mordem em silêncio:

                         1. `value` é o **nome de uma chave**, não uma
                            interpolação — `value="{x}"` faria o widget procurar
                            uma chave chamada "42" e desenhar zero;
                         2. `min`/`max` são numéricos LITERais: ao contrário de
                            `size`/`spacing`, eles não entram no
                            `numeric_templates`, então `max="{...}"` não resolve
                            e cai no default.

                         Daí a escala fixa 0–100 e a chave `__dialog.pct`, que o
                         `progress_set` calcula a partir do `max` declarado na
                         abertura. O valor cru continua em `__dialog.progresso`,
                         que é o que o rótulo e os testes leem. -->
                    <se cond="{__dialog.pct}" not_equals="">
                        <ProgressBar value="__dialog.pct" min="0" max="100" width="fill" />
                    </se>
                    <senao>
                        <Row width="fill" spacing="10">
                            <Spinner size="18" />
                            <Text content="{__dialog.indeterminado|Trabalhando…}" size="13" />
                        </Row>
                    </senao>
                </Column>"#
                .to_string(),
        )
    }

    fn update(&mut self, _a: &str, _v: Option<&str>, _c: &mut Context) {
        // Nada: a barra lê a chave, e quem escreve a chave é o app.
    }
}
