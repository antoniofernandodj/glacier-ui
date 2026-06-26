use std::collections::HashMap;
use crate::parser::{NoUI, TipoNo};

/// Process string template by replacing `{key}` placeholders with values from context
pub fn processar_template(template: &str, context: &HashMap<String, String>) -> String {
    let mut result = String::new();
    let mut chars = template.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            let mut key = String::new();
            let mut closed = false;
            while let Some(&nc) = chars.peek() {
                if nc == '}' {
                    chars.next(); // Consume '}'
                    closed = true;
                    break;
                } else {
                    key.push(chars.next().unwrap());
                }
            }
            if closed {
                if let Some(val) = context.get(&key) {
                    result.push_str(val);
                } else {
                    // Placeholder key not found; we leave it as is or empty. Let's make it empty.
                }
            } else {
                result.push('{');
                result.push_str(&key);
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Recursively evaluate a NoUI tree, resolving templates and placeholders.
pub fn evaluate_node(
    no: &NoUI,
    context: &HashMap<String, String>,
    templates: &HashMap<String, NoUI>,
) -> Result<NoUI, String> {
    // A component reference — either the legacy `<Include src="..." />` or a tag
    // named after a registered component (e.g. `<PerfilCard ... />`) — is replaced
    // with the evaluated template root, with its attributes passed in as props.
    let referencia: Option<(&String, &HashMap<String, String>)> = match &no.tipo {
        TipoNo::Include { src, props } => Some((src, props)),
        TipoNo::Component { name, props } => Some((name, props)),
        _ => None,
    };
    if let Some((nome, props)) = referencia {
        let template_ast = templates.get(nome)
            .ok_or_else(|| format!("Component '{}' not registered", nome))?;

        // Create a local context by copying the parent context and merging evaluated properties
        let mut local_context = context.clone();
        for (key, val_template) in props {
            let evaluated_val = processar_template(val_template, context);
            local_context.insert(key.clone(), evaluated_val);
        }

        // Recursively evaluate the referenced template root node
        return evaluate_node(template_ast, &local_context, templates);
    }

    // Evaluate current node attributes
    let tipo_eval = match &no.tipo {
        TipoNo::Container => TipoNo::Container,
        TipoNo::Column => TipoNo::Column,
        TipoNo::Row => TipoNo::Row,
        TipoNo::Text { content, size, bold, color } => {
            TipoNo::Text {
                content: processar_template(content, context),
                size: *size,
                bold: *bold,
                color: color.as_ref().map(|c| processar_template(c, context)),
            }
        }
        TipoNo::Button { text, on_click, color } => {
            TipoNo::Button {
                text: processar_template(text, context),
                on_click: on_click.as_ref().map(|o| processar_template(o, context)),
                color: color.as_ref().map(|c| processar_template(c, context)),
            }
        }
        TipoNo::TextInput { placeholder, value_var, on_change } => {
            TipoNo::TextInput {
                placeholder: processar_template(placeholder, context),
                value_var: processar_template(value_var, context),
                on_change: processar_template(on_change, context),
            }
        }
        TipoNo::Image { source, clip_circle } => {
            TipoNo::Image {
                source: processar_template(source, context),
                clip_circle: *clip_circle,
            }
        }
        TipoNo::Include { .. } | TipoNo::Component { .. } | TipoNo::ForEach { .. } => {
            TipoNo::Container
        }
    };

    let largura_eval = no.largura.as_ref().map(|s| processar_template(s, context));
    let altura_eval = no.altura.as_ref().map(|s| processar_template(s, context));
    let padding_eval = no.padding.as_ref().map(|s| processar_template(s, context));
    let align_x_eval = no.align_x.as_ref().map(|s| processar_template(s, context));
    let align_y_eval = no.align_y.as_ref().map(|s| processar_template(s, context));
    let background_eval = no.background.as_ref().map(|s| processar_template(s, context));
    let border_color_eval = no.border_color.as_ref().map(|s| processar_template(s, context));

    // Evaluate children recursively
    let mut filhos_eval = Vec::new();
    for filho in &no.filhos {
        if let TipoNo::ForEach { items, var } = &filho.tipo {
            let items_evaluated = processar_template(items, context);
            if let Some(json_str) = context.get(&items_evaluated) {
                if let Ok(serde_json::Value::Array(arr)) = serde_json::from_str::<serde_json::Value>(json_str) {
                    for item in arr {
                        let mut local_context = context.clone();
                        match item {
                            serde_json::Value::Object(obj) => {
                                for (key, val) in obj {
                                    let str_val = match val {
                                        serde_json::Value::String(s) => s,
                                        other => other.to_string(),
                                    };
                                    local_context.insert(format!("{}.{}", var, key), str_val);
                                }
                            }
                            serde_json::Value::String(s) => {
                                local_context.insert(var.clone(), s);
                            }
                            other => {
                                local_context.insert(var.clone(), other.to_string());
                            }
                        }
                        for sub_filho in &filho.filhos {
                            filhos_eval.push(evaluate_node(sub_filho, &local_context, templates)?);
                        }
                    }
                }
            }
        } else {
            filhos_eval.push(evaluate_node(filho, context, templates)?);
        }
    }

    Ok(NoUI {
        tipo: tipo_eval,
        filhos: filhos_eval,
        largura: largura_eval,
        altura: altura_eval,
        padding: padding_eval,
        align_x: align_x_eval,
        align_y: align_y_eval,
        spacing: no.spacing,
        background: background_eval,
        border_radius: no.border_radius,
        border_width: no.border_width,
        border_color: border_color_eval,
    })
}
