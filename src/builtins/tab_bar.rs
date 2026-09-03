/// `QTabBar`: a fileira de abas, com a ativa em destaque.
///
/// ```xml
/// <TabBar value="aba" active="{aba}" items="abas" />
///
/// <template if="{aba}" equals="geral">  … conteúdo da aba Geral …  </template>
/// <template else-if="{aba}" equals="rede"> … </template>
/// ```
///
/// # Por que só a barra, e não o `QTabWidget` inteiro
///
/// O `QTabWidget` do Qt é a barra **mais** o empilhado de páginas. A barra sai
/// hoje; o empilhado não, porque cada página precisaria do seu próprio buraco
/// no template (`<slot name="geral"/>`, `<slot name="rede"/>`) e o `<slot/>` do
/// motor é único e anônimo. Enquanto o slot nomeado não existe, o conteúdo
/// troca com `se`/`senao` na tela — que é o que o exemplo acima faz, e o que o
/// motor já sabia fazer antes desta barra existir.
///
/// A parte que valia entregar agora é justamente a barra: é ela que tem estado,
/// destaque e área de clique — o `se`/`senao` do lado do conteúdo nunca foi o
/// problema.
///
/// # As abas vêm de uma coleção do contexto
///
/// `items` é o **nome de uma chave** que guarda um array JSON de `{id, label}`
/// — a mesma convenção do `<Menu items="…">` e de qualquer `for-each` do motor
/// (uma lista literal no atributo não existe: o `for-each` lê chave, não texto).
///
/// ```rust,ignore
/// ctx.set("abas", r#"[{"id":"geral","label":"Geral"},{"id":"rede","label":"Rede"}]"#);
/// ctx.set("aba", "geral");
/// ```
///
/// # `value` e `active` andam em par
///
/// - `value` é o **nome** da chave onde a aba escolhida é gravada — é o padrão
///   do [`super::spin_box::SpinBox`], e é ele que dispensa uma linha de Lua: o
///   clique cai no `update` daqui, que escreve a chave sozinho.
/// - `active` é o **valor atual** dessa chave, e serve só para o destaque.
///
/// Os dois são necessários porque o template precisaria ler o valor da chave
/// cujo *nome* está numa prop — a indireção `{{value}}` que o interpolador não
/// tem (o mesmo limite que impede os degraus do `SpinBox` de desabilitarem no
/// fim da faixa). Daí a forma canônica repetir o nome dentro das chaves:
/// `value="aba" active="{aba}"`. Sem `active`, nenhuma aba casa e a barra
/// aparece inteira apagada — funcional, mas sem indicar onde se está.
///
/// # Props
///
/// - `items`   — **obrigatória**: nome da chave com o array de `{id, label}`.
/// - `value`   — **obrigatória**: nome da chave que recebe o `id` clicado.
/// - `active`  — o valor atual da chave `value`, para o destaque.
/// - `padding` — área de clique de cada aba. Default: `7 14`.
/// - `spacing` — espaço entre as abas. Default: `2`.
/// - `size`    — corpo do rótulo. Default: `13`.
/// - `width`   — largura da barra. Default: `fill`.
///
/// # Aparência
///
/// `.tab` (aba comum) e `.tab-active` (a selecionada) na folha global do
/// template. A ativa se distingue por **fundo**, não por sublinhado: uma linha
/// sob a aba precisaria de borda por lado, que o motor não tem.
///
/// Para repintar uma barra só, sem mexer nas outras, uma prop por nó — `class`
/// no uso aplica na `<Row>` que as segura, não nas abas:
///
/// - `tab_class`        — toda aba, ativa ou não.
/// - `tab_active_class` — só a ativa, **por cima** de `tab_class`.
/// - `label_class`      — o rótulo dentro da aba.
///
/// ```xml
/// <tabbar value="aba" active="{aba}" items="abas"
///         tab_class="aba" tab_active_class="aba_ativa" />
/// ```
use crate::component::{Component, Context, Template};

pub struct TabBar;

impl Component for TabBar {
    fn name(&self) -> &str {
        "TabBar"
    }

    fn template(&self) -> Template {
        // O `if`/`else` fica DENTRO do corpo do `for-each`, não no mesmo nó:
        // as duas diretivas não compõem numa tag só (ver o braço `<template>`
        // em `parser.rs`).
        //
        // `on_click="pick:{value}|{tab.id}"` é o payload do padrão SpinBox: o
        // `|` de fora é separador de campo, e o `update` abaixo o desmonta.
        Template::Inline(
            r#"<Row spacing="{spacing|2}" align_y="center" width="{width|fill}">
                    <style>
                        .tab {
                            color: #00000000;
                            text-color: #80868d;
                            border-width: 0;
                            border-radius: 6;
                        }
                        .tab:hover { background: #8080803d; }
                        .tab-active {
                            color: #8080803d;
                            text-color: #cdd6f4;
                        }
                    </style>

                    <template for-each="{items}" var="tab">
                        <template if="{tab.id}" equals="{active}">
                            <Button
                                class="tab tab-active {tab_class} {tab_active_class}"
                                on_click="pick:{value}|{tab.id}"
                                padding="{padding|7 14}"
                            >
                                <Text class="{label_class}" content="{tab.label}" size="{size|13}" bold="true" />
                            </Button>
                        </template>
                        <template else>
                            <Button
                                class="tab {tab_class}"
                                on_click="pick:{value}|{tab.id}"
                                padding="{padding|7 14}"
                            >
                                <Text class="{label_class}" content="{tab.label}" size="{size|13}" />
                            </Button>
                        </template>
                    </template>
                </Row>"#
                .to_string(),
        )
    }

    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        // `pick:aba|geral` — a chave que o app nomeou, e o id da aba clicada.
        let Some(("pick", payload)) = action.split_once(':') else {
            return;
        };
        let Some((chave, id)) = payload.split_once('|') else {
            return;
        };
        let chave = chave.trim();
        // Sem `value` não há onde escrever: melhor não fazer nada do que
        // inventar uma chave e poluir o contexto do app (mesma regra do
        // `SpinBox`).
        if chave.is_empty() {
            return;
        }
        ctx.set(chave, id.to_string());
    }
}
