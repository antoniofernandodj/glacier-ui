/// `InputDialog` / `QInputDialog`: o **corpo** do diálogo que pede um valor.
///
/// ```lua
/// local nome = prompt{ title = "Renomear", label = "Novo nome", value = servico }
/// if nome then rename(nome) end
/// ```
///
/// # As quatro variantes do Qt são um diálogo só
///
/// O `QInputDialog` tem quatro entradas estáticas — `getText`, `getInt`,
/// `getDouble`, `getItem` — e elas diferem em **uma** coisa: o widget do campo.
/// A moldura, o par de botões, a validação e o retorno são idênticos nas
/// quatro. Aqui isso vira uma tag com um `kind`, e o `se`/`senao` escolhe entre
/// três widgets que já existiam:
///
/// | `kind`            | widget            |
/// |-------------------|-------------------|
/// | `text` (default)  | `<TextInput>`     |
/// | `int` / `double`  | `<SpinBox>`       |
/// | `item`            | `<Select>`        |
///
/// # Por que ele lê chaves globais em vez de props
///
/// Este builtin é o único da biblioteca que **não** é usado escrevendo a tag
/// dele numa tela: quem o monta é o motor, como corpo de um `DialogSpec`, a
/// partir de um `prompt{}` da camada Luau. Não há um uso de tag onde pendurar
/// props, então a configuração chega por onde ela pode chegar — chaves de
/// contexto que o motor semeia antes de abrir (ver
/// [`crate::dialogs::DIALOG_KEY_PREFIX`]).
///
/// É a mesma escolha que o `<datetimeedit>` fez com o `__timeedit` na 0.70, e
/// pelo mesmo motivo: o widget não existe como nó que alguém escreveu, então
/// não há atributo para ler.
///
/// # O valor mora numa chave, e é por isso que ele não é `●`
///
/// O catálogo marcava o `InputDialog` como "exige estado por instância". Não
/// exige, e aqui a marca é estruturalmente impossível: o diálogo é singleton no
/// motor (`GlacierUI::dialog`), então nunca existe uma segunda instância com
/// que colidir. O que o usuário digita mora em
/// [`crate::dialogs::DIALOG_VALUE_KEY`] enquanto ele digita, e o aceite lê de
/// lá — o padrão do `SpinBox` (0.85), pela quarta vez.
use crate::component::{Component, Context, Template};

pub struct InputDialog;

/// O nome sob o qual o motor monta este corpo. Começa com `__` porque não é
/// para ser escrito numa tela: é uma peça interna, e o nome diz isso.
pub const INPUT_DIALOG_BODY: &str = "__InputDialog";

impl Component for InputDialog {
    fn name(&self) -> &str {
        INPUT_DIALOG_BODY
    }

    fn template(&self) -> Template {
        // O `label` some quando não foi pedido — um rótulo vazio ocupa a altura
        // de uma linha, e o buraco entre o título do diálogo e o campo denuncia
        // (a mesma razão pela qual o cartão esconde a mensagem vazia).
        //
        // # Por que o `onChange` traz o nome do componente escrito à mão
        //
        // Uma ação num template de componente normalmente ganha o namespace do
        // dono na avaliação (`editar` → `__InputDialog::editar`). Este corpo é
        // a exceção: ele é montado por `GlacierUI::render(nome)` como template
        // **de topo**, não inlinado numa tela, então não há dono e a ação sai
        // nua — e uma ação nua é roteada para a tela ativa, que não a trata.
        //
        // O sintoma disso é discreto e caro: o campo mostra o valor inicial,
        // aceita digitação e **não guarda nada**. Escrever o namespace aqui é o
        // preço de o corpo do diálogo não ser um uso de tag como os outros.
        Template::Inline(
            r#"<Column spacing="8" width="fill">
                    <se cond="{__dialog.label}" not_empty="true">
                        <Text content="{__dialog.label}" size="13" />
                    </se>

                    <se cond="{__dialog.kind}" one_of="int,double">
                        <SpinBox
                            value="__dialog.value"
                            min="{__dialog.min|0}"
                            max="{__dialog.max|100}"
                            step="{__dialog.step|1}"
                            decimals="{__dialog.decimals|0}"
                            width="fill"
                        />
                    </se>
                    <senaose cond="{__dialog.kind}" equals="item">
                        <Select
                            options="__dialog.items"
                            value="__dialog.value"
                            onChange="__InputDialog::editar"
                            width="fill"
                        />
                    </senaose>
                    <senao>
                        <TextInput
                            value="__dialog.value"
                            placeholder="{__dialog.placeholder}"
                            onChange="__InputDialog::editar"
                            width="fill"
                        />
                    </senao>
                </Column>"#
                .to_string(),
        )
    }

    fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context) {
        // Um `<TextInput>`/`<Select>` **não grava a chave sozinho**: ele
        // despacha `onChange` com o texto novo, e quem escreve é quem trata a
        // ação. Sem este `update` o campo seria decorativo — mostra o valor
        // inicial, aceita digitação e não guarda nada, e o aceite devolveria o
        // valor de antes.
        //
        // O `<SpinBox>` do ramo numérico não passa por aqui: ele é builtin e
        // escreve a própria chave (`spin_box.rs`), que é a razão de o template
        // acima não lhe dar `onChange`.
        if action == "editar"
            && let Some(v) = value
        {
            ctx.set(crate::dialogs::DIALOG_VALUE_KEY, v);
        }
    }
}
