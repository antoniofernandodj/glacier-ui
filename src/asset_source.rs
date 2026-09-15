//! Asset resolution seam — the layer between a logical asset *path* and its
//! *bytes*.
//!
//! Every template (`.gv`), stylesheet (`.gss`), theme/data JSON, Luau
//! script and binary asset (SVG/image) the engine loads at runtime goes through
//! an [`AssetSource`] instead of touching `std::fs` directly. The default
//! [`DiskAssets`] reads from the filesystem exactly as the engine always did,
//! so nothing changes for existing consumers.
//!
//! The point of the seam is to let a consumer swap in an *embedded* source: a
//! standalone release binary can bundle all its assets at compile time (e.g.
//! via `include_dir!`) behind an `AssetSource` and run with the working
//! directory pointing anywhere — the binary becomes fully decoupled from the
//! asset tree on disk. In that mode [`AssetSource::modified`] returns `None`,
//! which naturally disables hot-reload (there is no file to watch).
//!
//! Paths handed to an `AssetSource` are the same logical strings the engine
//! resolves today (CWD-relative, e.g. `crates/app/views/app.gv`, or a Luau
//! module path joined against its caller). A disk source resolves them against
//! the filesystem; an embedded source normalizes and looks them up in its
//! bundle. See [`GlacierDaemon::assets`](crate::GlacierDaemon::assets).

use std::borrow::Cow;
use iced::time::SystemTime;

/// Resolves logical asset paths to their contents.
///
/// Implementations must be cheap to share (`Arc<dyn AssetSource>`), thread-safe
/// (`Send + Sync`), and free of interior mutability surprises — the engine may
/// call these from render and from background reload checks.
pub trait AssetSource: Send + Sync + std::fmt::Debug {
    /// Reads a binary asset (image/SVG). `Cow` so an embedded source can hand
    /// back a borrow of its `'static` bundle without copying.
    fn read_bytes(&self, path: &str) -> std::io::Result<Cow<'static, [u8]>>;

    /// Reads a text asset (template/stylesheet/JSON/Luau) as UTF-8.
    fn read_to_string(&self, path: &str) -> std::io::Result<Cow<'static, str>>;

    /// Whether an asset exists at `path`. Used by the Luau module resolver to
    /// probe candidate module files without reading them.
    fn exists(&self, path: &str) -> bool;

    /// Last-modified time, or `None` when the source cannot change under the
    /// running process (an embedded bundle). Hot-reload keys off this: a `None`
    /// means "never reloads", so [`GlacierUI::check_reload`](crate::GlacierUI::check_reload)
    /// becomes a no-op for embedded assets.
    fn modified(&self, path: &str) -> Option<SystemTime>;

    /// Whether this source can ever report a file change — i.e. whether
    /// hot-reload is structurally possible at all. `true` by default (matches
    /// [`DiskAssets`]). An embedded source should override this to `false`:
    /// [`GlacierDaemon`](crate::GlacierDaemon)'s hot-reload tick uses it to skip
    /// scheduling that subscription entirely, instead of firing on a timer
    /// forever just to call [`modified`](Self::modified) and find `None` every
    /// time — a real cost on a fully software-rendered fallback, where every
    /// dispatched message forces a full redraw of the current screen.
    fn supports_reload(&self) -> bool {
        true
    }
}

/// The default [`AssetSource`]: reads straight from the filesystem, preserving
/// the engine's original behavior (CWD-relative paths, live hot-reload).
#[derive(Debug, Default, Clone, Copy)]
pub struct DiskAssets;

impl AssetSource for DiskAssets {
    fn read_bytes(&self, path: &str) -> std::io::Result<Cow<'static, [u8]>> {
        std::fs::read(path).map(Cow::Owned)
    }

    fn read_to_string(&self, path: &str) -> std::io::Result<Cow<'static, str>> {
        std::fs::read_to_string(path).map(Cow::Owned)
    }

    fn exists(&self, path: &str) -> bool {
        std::path::Path::new(path).is_file()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn modified(&self, path: &str) -> Option<SystemTime> {
        std::fs::metadata(path).and_then(|m| m.modified()).ok()
    }

    // Sem disco no navegador (o `metadata` sempre falharia); e o `SystemTime`
    // do `std` nem é o mesmo tipo do `iced::time` lá.
    #[cfg(target_arch = "wasm32")]
    fn modified(&self, _path: &str) -> Option<SystemTime> {
        None
    }
}

/// Assets embutidos no binário em tempo de compilação — o que o alvo web
/// precisa, porque o navegador não tem disco para o [`DiskAssets`] ler. Serve
/// igual para um binário nativo standalone.
///
/// Monte com [`embed_assets!`](crate::embed_assets), que chama `include_bytes!`
/// para cada caminho e guarda o mesmo texto de caminho que o markup usa: o
/// `Template::File("examples/app/app.gv")` e o `<link href="examples/app/app.gss">`
/// acham seus arquivos sem mudar uma letra.
///
/// Um arquivo que ficou fora da lista é um erro de leitura igual ao de um
/// arquivo que falta no disco — o registro do componente diz qual.
/// [`modified`](AssetSource::modified) é sempre `None`: sem hot-reload.
#[derive(Debug, Clone, Copy)]
pub struct EmbeddedAssets {
    arquivos: &'static [(&'static str, &'static [u8])],
}

impl EmbeddedAssets {
    /// Prefira [`embed_assets!`](crate::embed_assets); este construtor é para
    /// quem já tem a tabela montada de outro jeito (um `build.rs`, por exemplo).
    pub const fn new(arquivos: &'static [(&'static str, &'static [u8])]) -> Self {
        Self { arquivos }
    }

