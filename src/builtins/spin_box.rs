/// `QSpinBox`: um campo numérico com os degraus ▴▾ encostados nele — clicar
/// soma ou subtrai `step`, saturando em `min`/`max`.
///
/// É o primeiro builtin com **comportamento próprio** (os anteriores só montam
/// markup): a aritmética roda no `update` abaixo, em Rust, sem o app escrever
/// uma linha de Lua.
///
/// # Como ele tem estado sem ter estado
///
/// O `ctx` de um builtin é o contexto **global** — não há um slot por instância
/// (ver o módulo pai). O `SpinBox` contorna isso não guardando valor nenhum: o
/// número vive numa chave que **o app nomeia**, passada na prop `value`. Duas
/// instâncias com chaves diferentes (`<SpinBox value="qtd_a"/>` e
/// `<SpinBox value="qtd_b"/>`) são independentes.
///
/// O elo que faltava é o `update` saber *qual* chave a instância clicada usa —
/// ele recebe a ação, não as props. Por isso a ação carrega os parâmetros:
/// o template escreve `on_click="inc:{value}|{min|0}|{max|100}|{step|1}|{decimals|}"`, o
/// eval interpola e prefixa o dono (`namespace_action`), e chega aqui como
/// `inc:qtd|1|99|1|2`. O motor devolve tudo isso ao dono certo porque um builtin
/// entra no mesmo mapa de componentes que os do app (`GlacierUI::route_to_owner`).
///
/// # As duas formas
///
/// A prop `layout` escolhe o desenho, e as duas existem no Qt:
///
/// - `stacked` (**default**) — o `QSpinBox` clássico: o campo e, colado à
///   direita dele, uma coluna com as duas setinhas (▴ em cima, ▾ embaixo).
///   Ocupa a altura do campo e nada mais; é a forma para formulário denso.
/// - `inline` — a forma do `SpinBox` do Qt Quick Controls: `−  campo  +`, os
///   dois degraus nas pontas, alvo de clique grande. É a forma para toque e
///   para valores que o usuário ajusta muito (zoom, quantidade num carrinho).
///
/// ```xml
/// <SpinBox value="quantidade" min="1" max="99" />
/// <SpinBox value="zoom" min="25" max="400" step="25" layout="inline" />
/// ```
///
/// # Aparência
///
/// Os degraus não são botões primários (não seriam o widget que o Qt desenha:
/// lá eles são cromo discreto ao lado do campo, não a ação principal da tela).
/// O visual sai de um `<style>` **global** declarado no próprio template — uma
/// folha instalada em `GlacierUI::new`, portanto **antes** de qualquer `.gss`
/// do app, que por isso vence por ordem: redefinir `.spinbox-step` numa folha
/// do app é o caminho suportado para repintá-los.
///
/// As cores dessa folha são de propósito **neutras e translúcidas**
/// (`#8080801f` de fundo, cinza médio na seta): um cinza com alfa clareia sobre
/// um tema escuro e escurece sobre um claro, então o mesmo default atravessa os
/// quatro estilos embutidos ([`crate::style`]) sem que o widget precise saber
/// qual está ativo — nenhum hex de paleta viaja no template.
///
/// # Props
///
/// - `value`       — **obrigatória**: nome da chave de contexto com o número.
/// - `min` / `max` — limites. Default: `0` / `100` (a faixa padrão do Qt).
/// - `step`        — passo de cada clique. Default: `1`.
/// - `decimals`    — casas decimais da saída, e é o que fecha o
///   `QDoubleSpinBox`. **Sem ela**, as casas saem do `step` como escrito no
///   markup (`step="0.25"` → 2 casas) — o que acerta na maioria dos casos e
///   erra justamente no que importa: `step="1"` sobre um preço formatava
///   `10`, não `10.00`. Com ela, o `step` volta a ser só o passo:
///   `<SpinBox value="preco" step="1" decimals="2" />`.
///
///   Nos dois caminhos a formatação também evita o `0.30000000000000004` de
///   somar `f64`, que é o outro serviço que o `QDoubleSpinBox` presta.
/// - `layout`      — `stacked` (default) ou `inline`; ver acima.
/// - `width`       — largura do campo. Default: `72`.
/// - `placeholder` — dica quando a chave está vazia. Default: vazio.
/// - `field_class` — classe aplicada **ao campo de dentro**, não ao widget.
/// - `step_class`  — classe dos dois botões de degrau (aceita `:hover`).
/// - `glyph_class` — classe do glifo dentro de cada degrau.
///   Não se chama `class` de propósito: desde a 0.69 `class` numa tag de
///   componente estiliza a **raiz expandida** — aqui, a `Row` inteira (campo +
///   degraus), que é o que estilizar um `QSpinBox` significa no Qt. As duas
///   coisas são legítimas e diferentes; colapsá-las no mesmo nome seria criar
///   uma ambiguidade que não dá para desfazer depois.
/// - `form_control` — repassado ao campo de dentro, o que o liga à `<Form>` que
///   o envolve: o campo ganha um id de foco estável e passa a responder ao
///   **Enter** — submetendo, e avançando para o controle seguinte. Sem ele o
///   campo engolia o Enter. Não confundir com **Tab**: a travessia por Tab é um
///   listener global do motor (`focus_next`) e já alcançava este campo antes,
///   com ou sem `form_control`. Sem prefixo `field_` porque não há ambiguidade:
///   só existe um nó focável aqui dentro.
///   Funciona porque a hidratação da `<Form>` roda **depois** da expansão de
///   componente (`eval.rs`), então ela enxerga um `formControl` que só passou a
///   existir na expansão.
/// - `dec_text` / `inc_text` — glifos dos degraus. Default: `▾` / `▴` no
///   `stacked`, `−` / `+` no `inline`.
/// - `glyph_size`  — corpo do glifo. Default: `11` no `stacked` (é a altura
///   dele, duplicada, que casa com a do campo), `15` no `inline`.
///
/// # Limites conhecidos
///
/// - **Sem `on_change` para o app.** O `onChange` do `<TextInput>` é um só, e o
///   `SpinBox` usa o dele para filtrar a digitação; repassar *também* ao app
///   exigiria um `ctx.dispatch` que o motor não tem (o prefixo `app:`, que
///   resolve o caso do widget que só delega — ver `TimePicker` —, não serve
///   aqui, porque ele entrega a ação em vez de duplicá-la). Quem precisa
///   reagir lê a chave.
/// - **Digitação não satura.** Enquanto se digita, o texto entra filtrado (só
///   dígitos, um `-` à frente e um `.`) mas livre — `120` num `max="99"` fica
///   `120` até o próximo clique num degrau, que satura. É o comportamento do
///   `QSpinBox`, que só valida no `editingFinished`.
/// - **Os degraus não desabilitam no limite.** Para isso o template precisaria
///   ler o *valor* da chave cujo *nome* está numa prop — uma indireção
///   (`{{value}}`) que o interpolador não tem. Clicar no limite não faz nada.
/// - **A altura casa por aritmética, não por estica.** O `iced` não estica o
///   conteúdo de um `<Button>`, então os dois degraus empilhados alcançam a
///   altura do campo por soma (2 × 1,3 × `glyph_size` ≈ a linha do campo mais o
///   padding dele). Um `glyph_size` muito fora do default desencontra os dois.
use crate::component::{Component, Context, Template};

