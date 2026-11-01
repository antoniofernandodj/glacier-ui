# {{titulo}}

Aplicação desktop com [glacier-ui](https://crates.io/crates/glacier-ui), com o
comportamento em **Rust** (o trait `Component`) em vez de num `<script>` Luau.

```
cargo run
```

## Os dois caminhos

O glacier-ui aceita as duas formas de dar comportamento a um template, e elas
convivem no mesmo app:

| | `impl Component` (Rust) | `<script>` (Luau) |
|---|---|---|
| Estado | campos tipados na struct | chaves de string no `ctx` |
| Erro | em tempo de compilação | em runtime |
| Mudou o comportamento | recompila | salva o arquivo, e pronto |

Este preset usa o primeiro. O markup continua recarregando a quente: salvar
`views/contador.gv` ou `views/styles/app.gss` redesenha sem recompilar. E um
`<script>` pode ser acrescentado depois sem tirar o `Component` do lugar — por
isso o `views/scripts/glacier.d.luau` já vem junto.

## O mapa

```
src/main.rs                     a casca: sobe o runner e registra o componente
src/contador.rs                 o Component: template + estado + update
views/contador.gv               o layout
views/styles/app.gss            a paleta (:root) e as classes
views/scripts/glacier.d.luau    tipos dos globais do Luau, se um dia entrar um <script>
```

## O contrato

```rust
fn name(&self) -> &str;                   // nome do registro e do roteamento
fn template(&self) -> Template;           // Template::File(caminho) | Template::Inline(xml)
fn init(&mut self, ctx: &mut Context);    // estado inicial
fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context);
fn children(&self) -> Vec<Box<dyn Component>>;   // sub-componentes, opcional
fn on_broadcast(&mut self, event: &str, payload: &str, ctx: &mut Context);
```

## Como as peças se ligam

**`register` e não `register_component`.** O registro de um `Component` pega o
template dele por `Component::template()`; `register_component` é a forma para
um template sem comportamento em Rust.

**`Template::File` mantém o hot-reload.** O caminho é relativo ao diretório de
onde o app roda (a raiz do projeto, num `cargo run`).

**`publicar` é a ponte.** O estado real são os campos da struct — `passo` é um
`i32`, não uma string, e trocar o tipo quebra a compilação em vez de virar um
`nil` em runtime. O `ctx` guarda strings, e a conversão acontece num lugar só:
`ctx.set("contador", …)` é o que o `{contador}` do template lê.

**`update` recebe todo clique.** `value` traz o texto de um `on_change`/
`on_toggle` e é `None` num clique simples. Um `TextInput` faz binding de mão
dupla: `value` aponta para a chave exibida e `onChange` dispara a ação a cada
tecla.

Duas decisões no `update` que valem copiar:

- **Entrada inválida mantém o valor anterior.** Um campo de texto contém
  qualquer coisa enquanto é digitado; um passo ilegível não deve zerar o
  comportamento do app no meio da digitação.
- **Ação desconhecida não é erro.** Ela pode pertencer a outro componente da
  árvore, e o motor já cuidou do roteamento — daí o `_ => return`.
