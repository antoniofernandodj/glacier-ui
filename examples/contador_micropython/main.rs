//! Igual ao `contador_externo` (Lua), mas o `<script>` é **MicroPython** —
//! a segunda linguagem de `<script>` do motor, ao lado do Luau:
//!
//! ```xml
//! <script lang="micropython" src="contador_micropython.py"></script>
//! ```
//!
//! `lang="micropython"` (ou `lang="python"`, aceito como apelido) é o único
//! sinal que diferencia este bloco de um `<script>` Luau comum — sem ele, o
//! motor sempre presume Luau (compatível com todo `.gv` existente). O
//! caminho é resolvido relativo ao diretório do template, como no Lua;
//! `register_component` lê o template, segue o `src`, roda o MicroPython e
//! roteia as ações (`on_click`/`onChange`) para as funções homônimas — que
//! leem/escrevem o contexto pelo **dotdict** `ctx`: `ctx.contador` funciona
//! igual ao `ctx.contador` do Lua (leniente — `None`/ausente sem erro).
//!
//! Cobre só a mudança de estado (ver `src/micropython/mod.rs`): sem
//! `fetch`/rede, diálogos ou temporizadores ainda — isso fica para quando
//! algum exemplo futuro precisar.

use glacier_ui::GlacierDaemon;

fn main() -> iced::Result {
    GlacierDaemon::new()
        .title("Glacier - Contador (MicroPython)")
        .main(|motor| {
            if let Err(e) = motor.register_component(
                "contador",
                "examples/contador_micropython/contador_micropython.gv",
            ) {
                eprintln!("Erro ao registrar: {}", e);
            }
            motor.set_initial_screen("contador");
        })
        .run()
}