pub struct SpinBox;

/// Casas decimais que a saída deve ter.
///
/// A prop `decimals` manda quando existe; sem ela, o número é **deduzido do
/// `step` como escrito no markup** (`"0.25"` → 2), que é o comportamento que o
/// widget sempre teve e que continua acertando o caso comum.
///
/// Os dois caminhos são limitados a 6: nem um `decimals="17"` nem um `step`
/// com lixo de ponto flutuante devem produzir um número absurdo de casas.
fn casas_decimais(step: &str, decimals: &str) -> usize {
    if let Some(n) = decimals
        .trim()
        .parse::<usize>()
        .ok()
        .filter(|_| !decimals.trim().is_empty())
    {
        return n.min(6);
    }
    match step.trim().split_once('.') {
        Some((_, decimais)) => decimais.trim().len().min(6),
        None => 0,
    }
}

/// Mantém só o que pode fazer parte de um número enquanto se digita: dígitos,
/// um `-` (na frente) e um `.` (o primeiro). O resto cai fora, então a chave
/// nunca guarda texto que o `parse::<f64>` não entenda — exceto os estados
/// intermediários legítimos (`""`, `"-"`, `"1."`), que a digitação exige.
fn filtra_numero(bruto: &str) -> String {
    let mut saida = String::with_capacity(bruto.len());
    let mut tem_ponto = false;
    for (i, c) in bruto.chars().enumerate() {
        match c {
            '-' if i == 0 => saida.push(c),
            '.' if !tem_ponto => {
                tem_ponto = true;
                saida.push(c);
            }
            c if c.is_ascii_digit() => saida.push(c),
            _ => {}
        }
    }
    saida
}

impl Component for SpinBox {
    fn name(&self) -> &str {
        "SpinBox"
    }

