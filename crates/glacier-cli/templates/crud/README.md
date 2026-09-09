# {{titulo}}

Uma **lista editável (model/view)** com [glacier-ui](https://crates.io/crates/glacier-ui):
uma `<tableview>` ligada a um array do contexto, com Novo / Editar / Excluir via
`prompt{}` e `confirm{}` da camada Luau. Os dados vivem em memória.

```
cargo run
```

## O mapa

```
src/main.rs                        a casca: registra views/app.gv
views/
├── app.gv                         <tableview items="linhas" columns="colunas" value="sel" sort="ordem">
├── scripts/state.luau             o DEPÓSITO: { itens: {Servico}, seq } tipado
├── scripts/handlers/itens.luau    a VITRINE: publicar() filtra e encoda; novo()/editar()/excluir()
└── styles/{theme.json, app.gss}
```

## Como funciona

- **Duas memórias.** `state.luau` guarda a lista de verdade (structs tipadas);
  `Itens.publicar()` a projeta em `ctx.linhas` como JSON, já filtrada por
  `ctx.q`. O `<tableview>` lê essa chave.
- **`value="sel"`** recebe o `id` da linha selecionada — `editar()`/`excluir()`
  acham o item por ele.
- **Os modais suspendem.** `prompt{}` e `confirm{}` param a corrotina da ação e
  voltam com a resposta; `novo()` encadeia três `prompt{}` (nome, ambiente por
  `kind="item"`, réplicas por `kind="int"`). A janela não trava.
- **`sort="ordem"`** deixa o cabeçalho ordenar; `widths="larguras"` guarda as
  colunas arrastadas.

## Persistir

Hoje a lista é recriada a cada `cargo run`. Para gravar em disco, escreva
`State.itens` num JSON no `storage` a cada mudança e leia no `init()`; ou troque
o depósito por uma ponte Rust — ver o exemplo `sqlite_crud` do repositório do
glacier-ui, que faz exatamente isto com SQLite.
