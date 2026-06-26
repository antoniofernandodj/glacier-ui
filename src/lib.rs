pub mod parser;
pub mod eval;
pub mod widget;

pub use parser::{UiNode, NodeType};
pub use eval::{evaluate_node, process_template};
pub use widget::{render_node, EngineMessage};

use std::collections::HashMap;
use std::time::{SystemTime, Duration};

/// The XML-to-UI rendering engine
#[derive(Debug, Clone)]
pub struct UiEngine {
    /// Maps a component name (e.g. "perfil") to its XML file path
    pub registered_components: HashMap<String, String>,
    /// Cache of parsed component AST trees
    pub parsed_templates: HashMap<String, UiNode>,
    /// Cache of fully evaluated component AST trees (placeholders substituted, includes resolved)
    pub evaluated_templates: HashMap<String, UiNode>,
    /// In-memory context data for state binding
    pub context_data: HashMap<String, String>,
    /// File modification times to support hot reloading
    pub file_mod_times: HashMap<String, SystemTime>,
    /// Name of the component currently shown as the active screen
    pub current_screen: Option<String>,
    /// Navigation history (stack of previous screens) used by `navigate_back`
    pub history: Vec<String>,
}

impl UiEngine {
    /// Creates a new, empty UiEngine instance
    pub fn new() -> Self {
        Self {
            registered_components: HashMap::new(),
            parsed_templates: HashMap::new(),
            evaluated_templates: HashMap::new(),
            context_data: HashMap::new(),
            file_mod_times: HashMap::new(),
            current_screen: None,
            history: Vec::new(),
        }
    }

    /// Sets the initial active screen, clearing any navigation history.
    pub fn set_initial_screen(&mut self, name: &str) {
        self.current_screen = Some(name.to_string());
        self.history.clear();
    }

    /// Navigates to a new screen, pushing the current one onto the history stack.
    /// Navigating to the screen already shown is a no-op (avoids duplicate history).
    pub fn navigate_to(&mut self, name: &str) {
        if let Some(current) = &self.current_screen {
            if current == name {
                return;
            }
            self.history.push(current.clone());
        }
        self.current_screen = Some(name.to_string());
    }

    /// Returns to the previous screen in the history, if any.
    pub fn navigate_back(&mut self) {
        if let Some(previous) = self.history.pop() {
            self.current_screen = Some(previous);
        }
    }

    /// Renders the current active screen.
    pub fn render_current(&self) -> Result<iced::Element<'_, EngineMessage>, String> {
        let name = self.current_screen.as_ref()
            .ok_or_else(|| "No active screen defined; call set_initial_screen first".to_string())?;
        self.render(name)
    }

    /// Registers a component from its XML file, recursively loading any
    /// components it declares via `<import name="..." from="..." />`.
    pub fn register_component(&mut self, name: &str, path: &str) -> Result<(), String> {
        self.register_component_inner(name, path)?;
        // Evaluate once, after the whole import graph has been loaded.
        let _ = self.reevaluate_all();
        Ok(())
    }

    /// Parses and stores a component plus its imports, without re-evaluating.
    fn register_component_inner(&mut self, name: &str, path: &str) -> Result<(), String> {
        let xml_content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read XML file at '{}': {}", path, e))?;

        let ast = UiNode::parse_xml(&xml_content)
            .map_err(|e| format!("Failed to parse XML for component '{}': {}", name, e))?;

        let mod_time = std::fs::metadata(path)
            .and_then(|m| m.modified())
            .unwrap_or_else(|_| SystemTime::now());

        self.registered_components.insert(name.to_string(), path.to_string());
        self.parsed_templates.insert(name.to_string(), ast.clone());
        self.file_mod_times.insert(name.to_string(), mod_time);

        // Recursively load components declared with `<import>`.
        self.load_imports(&ast)?;

        Ok(())
    }

    /// Walks a parsed tree and registers every `<import>`ed component not yet loaded.
    fn load_imports(&mut self, node: &UiNode) -> Result<(), String> {
        if let NodeType::Import { name, from } = &node.kind {
            if !self.parsed_templates.contains_key(name) {
                let (name, from) = (name.clone(), from.clone());
                self.register_component_inner(&name, &from)?;
            }
        }
        for child in &node.children {
            self.load_imports(child)?;
        }
        Ok(())
    }

    /// Defines or updates a value in the state context and re-evaluates all templates
    pub fn define_data(&mut self, key: &str, value: &str) {
        self.context_data.insert(key.to_string(), value.to_string());
        let _ = self.reevaluate_all();
    }

    /// Gets a value from the state context
    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.context_data.get(key)
    }

    /// Gets a mutable reference to a value in the state context.
    /// Note: if you modify values, you should call `reevaluate_all()` manually.
    pub fn get_data_mut(&mut self, key: &str) -> Option<&mut String> {
        self.context_data.get_mut(key)
    }

    /// Re-evaluates all templates with the current context and caches them
    pub fn reevaluate_all(&mut self) -> Result<(), String> {
        let mut evals = HashMap::new();
        for (name, template_ast) in &self.parsed_templates {
            let evaluated_ast = evaluate_node(template_ast, &self.context_data, &self.parsed_templates)?;
            evals.insert(name.clone(), evaluated_ast);
        }
        self.evaluated_templates = evals;
        Ok(())
    }

    /// Recursively evaluates the component and translates it into an Iced Element
    pub fn render<'a>(&'a self, component_name: &str) -> Result<iced::Element<'a, EngineMessage>, String> {
        let evaluated_ast = self.evaluated_templates.get(component_name)
            .ok_or_else(|| format!("Component '{}' is not evaluated or registered", component_name))?;

        // Render the evaluated AST to Iced Widgets
        Ok(render_node(evaluated_ast, &self.context_data))
    }

    /// Checks registered XML files for changes and re-parses them if modified.
    /// Returns the list of component names that were reloaded.
    pub fn check_reload(&mut self) -> Vec<String> {
        let mut reloaded = Vec::new();
        let mut updates = Vec::new();

        for (name, path) in &self.registered_components {
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(modified) = metadata.modified() {
                    let last_modified = self.file_mod_times.get(name);
                    if last_modified.map_or(true, |&last| modified > last) {
                        // File changed, reload it
                        if let Ok(xml_content) = std::fs::read_to_string(path) {
                            if let Ok(new_ast) = UiNode::parse_xml(&xml_content) {
                                updates.push((name.clone(), new_ast, modified));
                                reloaded.push(name.clone());
                            }
                        }
                    }
                }
            }
        }

        if !updates.is_empty() {
            // Apply changes
            for (name, new_ast, modified) in updates {
                // Pick up any newly-added `<import>` declarations.
                let _ = self.load_imports(&new_ast);
                self.parsed_templates.insert(name.clone(), new_ast);
                self.file_mod_times.insert(name, modified);
            }
            // Re-evaluate all templates
            let _ = self.reevaluate_all();
        }

        reloaded
    }

    /// Returns a Subscription that ticks periodically to trigger file reloading checks.
    /// The client application should map this subscription to call `check_reload`.
    pub fn reload_subscription(period: Duration) -> iced::Subscription<EngineMessage> {
        iced::time::every(period).map(|_| EngineMessage::FileChanged("".to_string()))
    }
}
