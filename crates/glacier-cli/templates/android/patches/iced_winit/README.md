# iced_winit 0.14.1 + patch de Android

Cópia do `iced_winit` 0.14.1 (MIT, © Héctor Ramón Jiménez) com **duas**
mudanças, ambas atrás de `#[cfg(target_os = "android")]` — em qualquer outro
alvo o crate é idêntico ao publicado. O diff completo está em `ANDROID.patch`.

1. `src/lib.rs` — o `winit` 0.30 só cria o `EventLoop` no Android com o
   `AndroidApp` que o `android_main` recebe. O módulo `iced_winit::android`
   guarda esse valor (`set_android_app`) e o `run()` o entrega ao
   `EventLoopBuilderExtAndroid::with_android_app`.
2. `src/conversion.rs` — `winit::platform::modifier_supplement` não existe no
   Android; lá o teclado usa `logical_key`/`text` direto (o mesmo desvio que o
   `iced` já faz na web).

Entra no build por `[patch.crates-io]` no `Cargo.toml` do projeto. Ao subir a
versão do `glacier-ui` para um `iced` novo, refaça a cópia e reaplique o
`ANDROID.patch` (`patch -p2 < ANDROID.patch` dentro desta pasta).
