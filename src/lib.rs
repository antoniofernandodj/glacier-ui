pub mod parser;
pub mod eval;
pub mod widget;

pub use parser::{NoUI, TipoNo};
pub use eval::{evaluate_node, processar_template};
pub use widget::{renderizar_no, MensagemMotor};

use std::collections::HashMap;
use std::time::{SystemTime, Duration};

/// The XML-to-UI rendering engine
#[derive(Debug, Clone)]
pub struct MotorUI {
    /// Maps a component name (e.g. "perfil") to its XML file path
    pub registered_components: HashMap<String, String>,
    /// Cache of parsed component AST trees
    pub parsed_templates: HashMap<String, NoUI>,
    /// Cache of fully evaluated component AST trees (placeholders substituted, includes resolved)
    pub evaluated_templates: HashMap<String, NoUI>,
    /// In-memory context data for state binding
    pub contexto_dados: HashMap<String, String>,
    /// File modification times to support hot reloading
    pub file_mod_times: HashMap<String, SystemTime>,
}

impl MotorUI {
    /// Creates a new, empty MotorUI instance
    pub fn new() -> Self {
        Self {
            registered_components: HashMap::new(),
            parsed_templates: HashMap::new(),
            evaluated_templates: HashMap::new(),
            contexto_dados: HashMap::new(),
            file_mod_times: HashMap::new(),
        }
    }

    /// Registers a component and parses its XML file immediately
    pub fn registrar_componente(&mut self, nome: &str, path: &str) -> Result<(), String> {
        let xml_content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read XML file at '{}': {}", path, e))?;
        
        let ast = NoUI::parse_xml(&xml_content)
            .map_err(|e| format!("Failed to parse XML for component '{}': {}", nome, e))?;

        let mod_time = std::fs::metadata(path)
            .and_then(|m| m.modified())
            .unwrap_or_else(|_| SystemTime::now());

        self.registered_components.insert(nome.to_string(), path.to_string());
        self.parsed_templates.insert(nome.to_string(), ast);
        self.file_mod_times.insert(nome.to_string(), mod_time);

        // Re-evaluate all templates with new registered component
        let _ = self.reavaliar_todos();

        Ok(())
    }

    /// Defines or updates a value in the state context and re-evaluates all templates
    pub fn definir_dado(&mut self, chave: &str, valor: &str) {
        self.contexto_dados.insert(chave.to_string(), valor.to_string());
        let _ = self.reavaliar_todos();
    }

    /// Gets a value from the state context
    pub fn obter_dado(&self, chave: &str) -> Option<&String> {
        self.contexto_dados.get(chave)
    }

    /// Gets a mutable reference to a value in the state context.
    /// Note: if you modify values, you should call `reavaliar_todos()` manually.
    pub fn obter_dado_mut(&mut self, chave: &str) -> Option<&mut String> {
        self.contexto_dados.get_mut(chave)
    }

    /// Re-evaluates all templates with the current context and caches them
    pub fn reavaliar_todos(&mut self) -> Result<(), String> {
        let mut evals = HashMap::new();
        for (nome, template_ast) in &self.parsed_templates {
            let evaluated_ast = evaluate_node(template_ast, &self.contexto_dados, &self.parsed_templates)?;
            evals.insert(nome.clone(), evaluated_ast);
        }
        self.evaluated_templates = evals;
        Ok(())
    }

    /// Recursively evaluates the component and translates it into an Iced Element
    pub fn renderizar<'a>(&'a self, nome_componente: &str) -> Result<iced::Element<'a, MensagemMotor>, String> {
        let evaluated_ast = self.evaluated_templates.get(nome_componente)
            .ok_or_else(|| format!("Component '{}' is not evaluated or registered", nome_componente))?;

        // Render the evaluated AST to Iced Widgets
        Ok(renderizar_no(evaluated_ast, &self.contexto_dados))
    }

    /// Checks registered XML files for changes and re-parses them if modified.
    /// Returns the list of component names that were reloaded.
    pub fn verificar_recarregamento(&mut self) -> Vec<String> {
        let mut reloaded = Vec::new();
        let mut updates = Vec::new();

        for (nome, path) in &self.registered_components {
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(modified) = metadata.modified() {
                    let last_modified = self.file_mod_times.get(nome);
                    if last_modified.map_or(true, |&last| modified > last) {
                        // File changed, reload it
                        if let Ok(xml_content) = std::fs::read_to_string(path) {
                            if let Ok(new_ast) = NoUI::parse_xml(&xml_content) {
                                updates.push((nome.clone(), new_ast, modified));
                                reloaded.push(nome.clone());
                            }
                        }
                    }
                }
            }
        }

        if !updates.is_empty() {
            // Apply changes
            for (nome, new_ast, modified) in updates {
                self.parsed_templates.insert(nome.clone(), new_ast);
                self.file_mod_times.insert(nome, modified);
            }
            // Re-evaluate all templates
            let _ = self.reavaliar_todos();
        }

        reloaded
    }

    /// Returns a Subscription that ticks periodically to trigger file reloading checks.
    /// The client application should map this subscription to call `verificar_recarregamento`.
    pub fn subscricao_recarregamento(periodo: Duration) -> iced::Subscription<MensagemMotor> {
        iced::time::every(periodo).map(|_| MensagemMotor::FileChanged("".to_string()))
    }
}
