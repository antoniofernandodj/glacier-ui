# {{titulo}}

Um **formulário validado** com [glacier-ui](https://crates.io/crates/glacier-ui):
as regras moram no `<form>` e o motor faz o ciclo inteiro no envio — roda as
regras, mostra a mensagem por campo, acende o destaque vermelho e chama o
handler certo.

```
cargo run
```

## O mapa

```
src/main.rs                          a casca: registra views/app.gv
views/
├── app.gv                           o <form> com rules="…" nos campos; mostra {erro_<campo>}
├── scripts/app.luau                 init(): semeia as UFs, a data de hoje e o status
├── scripts/handlers/validar.luau    salvar() / apontar() / limpar() — só o "depois"
└── styles/{theme.json, app.gss}
```

## Como funciona

O `<form>` declara o que fazer e as regras ficam nos campos:

```xml
<form on_submit="salvar" on_validation_error="apontar" validate_on="submit">
  <input       form_control="f_nome" rules="required|minlen:3"  msg="informe ao menos 3 letras" />
  <maskedinput form_control="f_cpf"  rules="required|digits:11"  mask="cpf" />
  <spinbox     form_control="f_idade" value="f_idade" rules="gte:18" />
  <checkbox    form_control="f_aceite" rules="accepted" label="Aceito os termos" />

  <button type="reset"  on_click="limpar" text="Limpar" />
  <button type="submit"                   text="Salvar" />
</form>
```

- **`rules="…"`** — `|` separa, `:` é o argumento. Aqui: `required`,
  `minlen:N`, `digits:N` (conta só os dígitos — ideal para CPF/telefone com
  máscara), `gte:N`, `accepted`. Também existem `maxlen`, `lte`, `email`,
  `digits:MIN,MAX`, `pattern="regex"` (atributo à parte) e `fn:NOME` (chama a
  função global Luau `NOME(valor)` e usa a string que ela devolver).
- **`msg="…"`** é a mensagem daquele campo, no lugar do texto-padrão do motor.
- **Ao enviar** (Enter num campo ou `<button type="submit">`): tudo passou →
  `salvar`; algo falhou → `apontar`, que recebe as falhas em JSON
  (`[{"campo":"f_nome","msg":"…"}]`). O motor publica `{erro_<campo>}` sozinho.
- **`.campo:invalid` no `.gss`** acende sozinho enquanto o `{erro_<campo>}`
  daquele campo estiver preenchido — não é uma classe que o script liga.
- **`validate_on="submit"`** (padrão): editar um campo **apaga** o erro dele —
  a mensagem só volta no próximo envio. `validate_on="change"` revalida o campo
  a cada tecla.
- **`<button type="reset">`** limpa os `{erro_<campo>}` (e o `:invalid`) e então
  chama o próprio `on_click` — `limpar()` só devolve os valores ao estado
  inicial.

## Adaptar

Mude as `rules` e os campos direto no `app.gv`. Para submeter a uma API, ponha
um `fetch(...)` dentro de `salvar()` — ele suspende sem travar a janela. Para
uma regra que o vocabulário não cobre (dígito verificador de CPF, "senha ≠
login"), use `rules="…|fn:minha_regra"` e escreva `minha_regra(valor)` no
`validar.luau`, devolvendo a mensagem ou `nil`.
