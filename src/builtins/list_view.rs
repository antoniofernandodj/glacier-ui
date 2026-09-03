/// `QListWidget`: a lista rolável cujo item escolhido mora numa chave.
///
/// É o `TabBar` na vertical, com scroll — e é essa a razão de ele ser um
/// builtin e não uma primitiva: tudo de que ele precisa (uma coleção numa
/// chave, um `for-each`, um destaque condicional) o markup já sabia fazer. O
/// que faltava era **a seleção**, e ela é o padrão do `SpinBox`: a chave é
/// nomeada pelo app, a ação carrega o nome, o `update` grava.
///
/// ```xml
/// <listview items="servicos" value="servico" selected="{servico}" height="240" />
/// ```
///
/// # Os dois modos de seleção
///
/// | `mode` | a chave guarda | o destaque testa |
/// |---|---|---|
/// | `single` (default) | um id (`"api"`) | `equals` |
/// | `multi` | um **conjunto** (`"api,db"`) | `contains` |
///
/// O modo múltiplo é o primeiro consumidor do `contains` (0.84) — e a
/// demonstração de que ele não precisava de estado por instância, que é o que
/// o `PLANO_WIDGETS.md` §3 dizia. Um conjunto é uma string; alternar um item
/// dentro dela é uma linha do `update` daqui.
///
/// # `value` e `selected` andam em par
///
/// Pelo mesmo motivo do `TabBar`: o template precisaria ler o valor da chave
/// cujo *nome* está numa prop — a indireção `{{value}}` que o interpolador não
/// tem. Daí a forma canônica repetir o nome dentro das chaves:
/// `value="servico" selected="{servico}"`. Sem `selected`, nada casa e a lista
/// aparece inteira apagada — funcional, mas sem indicar onde se está.
///
/// # Props
///
/// - `items`    — **obrigatória**: nome da chave com o array de `{id, label}`.
///   `sub` num item vira a segunda linha (o `QListWidgetItem` com subtítulo).
/// - `value`    — **obrigatória**: nome da chave que recebe o id clicado.
/// - `selected` — o valor atual dessa chave, para o destaque.
/// - `mode`     — `single` (default) ou `multi`.
/// - `height`   — altura da área rolável. Default: `240`.
/// - `width`    — largura. Default: `fill`.
/// - `spacing`  — espaço entre as linhas. Default: `2`.
/// - `padding`  — área de clique de cada linha. Default: `8 12`.
/// - `size`     — corpo do rótulo. Default: `13`.
/// - `virtualize` — altura declarada de cada linha, para listas longas (ver
///   `PRIMITIVAS.md`). Default: `0`, que desliga a virtualização.
///
/// # Aparência
///
/// `.listview-item` e `.listview-item-sel`, na folha global do template —
/// instalada em `GlacierUI::new` e portanto **antes** de qualquer `.gss` do
/// app, que por isso vence por ordem.
use crate::component::{Component, Context, Template};

pub struct ListView;

/// Alterna a presença de `id` num conjunto separado por vírgula, preservando a
/// ordem de inserção (que é a ordem em que a pessoa clicou — a única ordem que
/// ela consegue prever).
///
/// Aceita os mesmos três separadores do `contains` na entrada e devolve sempre
/// com vírgula: um conjunto que sai daqui volta a entrar aqui, e a ida e a
/// volta precisam ser estáveis.
pub(super) fn alterna_no_conjunto(atual: &str, id: &str) -> String {
    let mut itens: Vec<&str> = atual
        .split([',', ';', ' ', '\t', '\n'])
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .collect();
    match itens.iter().position(|t| *t == id) {
        Some(i) => {
            itens.remove(i);
        }
        None => itens.push(id),
    }
    itens.join(",")
}

impl Component for ListView {
    fn name(&self) -> &str {
        "ListView"
    }

