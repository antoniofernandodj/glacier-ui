# Plano: `script` no nível do `app`

**Estado:** proposta. Nasce de `PLANO_APP_TELAS.md` (seção 6): com o manifesto
funcionando, o `script` ainda só existe **dentro de uma tela**.

Hoje `script` no `resources` do `app` é erro de parse ("o script do app inteiro
ainda não existe"), e um `<script>` fora de qualquer tela também. O que falta é um
lugar para o estado e a lógica que vivem o app inteiro, sem repetir em cada tela.

---

## 1. O que levantar antes de escrever código

### 1.1 Onde o script do app roda

- **Um runtime por app + um por tela** (o modelo de hoje, mais um) ou **um só
  compartilhado**? Hoje cada tela com `script` é um `LuauComponent` com a própria
  VM (`ctx` é uma tabela por componente, sincronizada com o contexto do motor).
- O que as telas enxergam do app: `ctx` (o contexto do motor já é um só por
  janela — ver `AGENTS.md`), o global `storage`, funções (`require` de um módulo, ou
  funções do app visíveis nas telas?).
- Janelas filhas (`WindowSource::AppScreen`) carregam o manifesto de novo: o
  script do app roda **uma vez por janela** (estado isolado, como o resto do
  motor) — confirmar que é isso que se quer, e como o `broadcast` entra.

### 1.2 Ordem de inicialização

- `init` do app **antes** do `init` da primeira tela; o que acontece com o `init`
  de uma tela inline recarregada pelo hot-reload (`load_app`), que hoje recria a
  VM da tela.
- Ações sem dono: hoje caem na tela ativa (`dispatch`); ações do app (`tray:*`
  desconhecidas, atalhos) poderiam cair no script do app antes da tela.

### 1.3 Convivência

- com o `script` local de cada tela (funções de mesmo nome: quem vence?);
- com o hot-reload (`script_only_components`, `register_inline_screen`): recompilar
  o script do app reinicia o estado de todas as telas?
- com `lua_extension` (`GlacierDaemon::lua_extension`) e o global `storage`: a
  extensão é registrada por motor; o script do app seria o lugar natural de
  `require`-ar o que ela expõe.

## 2. Forma proposta (a validar)

```
app(id = meu_app) {
  resources {
    script(src = "scripts/app.luau")      // ou inline
  }
  screen(name = home) { … }
}
```

Parser: `AppManifest` ganha `script: Option<String>`; o `lift_scripts` já separa os
blocos por linha, então o do `resources` do `app` deixa de ser erro e passa a ser
atribuído ao app. Motor: `GlacierUI::register_app` instala um componente
`__app` (como o `APP_SCOPE` das declarações globais) e o `dispatch` o consulta
antes da tela ativa.

## 3. Como verificar

Mesmo princípio do plano das telas: olhar o **resultado**, não só o parse — o
`init` do app escreveu uma chave que a primeira tela **lê** (e a chave não é a
mesma que o teste escreve), o estado sobrevive à navegação, e o hot-reload do
manifesto não perde o que o `init` semeou.
