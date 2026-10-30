# Assets remotos: abrir um app por URL

Fase 1 do `PLANO_SUBSTITUIBILIDADE.md`. Exemplo: `examples/gvb/remoto`.

Um `HttpAssets` é um `AssetSource` como o disco e o embutido — o motor não sabe a
diferença. Duas formas de usar, com o mesmo tipo:

```rust
// App INTEIRO remoto: o binário é só uma casca com a chave do autor.
GlacierDaemon::new()
    .assets(Arc::new(HttpAssets::assinado("https://apps.exemplo.com/loja", CHAVE)))
    .main_template("app.gvb")
    .run()

// TELAS remotas: o app é local; só o prefixo `remoto/` vem da rede.
GlacierDaemon::new()
    .assets(Arc::new(
        RoteadorAssets::new(DiskAssets).montar("remoto/", HttpAssets::assinado(URL, CHAVE)),
    ))
    .main_template("views/app.gvb")   // com screen(name = x, src = "remoto/x.gvb")
    .run()
```

## Assinatura

`manifest.json` lista o SHA-256 de cada arquivo e é assinado com ed25519. O cliente
confere a assinatura com a chave pública que **ele** carrega e cada arquivo contra o
hash do manifesto. `https` protege o transporte; a assinatura protege contra um
servidor comprometido. `HttpAssets::sem_verificacao` é só para desenvolvimento.

```
cargo run --example publicar_remoto -- <pasta> <arquivo-da-chave>   # gera chave + manifesto
```

A chave secreta (o arquivo) não vai para o repositório. A pública (64 hex) vai no
código do cliente: `chave_publica_de_hex`.

## Cache, offline, atualização ao vivo

- Arquivos em `~/.cache/glacier-ui/<url>/blobs/<sha256>`; o hash do manifesto é o
  validador, então arquivo que não mudou nunca é baixado de novo.
- `manifest.json` revalidado com `ETag`/`If-None-Match`.
- Sem rede, serve o último manifesto em cache (reverificado) e seus arquivos.
- A cada 2s (`atualizar_a_cada`) o manifesto é revalidado; arquivo carregado que
  mudou é baixado, conferido e o `modified` avança — o hot-reload faz o resto.
- Uma atualização que não bate com a assinatura/hash **não** troca a versão boa;
  fica em `HttpAssets::ultimo_erro()` e no `stderr`.

## Origem

`AssetSource::origem()` devolve `scheme://host:porta` para fonte remota. O
`GlacierDaemon` grava o `storage` em `<storage_dir>/origens/<origem>/`: um app de
`loja.com` não lê o que o de `banco.com` gravou. Com `RoteadorAssets` a origem é a
da base (o app local), não a da tela que veio da rede.

## O que NÃO está feito

- **Política de confiança.** Um app remoto assinado roda Luau com tudo que o motor
  oferece (`fetch`, `storage`, `open_window`, `LuaExtension`). Não há lista de
  permissões por origem. A assinatura diz *quem* escreveu o app, não *o que* ele
  pode fazer.
- **Várias chaves / "confiar neste autor?".** A chave é uma só, fixa no cliente.
  Abrir uma URL arbitrária, como num navegador, exige isso.
- **Revogação e versão mínima.** Um manifesto antigo mas bem assinado pode ser
  reapresentado (rollback) por um servidor malicioso.
- **Validador sem ETag.** O servidor do exemplo (`python -m http.server`) não manda
  `ETag`; o cliente compara o corpo do manifesto, o que baixa o manifesto a cada
  ciclo. Com um servidor real (nginx) o `304` evita isso.
