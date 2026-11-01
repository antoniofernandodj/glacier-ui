//! Painel de dados — KPIs e gráficos numa tela só, alimentados por um
//! `every(1000, …)` no `<script>`. `src/main.rs` é só a casca: registra
//! `views/app.gv` automaticamente e abre a janela. Tudo o que o painel
//! É mora nos `.gv`/`.gss`/`.luau` e recarrega a quente.

use glacier_ui::GlacierDaemon;

fn main() -> glacier_ui::iced::Result {
    GlacierDaemon::new()
        .run()
}
