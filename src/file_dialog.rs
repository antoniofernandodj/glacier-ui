//! Diálogo de arquivo/pasta **nativo do SO** (Open/OpenMultiple/Save/
//! Directory), via a crate `rfd`. Ao contrário de [`crate::dialogs`] (um
//! overlay desenhado pelo próprio motor), este é um diálogo de verdade do
//! sistema operacional — sem controle visual do glacier-ui sobre ele, e
//! assíncrono (o processo de UI não pode bloquear enquanto ele está aberto).
//!
//! Exposto à camada Luau como `open_file`/`open_files`/`save_file`/
//! `pick_folder` (ver [`crate::luau`]), seguindo exatamente o mesmo padrão
//! suspensivo de `confirm()`/`fetch()`: a corrotina Lua cede um pedido, o
//! motor faz o trabalho de verdade (aqui, `run()` abaixo) fora da Lua, e
//! retoma a corrotina com o resultado — dando a impressão de `async/await`
//! síncrono (`local caminho = open_file{...}`).

/// Qual variante do diálogo mostrar. Só muda quais métodos de
/// `rfd::AsyncFileDialog` são chamados em [`run`] — a `FileDialogSpec` é a
/// mesma para os quatro.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileDialogMode {
    /// Escolher um único arquivo existente.
    Open,
    /// Escolher um ou mais arquivos existentes.
    OpenMultiple,
    /// Escolher (ou digitar) o caminho de um arquivo a salvar — não precisa
    /// existir ainda.
    Save,
    /// Escolher um diretório existente.
    Directory,
}

/// Um filtro de extensão (`rfd::AsyncFileDialog::add_filter`), ex.:
/// `FileFilter { name: "Imagens".into(), extensions: vec!["png".into(), "jpg".into()] }`.
#[derive(Debug, Clone)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

/// O que pedir ao diálogo. Construído a partir do pedido Lua (ver
/// `luau::build_file_dialog`); `filters`/`default_name` só fazem sentido em
/// alguns modos (ex.: `default_name` em `Save`), mas não custa nada deixá-los
/// disponíveis nos demais — `rfd` simplesmente os ignora se não se aplicam.
#[derive(Debug, Clone)]
pub struct FileDialogSpec {
    pub mode: FileDialogMode,
    pub title: Option<String>,
    pub starting_dir: Option<String>,
    /// Nome sugerido inicial — relevante sobretudo em `Save`.
    pub default_name: Option<String>,
    pub filters: Vec<FileFilter>,
}

/// O caminho (ou caminhos) escolhidos, ou a ausência deles se o usuário
/// cancelou — a forma espelha o [`FileDialogMode`] que gerou o pedido:
/// `Open`/`Save`/`Directory` sempre devolvem `Path`, `OpenMultiple` sempre
/// devolve `Paths`. Convertido para um valor Lua em
/// `luau::resume_file_dialog_inner` (`Path(None)`/`Paths(None)` viram
/// `nil`, o cancelamento "silencioso" que `confirm()` dá com `false`).
#[derive(Debug, Clone)]
pub enum FileDialogResult {
    Path(Option<String>),
    Paths(Option<Vec<String>>),
}

/// Mostra o diálogo nativo do SO e espera a escolha do usuário. Não bloqueia
/// a thread de UI — chamado via `iced::Task::perform` em
/// `GlacierUI::run_on_owner`, exatamente como `net::perform` já é para
/// `fetch()`.
pub async fn run(spec: FileDialogSpec) -> FileDialogResult {
    let mut dialog = rfd::AsyncFileDialog::new();
    if let Some(t) = &spec.title {
        dialog = dialog.set_title(t);
    }
    if let Some(d) = &spec.starting_dir {
        dialog = dialog.set_directory(d);
    }
    if let Some(n) = &spec.default_name {
        dialog = dialog.set_file_name(n);
    }
    for f in &spec.filters {
        let extensions: Vec<&str> = f.extensions.iter().map(String::as_str).collect();
        dialog = dialog.add_filter(&f.name, &extensions);
    }

    let path_of = |handle: rfd::FileHandle| handle.path().to_string_lossy().into_owned();

    match spec.mode {
        FileDialogMode::Open => {
            FileDialogResult::Path(dialog.pick_file().await.map(path_of))
        }
        FileDialogMode::OpenMultiple => FileDialogResult::Paths(
            dialog
                .pick_files()
                .await
                .map(|handles| handles.into_iter().map(path_of).collect()),
        ),
        FileDialogMode::Save => {
            FileDialogResult::Path(dialog.save_file().await.map(path_of))
        }
        FileDialogMode::Directory => {
            FileDialogResult::Path(dialog.pick_folder().await.map(path_of))
        }
    }
}
