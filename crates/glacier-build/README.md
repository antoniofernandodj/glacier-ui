# glacier-build

Helpers para o `build.rs` de um app glacier-ui. Use como `[build-dependencies]`.

```rust
fn main() {
    // Logos: copia `assets/logos/**` para `$OUT_DIR/logos/**`, reduzindo os raster a 96 px.
    glacier_build::stage_images("assets/logos", "logos", 96);
    // `cargo install` grava o .desktop e os ícones de `packaging/` em ~/.local/share.
    glacier_build::desktop_on_cargo_install("meuapp", "packaging", "MEUAPP_INSTALL_DESKTOP");
    // Windows: ícone e versão do .exe a partir de um .rc que usa MEUAPP_VER_COMMA / MEUAPP_VER_STR.
    glacier_build::windows_resource("assets/meuapp.rc", "MEUAPP", &["assets/meuapp.ico"]);
}
```

O embutir em si continua sendo `glacier_ui::embed_dirs!` (por exemplo
`"logos/" => "$OUT_DIR/logos"`).
