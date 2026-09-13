// Compila o "embed port" do MicroPython vendorizado em `vendor/`, só quando a
// feature `micropython` está ligada — quem não usa `<script lang="micropython">`
// não paga o custo de compilar um interpretador C inteiro. Ver
// `vendor/glacier_mp_shim/glacier_mp_shim.h` para o porquê do shim próprio, e
// `src/micropython/mod.rs` para o lado Rust que chama estas funções via FFI.
//
// `cc` é uma build-dependency OPCIONAL (`dep:cc` só entra com a feature
// `micropython` ligada — ver Cargo.toml) — sem a feature, a crate `cc` nem
// está disponível pra este binário, então a chamada só pode existir atrás de
// `#[cfg(feature = "micropython")]` de verdade (não um `if` em runtime): o
// `if` sozinho ainda tentaria resolver `cc::Build` na compilação do PRÓPRIO
// build.rs e falharia com "crate `cc` não encontrada" mesmo com a feature
// desligada, porque nesse caso a dependência opcional não foi nem compilada.
fn main() {
    build_micropython();
}

#[cfg(not(feature = "micropython"))]
fn build_micropython() {}

#[cfg(feature = "micropython")]
fn build_micropython() {
    let mut build = cc::Build::new();
    build
        .include("vendor/glacier_mp_shim")
        .include("vendor/micropython_embed")
        .include("vendor/micropython_embed/port")
        // gnu99, não c99: `shared/runtime/gchelper_generic.c` usa variáveis de
        // registrador nomeadas (`register long rbx asm("rbx")`) pra escanear
        // os registros calados na pilha durante uma coleta do GC — é extensão
        // GNU, `-std=c99` estrito rejeita a sintaxe `asm` inteira.
        .std("gnu99")
        .warnings(false);

    for dir in [
        "vendor/micropython_embed/py",
        "vendor/micropython_embed/shared/runtime",
        "vendor/glacier_mp_shim",
    ] {
        println!("cargo:rerun-if-changed={dir}");
        for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("lendo {dir}: {e}")) {
            let path = entry.expect("entrada de diretório ilegível").path();
            if path.extension().is_some_and(|ext| ext == "c") {
                build.file(&path);
            }
        }
    }

    // Não compila `vendor/micropython_embed/port/*.c` (embed_util.c,
    // mphalport.c): o shim redefine tudo que eles fornecem (mp_embed_*,
    // gc_collect, nlr_jump_fail, __assert_func, mp_hal_stdout_tx_strn_cooked)
    // — compilar os dois lados daria símbolo duplicado no link.

    build.compile("glacier_micropython");
}
