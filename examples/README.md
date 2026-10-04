# examples

Os exemplos vêm em duas grafias do mesmo markup:

| pasta | grafia | como rodar |
|---|---|---|
| `examples/gva/<nome>/` | **`.gva`** (atributos, XML) — os exemplos de sempre, com `main.rs` | `cargo run --example <nome>` |
| `examples/gvb/<nome>/` | **`.gvb`** (blocos) — a tradução de cada exemplo `.gva`, conferida árvore a árvore | `cargo run --example gvb_run -- examples/gvb/<nome>/<arquivo>.gvb` |

O `gvb_run` abre qualquer template, mas só serve aos exemplos cujo comportamento
mora em Luau ou que são telas estáticas: os que dependem de um `Component` em Rust
(`contador`, `onda9`…) usam o `main.rs` da versão `.gva`.

Ferramentas, em `examples/gvb/gvb_*` (cada uma com o seu `main.rs`):

- `gvb_convert <saida> <arquivo.gva>...` — traduz `.gva` em `.gvb` e confere cada
  tradução (`--in-place` grava ao lado do original);
- `gvb_check --load <arquivos>` — registra os arquivos pelo carregador real
  (`--same A B` compara a árvore de dois `.gvb`; `--xml A` mostra o XML gerado);
- `gvb_run <arquivo>` — abre um template numa janela.

`examples/gvb/inline_script` é escrito à mão: o teste de realce de `script` e
`style` com o código no próprio arquivo.
