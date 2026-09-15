//! O módulo `luau` no alvo `wasm32` — sem Luau.
//!
//! O Luau é C++ vendorizado (`luau0-src`), e esse crate só gera wasm para
//! `wasm32-unknown-emscripten`; o iced/winit roda no navegador em
//! `wasm32-unknown-unknown`. Os dois não moram no mesmo binário, então na web a
//! camada `<script>` não existe.
//!
//! Este arquivo mantém a superfície que o resto do motor usa (`has_script`,
//! `storage_root`, `LuauComponent`) para `lib.rs` e `daemon.rs` não se encherem
//! de `cfg`. O que ele NÃO faz é ignorar um `<script>`: um template com script
//! falha no registro com um [`GlacierError::Luau`] que diz o motivo. Uma tela
//! que aparece com os botões mortos é justamente a falha silenciosa que este
//! motor mais caro já pagou (ver CLAUDE.md, "Antes de dizer que funciona").

use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use crate::asset_source::AssetSource;
use crate::component::{Component, Context, Template};
use crate::error::{GlacierError, Result};

const SEM_LUAU: &str = "`<script>` Luau não roda no alvo web (wasm32): o Luau é C++ e só \
    compila para emscripten. Mova o comportamento para um `Component` Rust — ver docs/WEB.md";

static STORAGE_ROOT: OnceLock<PathBuf> = OnceLock::new();

/// Ver `luau/mod.rs`. Na web não há `storage` para usar a raiz, mas o
/// `__data_dir` do markup continua vindo daqui.
pub fn set_storage_root(path: PathBuf) {
    let _ = STORAGE_ROOT.set(path);
}

pub fn storage_root() -> Option<&'static Path> {
    STORAGE_ROOT.get().map(PathBuf::as_path)
}

/// A mesma pergunta do nativo, com a mesma regra de comentário
/// (`find_script_open` pula `<!-- <script> -->`). Aqui um `true` não liga Luau
/// nenhum — faz o registro recusar o componente.
pub(crate) fn has_script(markup: &str) -> bool {
    crate::eval::script_lang(markup).as_deref() != Some("micropython")
        && crate::eval::find_script_open(markup).is_some()
}

/// Nunca é construído na web: os construtores só devolvem erro. `enum` sem
/// variantes para o compilador provar isso nos `match` abaixo.
pub enum LuauComponent {}

impl LuauComponent {
    pub fn from_file(path: impl Into<String>, name: impl Into<String>) -> Result<Self> {
        Self::from_file_with(path, name, Arc::new(crate::DiskAssets))
    }

    pub fn from_file_with(
        _path: impl Into<String>,
        name: impl Into<String>,
        _assets: Arc<dyn AssetSource>,
    ) -> Result<Self> {
        Err(sem_luau(name.into()))
    }

    pub(crate) fn wrap(
        name: &str,
        _path: &str,
        _inner: Box<dyn Component>,
        _assets: Arc<dyn AssetSource>,
    ) -> Result<Self> {
        Err(sem_luau(name.to_string()))
    }
}

fn sem_luau(component: String) -> GlacierError {
    GlacierError::Luau {
        component,
        message: SEM_LUAU.to_string(),
    }
}

impl Component for LuauComponent {
    fn name(&self) -> &str {
        match *self {}
    }

    fn template(&self) -> Template {
        match *self {}
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {
        match *self {}
    }
}
