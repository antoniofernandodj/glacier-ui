# Plano: `script` no nível do `app`

> **ARQUIVADO — CONCLUÍDO** (2026-10-25). Mantido como registro histórico.

**Estado:** **implementado** (glacier-ui 0.139.0). Nasce de `PLANO_APP_TELAS.md` (seção 6):
com o manifesto funcionando, o `script` ainda só existia **dentro de uma tela**. O exemplo de
aceitação é `examples/app_script`. As perguntas da seção 1 estão respondidas em "O que ficou
decidido", no fim.

Antes da 0.139, `script` no `resources` do `app` era erro de parse ("o script do app inteiro
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

---

## 4. O que ficou decidido

- **1.1 Onde roda.** Um componente a mais por motor (`__app_script`), com a própria VM, ao lado
  do de cada tela. O `ctx` é o mesmo (o contexto do motor já é um só por janela). Uma janela
  filha (`open_window`) roda o script do app de novo, no contexto dela — estado isolado, como o
  resto do motor.
- **1.2 Ordem.** O `init` do app roda dentro do `load_app`, **antes** de registrar qualquer
  tela, então antecede o `init` da primeira. Ações sem prefixo seguem para a tela ativa; o
  caminho para o script do app é explícito: `app::nome`. Não há captura automática de `tray:*`
  ou atalho — quem quiser, escreve `on_click = "app::…"`.
- **1.3 Convivência.** Funções de mesmo nome não colidem: são VMs separadas, e o prefixo
  escolhe o dono. Hot-reload: o texto do script mudou → VM nova e `init` de novo; igual → nada.
  Tirar o script remove o componente. Uma tela chamada `app` tem precedência sobre o prefixo.
- **Fora do escopo.** O `glacier test` (`component()`) não carrega o script do app; a lógica que
  precisa de teste vai num módulo `require`ado. `lua_extension` e `storage` valem no script do
  app como em qualquer script (são registrados por motor).