    fn template(&self) -> Template {
        // Três braços de destaque, não dois: o modo múltiplo testa `contains`
        // (o conjunto está na chave) e o simples testa `equals`. As duas
        // diretivas não compõem numa tag só, então cada uma tem o seu
        // `<template>`.
        //
        // A `<Column>` é filha DIRETA do `<Scrollable>` de propósito: é onde o
        // motor procura o `virtualize` (ver `PRIMITIVAS.md`).
        Template::Inline(
            r#"<Scrollable width="{width|fill}" height="{height|240}">
                    <style>
                        .listview-item {
                            color: #00000000;
                            text-color: #cdd6f4;
                            border-width: 0;
                            border-radius: 5;
                        }
                        .listview-item:hover { background: #8080803d; }
                        .listview-item-sel {
                            color: #8080805c;
                            text-color: #cdd6f4;
                        }
                        .listview-sub { color: #80868d; }
                    </style>

                    <Column
                        spacing="{spacing|2}"
                        width="fill"
                        virtualize="{virtualize|0}"
                    >
                        <template for-each="{items}" var="it">
                            <template if="{mode|single}" equals="multi">
                                <template if="{selected}" contains="{it.id}">
                                    <Button
                                        class="listview-item listview-item-sel"
                                        on_click="pick:{mode|single}|{value}|{it.id}"
                                        padding="{padding|8 12}"
                                        width="fill"
                                    >
                                        <Column spacing="1" width="fill">
                                            <Text content="{it.label}" size="{size|13}" bold="true" />
                                            <template if="{it.sub}" notEquals="">
                                                <Text class="listview-sub" content="{it.sub}" size="11" />
                                            </template>
                                        </Column>
                                    </Button>
                                </template>
                                <template else>
                                    <Button
                                        class="listview-item"
                                        on_click="pick:{mode|single}|{value}|{it.id}"
                                        padding="{padding|8 12}"
                                        width="fill"
                                    >
                                        <Column spacing="1" width="fill">
                                            <Text content="{it.label}" size="{size|13}" />
                                            <template if="{it.sub}" notEquals="">
                                                <Text class="listview-sub" content="{it.sub}" size="11" />
                                            </template>
                                        </Column>
                                    </Button>
                                </template>
                            </template>

                            <template else>
                                <template if="{it.id}" equals="{selected}">
                                    <Button
                                        class="listview-item listview-item-sel"
                                        on_click="pick:{mode|single}|{value}|{it.id}"
                                        padding="{padding|8 12}"
                                        width="fill"
                                    >
                                        <Column spacing="1" width="fill">
                                            <Text content="{it.label}" size="{size|13}" bold="true" />
                                            <template if="{it.sub}" notEquals="">
                                                <Text class="listview-sub" content="{it.sub}" size="11" />
                                            </template>
                                        </Column>
                                    </Button>
                                </template>
                                <template else>
                                    <Button
                                        class="listview-item"
                                        on_click="pick:{mode|single}|{value}|{it.id}"
                                        padding="{padding|8 12}"
                                        width="fill"
                                    >
                                        <Column spacing="1" width="fill">
                                            <Text content="{it.label}" size="{size|13}" />
                                            <template if="{it.sub}" notEquals="">
                                                <Text class="listview-sub" content="{it.sub}" size="11" />
                                            </template>
                                        </Column>
                                    </Button>
                                </template>
                            </template>
                        </template>
                    </Column>
                </Scrollable>"#
                .to_string(),
        )
    }

    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        // `pick:multi|servico|api` — o modo, a chave que o app nomeou e o id
        // clicado. O modo viaja na ação porque o `update` não enxerga as props
        // da instância (a mesma razão do payload do `SpinBox`).
        let Some(("pick", payload)) = action.split_once(':') else {
            return;
        };
        let mut campos = payload.splitn(3, '|');
        let modo = campos.next().unwrap_or("single").trim();
        let chave = campos.next().unwrap_or("").trim();
        let id = campos.next().unwrap_or("").trim();
        // Sem `value` não há onde escrever: não faz nada, em vez de inventar
        // uma chave e poluir o contexto do app (mesma regra do `SpinBox`).
        if chave.is_empty() || id.is_empty() {
            return;
        }

        if modo == "multi" {
            let atual = ctx.get(chave).cloned().unwrap_or_default();
            ctx.set(chave, alterna_no_conjunto(&atual, id));
        } else {
            ctx.set(chave, id.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::alterna_no_conjunto;

    /// Entrar, sair, e a ordem de clique preservada. A saída é sempre com
    /// vírgula, mesmo quando a entrada veio com espaço — o conjunto volta a
    /// entrar aqui no clique seguinte, e a ida e a volta precisam ser estáveis.
    #[test]
    fn conjunto_alterna_e_normaliza() {
        assert_eq!(alterna_no_conjunto("", "api"), "api");
        assert_eq!(alterna_no_conjunto("api", "db"), "api,db");
        assert_eq!(alterna_no_conjunto("api,db", "api"), "db");
        assert_eq!(alterna_no_conjunto("api,db", "db"), "api");
        assert_eq!(alterna_no_conjunto("api db", "cache"), "api,db,cache");
        assert_eq!(alterna_no_conjunto("  api , db ", "api"), "db");
        // Tirar o último devolve o vazio, não uma vírgula solta.
        assert_eq!(alterna_no_conjunto("api", "api"), "");
    }
}
