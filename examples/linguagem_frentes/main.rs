//! O exemplo das Frentes 1–11 do `TODO.md` — uma tela só, com a lógica em Luau.
//!
//! Rodar: `WGPU_BACKEND=gl cargo run --example linguagem_frentes`
//!
//! Cada bloco da tela é uma das frentes do plano, e o rodapé diz qual:
//!
//! | Frente | O que fazer na tela para ver |
//! |---|---|
//! | 1 — contexto tipado | mover o slider (`@n > 3` é número), marcar/desmarcar o toggle (`@ativo` é bool), escrever "não" no campo e ver que `if @resposta` é **verdadeiro** com `truthy = typed` |
//! | 2 — estado local | o `Cartao` tem `state { aberto = false }` próprio; os dois cartões não se veem; `@app.contagem` é o global explícito |
//! | 3 — componente fechado | `Cartao` recebe props tipadas, tem slot nomeado com `fallback` e emite `on_abrir(@id)` |
//! | 4 — botão ocupado | `app(busy_buttons = true, busy_release = toast)`: clicar duas vezes em "Salvar (lento)" não dispara duas vezes, e o spinner some no toast, antes do `after(1200)` acabar |
//! | 5 — keyed each | `each @itens as i(key = @i.id)` |
//! | 6 — handler verificado | as ações são todas `nome()`; `glacier check` confere nome, aridade e tipo contra o `.luau` |
//! | 7 — fmt | `glacier fmt` reindentou este arquivo sem perder um comentário |
//! | 8 — cor do CSS | `app(gss_color = css)`: `color` é o conteúdo, `background` é o fundo, no botão também |
//! | 10 — sinais únicos | `:attr = expr` numa via, `::value = @x` nos dois sentidos, `:class.ativa = expr`, `class = [a, b]`, `size = [420, 60]`, `@x ?? "padrão"` |
//! | 11 — escopo local | `Cartao` é importado neste arquivo com `link(rel = import, …)`; o `style` é local ao arquivo |
//!
//! A Frente 9 (migrar o `rustploy-gui`) não acontece aqui: precisa da versão
//! publicada no crates.io.

fn main() {
    let _ = glacier_ui::GlacierDaemon::new()
        .main_template("examples/linguagem_frentes/app.gv")
        .run();
}
