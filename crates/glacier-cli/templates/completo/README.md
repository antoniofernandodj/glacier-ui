# {{titulo}}

Aplicação desktop com [glacier-ui](https://crates.io/crates/glacier-ui): a
interface é descrita em XML (`.gv`), o estilo num `.gss` CSS-like e o
comportamento em [Luau](https://luau.org) interpretado em runtime.

```
cargo run
```

## O mapa

```
src/main.rs                     a casca: runner, cromo da janela, storage
views/
├── app.gv                      a JANELA: <screen>, titlebar própria, sidebar, roteador
├── home.gv                     rota "home"   — <component>, cards, for-each, if/else
├── sobre.gv                    rota "sobre"  — <component>
├── components/
│   ├── nav_item.gv             item de sidebar, com contrato em <props>
│   └── stat_card.gv            cartão de número, com prop opcional
├── scripts/
│   ├── app.luau                entrada: carrega os handlers e define init()
│   ├── state.luau              estado mutável compartilhado
│   ├── handlers/nav.luau       navegação
│   ├── handlers/dados.luau     dados, mutação e fetch
│   └── glacier.d.luau          tipos dos globais do motor (só para o luau-lsp)
└── styles/
    ├── theme.json              tema do iced (cores base dos widgets)
    └── app.gss                 tokens :root + as classes compartilhadas
```

## Como as peças se ligam

- **Um registro só.** `src/main.rs` registra `views/app.gv`; os outros templates
  entram por `<link rel="import">` e são carregados em cascata.
- **Um script só.** O contexto do motor é global, então `views/app.gv` é o único
  template com `<script>`: os `on_click` dos templates importados resolvem para
  as funções globais declaradas em `views/scripts/`.
- **Duas memórias.** `ctx` guarda strings — é o que os `{marcadores}` do markup
  leem. `state.luau` guarda as estruturas de verdade; `Dados.publicar()` copia
  uma na outra.
- **Onde fica cada estilo.** Classe usada por mais de um template vai no
  `app.gss`; classe de um template só vai no `<style scoped="true">` dele.

## Hot-reload

Com o app aberto, salve qualquer `.gv`, `.gss` ou `.luau`: o motor relê e
redesenha. Só `src/main.rs` exige recompilar — e ele quase não muda.

## Tipos no editor

O `.luaurc` declara os globais que o motor injeta, e
`views/scripts/glacier.d.luau` os tipa. Com o [luau-lsp](https://github.com/JohnnyMorganz/luau-lsp):

```
luau-lsp analyze --definitions=views/scripts/glacier.d.luau views/scripts
```

Para realce e ir-para-definição nos `.gv`/`.gss`, instale as extensões de VS
Code com `glacier install-extensions`.
