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
`views/contador.gv` ou `views/styles/app.gss` redesenha sem recompilar.

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

`ctx` é a ponte para o markup: `ctx.set("contador", ...)` é o que o
`{contador}` do template lê.