    fn template(&self) -> Template {
        // Os três `on_*` carregam o mesmo payload `chave|min|max|step` porque
        // o `update` não enxerga as props da instância — ver a docstring.
        // `{min|0}` é o default inline do interpolador (`|` dentro das chaves),
        // enquanto o `|` de fora é literal e separa os campos do payload.
        //
        // `spacing="0"` na `<Row>` é o que faz os degraus lerem como parte do
        // campo, e não como dois botões que por acaso estão ao lado dele — a
        // borda do `<TextInput>` já é a moldura do conjunto.
        Template::Inline(
            r#"<Row spacing="0" align_y="center">
                    <style>
                        .spinbox-step {
                            color: #8080801f;
                            text-color: #80868d;
                            border-width: 0;
                            border-radius: 3;
                        }
                        .spinbox-step:hover  { background: #8080803d; }
                        .spinbox-step:active { background: #80808066; }
                    </style>

                    <template if="{layout|stacked}" equals="inline">
                        <Button
                            class="spinbox-step {step_class}"
                            on_click="dec:{value}|{min|0}|{max|100}|{step|1}|{decimals|}"
                            padding="6 12"
                        >
                            <Text class="{glyph_class}" content="{dec_text|−}" size="{glyph_size|15}" />
                        </Button>
                        <TextInput
                            class="{field_class}"
                            formControl="{form_control}"
                            value="{value}"
                            onChange="edit:{value}|{min|0}|{max|100}|{step|1}|{decimals|}"
                            placeholder="{placeholder}"
                            width="{width|72}"
                        />
                        <Button
                            class="spinbox-step {step_class}"
                            on_click="inc:{value}|{min|0}|{max|100}|{step|1}|{decimals|}"
                            padding="6 12"
                        >
                            <Text class="{glyph_class}" content="{inc_text|+}" size="{glyph_size|15}" />
                        </Button>
                    </template>

                    <template else>
                        <TextInput
                            class="{field_class}"
                            formControl="{form_control}"
                            value="{value}"
                            onChange="edit:{value}|{min|0}|{max|100}|{step|1}|{decimals|}"
                            placeholder="{placeholder}"
                            width="{width|72}"
                        />
                        <Column spacing="1">
                            <Button
                                class="spinbox-step {step_class}"
                                on_click="inc:{value}|{min|0}|{max|100}|{step|1}|{decimals|}"
                                padding="0 7"
                            >
                                <Text class="{glyph_class}" content="{inc_text|▴}" size="{glyph_size|11}" />
                            </Button>
                            <Button
                                class="spinbox-step {step_class}"
                                on_click="dec:{value}|{min|0}|{max|100}|{step|1}|{decimals|}"
                                padding="0 7"
                            >
                                <Text class="{glyph_class}" content="{dec_text|▾}" size="{glyph_size|11}" />
                            </Button>
                        </Column>
                    </template>
                </Row>"#
                .to_string(),
        )
    }

    fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context) {
        // `inc:qtd|1|99|1|2` — operação, chave-alvo e os parâmetros da
        // instância (min, max, step, decimals).
        let Some((op, payload)) = action.split_once(':') else {
            return;
        };
        let mut campos = payload.split('|');
        let Some(chave) = campos.next().map(str::trim).filter(|c| !c.is_empty()) else {
            // `<SpinBox/>` sem `value` não tem onde escrever: não faz nada, em
            // vez de inventar uma chave e poluir o contexto do app.
            return;
        };
        let num = |s: Option<&str>, padrao: f64| {
            s.and_then(|s| s.trim().parse::<f64>().ok())
                .unwrap_or(padrao)
        };
        let min = num(campos.next(), 0.0);
        let max = num(campos.next(), 100.0);
        let step_txt = campos.next().unwrap_or("1");
        let step = num(Some(step_txt), 1.0);
        // O quinto campo é novo (0.85) e pode vir vazio: um payload antigo,
        // de um template que o app tenha copiado, continua valendo.
        let casas = casas_decimais(step_txt, campos.next().unwrap_or(""));

        if op == "edit" {
            // Digitação: entra filtrado e sem saturar (ver limites conhecidos).
            ctx.set(chave, filtra_numero(value.unwrap_or("")));
            return;
        }

        let atual = ctx.get(chave).and_then(|s| s.trim().parse::<f64>().ok());
        let novo = match atual {
            // Chave vazia/não-numérica: o primeiro clique **inicializa** no
            // mínimo, em vez de pular para `min + step` (que seria o resultado
            // de tratar o vazio como `min` e depois somar).
            None => min,
            Some(v) => match op {
                "inc" => v + step,
                "dec" => v - step,
                _ => return,
            },
        };
        let novo = novo.clamp(min, max);
        // `-0` sai de `0.0 - 0.0` com sinal e formataria como "-0".
        let novo = if novo == 0.0 { 0.0 } else { novo };
        ctx.set(chave, format!("{novo:.casas$}"));
    }
}
