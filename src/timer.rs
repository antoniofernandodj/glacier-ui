//! `sleep` para os dois alvos.
//!
//! O `after(ms, fn)` e os polls de canal (`external`) esperavam com
//! `tokio::time::sleep`. No navegador não há tokio: o executor do `iced` é o
//! `wasm-bindgen-futures`, e o timer que ele usa é o `wasmtimer` — o mesmo que
//! este módulo usa lá.

use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn sleep(duracao: Duration) {
    tokio::time::sleep(duracao).await
}

#[cfg(target_arch = "wasm32")]
pub(crate) async fn sleep(duracao: Duration) {
    wasmtimer::tokio::sleep(duracao).await
}