    /// Compara pela chave normalizada, para `./a/b.gv` e `a/x/../b.gv` acharem
    /// o `a/b.gv` embutido — as mesmas grafias que chegam de um `href` relativo.
    fn buscar(&self, path: &str) -> Option<&'static [u8]> {
        let chave = normalize_key(std::path::Path::new(path));
        self.arquivos
            .iter()
            .find(|(caminho, _)| normalize_key(std::path::Path::new(caminho)) == chave)
            .map(|(_, bytes)| *bytes)
    }

    fn faltando(path: &str) -> std::io::Error {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("'{path}' não está entre os assets embutidos (faltou no embed_assets!?)"),
        )
    }
}

impl AssetSource for EmbeddedAssets {
    fn read_bytes(&self, path: &str) -> std::io::Result<Cow<'static, [u8]>> {
        self.buscar(path).map(Cow::Borrowed).ok_or_else(|| Self::faltando(path))
    }

    fn read_to_string(&self, path: &str) -> std::io::Result<Cow<'static, str>> {
        let bytes = self.buscar(path).ok_or_else(|| Self::faltando(path))?;
        std::str::from_utf8(bytes)
            .map(Cow::Borrowed)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    fn exists(&self, path: &str) -> bool {
        self.buscar(path).is_some()
    }

    fn modified(&self, _path: &str) -> Option<SystemTime> {
        None
    }

    fn supports_reload(&self) -> bool {
        false
    }
}

/// Monta um [`EmbeddedAssets`] a partir de caminhos relativos à raiz do crate
/// que chama a macro (`CARGO_MANIFEST_DIR`), que é de onde o `cargo run` resolve
/// os mesmos caminhos no disco.
///
/// ```ignore
/// GlacierDaemon::new().assets(std::sync::Arc::new(glacier_ui::embed_assets![
///     "views/app.gv",
///     "views/app.gss",
/// ]))
/// ```
#[macro_export]
macro_rules! embed_assets {
    ($($caminho:literal),* $(,)?) => {
        $crate::EmbeddedAssets::new(&[
            $((
                $caminho,
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $caminho)) as &'static [u8],
            )),*
        ])
    };
}

/// Colapsa `.`/`..` **lexicalmente** e unifica separadores em `/`, produzindo
/// uma chave estável de identidade sem tocar o filesystem. Usado tanto pelo
/// cache de `require` (deste módulo) quanto por
/// [`GlacierUI::resolve_import_href`](crate::GlacierUI::resolve_import_href)
/// (`href` de `<link rel="import">` relativo ao arquivo importador — mesma
/// ideia de resolução, fora do universo Luau).
///
/// Antes o cache de `require` usava [`Path::canonicalize`], que exige o arquivo
/// existir no disco; com uma fonte de assets embutida não há disco, então a
/// identidade tem de ser derivada só do texto do caminho. Preserva uma `/`
/// inicial (caminho absoluto vindo de `GLACIER_LUAU_PATH`).
pub(crate) fn normalize_key(path: &std::path::Path) -> String {
    use std::path::Component;
    let mut absolute = false;
    let mut parts: Vec<String> = Vec::new();
    for comp in path.components() {
        match comp {
            Component::RootDir => absolute = true,
            Component::Prefix(p) => parts.push(p.as_os_str().to_string_lossy().into_owned()),
            Component::CurDir => {}
            Component::ParentDir => {
                if matches!(parts.last().map(String::as_str), Some(s) if s != "..") {
                    parts.pop();
                } else if !absolute {
                    parts.push("..".into());
                }
            }
            Component::Normal(s) => parts.push(s.to_string_lossy().into_owned()),
        }
    }
    let joined = parts.join("/");
    if absolute {
        format!("/{joined}")
    } else {
        joined
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static ARQUIVOS: &[(&str, &[u8])] = &[
        ("views/app.gv", b"<screen/>"),
        ("views/binario.png", &[0xff, 0xfe]),
    ];

    #[test]
    fn acha_o_arquivo_pelas_grafias_de_um_href_relativo() {
        let assets = EmbeddedAssets::new(ARQUIVOS);
        for grafia in ["views/app.gv", "./views/app.gv", "views/outra/../app.gv"] {
            assert_eq!(assets.read_to_string(grafia).unwrap(), "<screen/>", "{grafia}");
            assert!(assets.exists(grafia), "{grafia}");
        }
    }

    #[test]
    fn arquivo_fora_da_lista_e_not_found_com_o_caminho_na_mensagem() {
        let erro = EmbeddedAssets::new(ARQUIVOS).read_to_string("views/falta.gss").unwrap_err();
        assert_eq!(erro.kind(), std::io::ErrorKind::NotFound);
        assert!(erro.to_string().contains("views/falta.gss"), "{erro}");
    }

    #[test]
    fn binario_nao_utf8_le_como_bytes_e_falha_como_texto() {
        let assets = EmbeddedAssets::new(ARQUIVOS);
        assert_eq!(&*assets.read_bytes("views/binario.png").unwrap(), &[0xff, 0xfe]);
        assert_eq!(
            assets.read_to_string("views/binario.png").unwrap_err().kind(),
            std::io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn embutido_nunca_recarrega() {
        let assets = EmbeddedAssets::new(ARQUIVOS);
        assert!(assets.modified("views/app.gv").is_none());
        assert!(!assets.supports_reload());
    }

    #[test]
    fn a_macro_embute_pelo_caminho_relativo_ao_crate() {
        let assets = crate::embed_assets!["examples/web_contador/app.gv"];
        let gv = assets.read_to_string("examples/web_contador/app.gv").unwrap();
        assert!(gv.contains("<screen"), "{gv}");
    }
}
