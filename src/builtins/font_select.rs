/// `QFontComboBox`: a lista de famílias de fonte, **cada uma desenhada nela
/// mesma**.
///
/// ```xml
/// <fontselect value="fonte" selected="{fonte}" preview="Sphinx of black quartz" />
/// ```
///
/// # Por que é builtin, e a 16ª correção de nível
///
/// O catálogo (§2.4) marcava o `FontComboBox` como `Comp ●` — "exige estado por
/// instância". Não exige: a família escolhida é **um texto numa chave que o app
/// nomeia** (`value="fonte"`), o padrão do `<SpinBox>` pela enésima vez. O que
/// faltava não era o motor, era o **registro de famílias** — a Onda 10
/// (`src/fonts.rs`), que também semeia a chave `__fonts` de onde esta lista lê
/// por padrão.
///
/// # O que o separa de um `<select>` qualquer
///
/// Uma linha: `font="{familia}"` no `<Text>` de cada item. Um `<pick_list>` do
/// `iced` não deixa estilizar cada opção do dropdown; por isso o corpo é uma
/// lista de botões (a forma do `<listview>`), onde cada rótulo é um nó que
/// aceita `font=` — e aí o nome "JetBrains Mono" aparece **em** JetBrains Mono.
///
/// # Props
///
/// - `value`    — **obrigatória**: chave que recebe a família clicada.
/// - `selected` — o valor atual dessa chave, para o destaque. Sem ela nada casa
///   e a lista aparece inteira sem seleção (funcional, sem indicar onde se está).
/// - `items`    — chave com o array de nomes. Default: `__fonts` (a do motor).
/// - `preview`  — texto de amostra mostrado abaixo da lista, na fonte
///   selecionada. Sem ele, sem amostra.
/// - `height`   — altura da área rolável. Default: `220`.
/// - `width`    — largura. Default: `fill`.
/// - `size`     — corpo de cada rótulo. Default: `15`.
///
/// # Aparência
///
/// `.fontselect-item` / `.fontselect-item-sel` na folha do template (instalada
/// em `GlacierUI::new`, antes de qualquer `.gss` do app). `item_class` /
/// `selected_class` no uso injetam classes por cima.
use crate::component::{Component, Context, Template};

pub struct FontSelect;

impl Component for FontSelect {
    fn name(&self) -> &str {
        "FontSelect"
    }

    fn template(&self) -> Template {
        // `for-each="{items|__fonts}"` sobre um array de STRINGS — `var="fam"`
        // dá a string nua (como o `<Chip for-each="tags">` da Onda 11), não um
        // `{it.id}`/`{it.label}`.
        Template::Inline(
            r#"<Column spacing="8" width="{width|fill}" class="{class}">
                    <style>
                        .fontselect-item {
                            color: #00000000;
                            text-color: #cdd6f4;
                            border-width: 0;
                            border-radius: 5;
                        }
                        .fontselect-item:hover { background: #8080803d; }
                        .fontselect-item-sel { color: #8080805c; }
                        .fontselect-preview { color: #a6adc8; }
                    </style>

                    <Scrollable width="fill" height="{height|220}">
                        <Column spacing="2" width="fill">
                            <template for-each="{items|__fonts}" var="fam">
                                <template if="{fam}" equals="{selected}">
                                    <Button
                                        class="fontselect-item fontselect-item-sel {item_class} {selected_class}"
                                        on_click="pick:{value}|{fam}"
                                        padding="8 12"
                                        width="fill"
                                    >
                                        <Text content="{fam}" font="{fam}" size="{size|15}" />
                                    </Button>
                                </template>
                                <template else>
                                    <Button
                                        class="fontselect-item {item_class}"
                                        on_click="pick:{value}|{fam}"
                                        padding="8 12"
                                        width="fill"
                                    >
                                        <Text content="{fam}" font="{fam}" size="{size|15}" />
                                    </Button>
                                </template>
                            </template>
                        </Column>
                    </Scrollable>

                    <template if="{preview}" notEquals="">
                        <Text
                            class="fontselect-preview"
                            content="{preview}"
                            font="{selected}"
                            size="18"
                        />
                    </template>
                </Column>"#
                .to_string(),
        )
    }

    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        // `pick:fonte|JetBrains Mono` — a chave que o app nomeou e a família
        // clicada. A chave viaja na ação porque o `update` não enxerga as props
        // da instância (a mesma razão do payload do `<listview>`/`<spinbox>`).
        let Some(("pick", payload)) = action.split_once(':') else {
            return;
        };
        let Some((chave, familia)) = payload.split_once('|') else {
            return;
        };
        let chave = chave.trim();
        if chave.is_empty() || familia.is_empty() {
            return;
        }
        ctx.set(chave, familia.to_string());
    }
}
