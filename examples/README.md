# examples

Cada exemplo mora em `examples/<nome>/`: o markup em `.gv`, o estilo em `.gss`
e, quando o comportamento é Rust, um `main.rs`. Os que dependem de arquivos
próprios os guardam na própria pasta (`i18n/locales/`, `rede_completa/rede_rust.rs`…).

Rodar um exemplo declarado no `Cargo.toml`:

```
WGPU_BACKEND=gl cargo run --example <nome>
```

O `gv_run` abre qualquer template, mas só serve aos exemplos cujo comportamento
mora em Luau ou que são telas estáticas: os que dependem de um `Component` em Rust
(`contador`, `onda9`…) usam o próprio `main.rs`.

```
cargo run --example gv_run -- examples/<nome>/<arquivo>.gv
```

Ferramentas, em `examples/gv_*` (cada uma com o seu `main.rs`):

- `gv_check --load <arquivos>` — registra os arquivos pelo carregador real
  (`--same A B` compara a árvore de duas escritas; `--xml A` mostra o XML interno gerado);
- `gv_run <arquivo>` — abre um template numa janela.

`examples/conteudo_prose/` guarda os textos de artigo (HTML, `.gv`, Markdown) que
`prose_artigo` e `prose_artigo_luau` leem do disco.

`examples/inline_script` é escrito à mão: o teste de realce de `script` e
`style` com o código no próprio arquivo.
