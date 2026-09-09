# {{titulo}}

Um **formulário validado** com [glacier-ui](https://crates.io/crates/glacier-ui):
entradas com máscara, um `<buttonbox>` e validação em Luau que roda a cada
mudança e no envio.

```
cargo run
```

## O mapa

```
src/main.rs                          a casca: registra views/app.gv
views/
├── app.gv                           os campos + <buttonbox>; mostra {erro_<campo>}
├── scripts/app.luau                 init(): semeia as UFs, a data de hoje e os erros vazios
├── scripts/handlers/validar.luau    campo() grava e revalida; enviar() decide
└── styles/{theme.json, app.gss}
```

## Como funciona

- Cada `on_change="campo:f_cpf"` chama `campo("f_cpf", texto)` — grava a chave
  **e** recalcula todos os `erro_<campo>`. O markup só exibe `{erro_cpf}`; não
  decide nada.
- `<maskedinput mask="cpf">` guarda o valor **cru** (sem pontuação) — por isso
  a regra é "11 dígitos", não um formato.
- `<spinbox>` grava a própria chave (é builtin) e não tem `on_change`, então a
  idade só revalida quando outro campo muda ou no envio — `enviar()` sempre
  revalida tudo antes de decidir.
- `enviar()` mostra o status ou um toast de aviso; `limpar()` zera os campos.

## Adaptar

Troque as regras em `erro_de()` e os campos em `app.gv`. Para submeter a uma
API, ponha um `fetch(...)` dentro de `enviar()` no ramo de sucesso — ele
suspende sem travar a janela.
