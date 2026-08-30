# {{titulo}}

Interface declarativa com [glacier-ui](https://crates.io/crates/glacier-ui):
o layout mora em XML (`.gv`), o estilo num `.gss` CSS-like e o comportamento
num `<script>` Luau interpretado em runtime.

```
cargo run
```

## Onde está o quê

| Arquivo | O que é |
|---|---|
| `src/main.rs` | a casca: sobe o runner e registra a tela |
| `views/contador.gv` | a tela: `<screen>` (janela) + `<resources>` + layout + `<script>` |
| `views/styles/app.gss` | a paleta (`:root`) e as classes |
| `views/scripts/glacier.d.luau` | tipos dos globais do motor, só para o luau-lsp |
| `.luaurc` | declara esses globais para o type-checker |

## Hot-reload

Com o app aberto, salve o `.gv` ou o `.gss`: o motor relê o arquivo e redesenha.
Só o `src/main.rs` exige recompilar — e ele quase não muda.

## Próximo passo

`glacier presets` lista presets maiores (navegação entre telas, componentes com
props, multi-janela, bandeja). A referência completa da linguagem está no
[README do glacier-ui](https://github.com/antoniofernandodj/xml-ui).
