use std::collections::HashMap;
use iced::widget::{button, column, row, text, container, text_input, image};
use iced::{Element, Length, Alignment, Color, Border, Padding, Background};
use crate::parser::{NoUI, TipoNo};

#[derive(Debug, Clone)]
pub enum MensagemMotor {
    XmlClick(String),
    XmlInputChanged { action: String, value: String },
    FileChanged(String),
}

/// Helper to parse iced::Length from optional string
fn parse_length(s: &Option<String>) -> Length {
    match s.as_deref() {
        Some("fill") | Some("Fill") | Some("FILL") => Length::Fill,
        Some("shrink") | Some("Shrink") | Some("SHRINK") => Length::Shrink,
        Some(val) => {
            if let Ok(f) = val.parse::<f32>() {
                Length::Fixed(f)
            } else {
                Length::Shrink
            }
        }
        None => Length::Shrink,
    }
}

/// Helper to parse Padding
fn parse_padding(s: &Option<String>) -> Padding {
    if let Some(p_str) = s {
        let parts: Vec<f32> = p_str
            .split_whitespace()
            .filter_map(|p| p.parse::<f32>().ok())
            .collect();
        match parts.len() {
            1 => Padding::new(parts[0]),
            2 => Padding {
                top: parts[0],
                right: parts[1],
                bottom: parts[0],
                left: parts[1],
            },
            4 => Padding {
                top: parts[0],
                right: parts[1],
                bottom: parts[2],
                left: parts[3],
            },
            _ => Padding::ZERO,
        }
    } else {
        Padding::ZERO
    }
}

/// Helper to parse alignment
fn parse_alignment(s: &Option<String>) -> Option<Alignment> {
    match s.as_deref() {
        Some("start") | Some("Start") | Some("START") => Some(Alignment::Start),
        Some("center") | Some("Center") | Some("CENTER") => Some(Alignment::Center),
        Some("end") | Some("End") | Some("END") => Some(Alignment::End),
        _ => None,
    }
}

/// Helper to parse hex colors like #RRGGBB or #RRGGBBAA
fn parse_hex_color(s: &str) -> Option<Color> {
    let s = s.trim().trim_start_matches('#');
    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&s[2..4], 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&s[4..6], 16).ok()? as f32 / 255.0;
        Some(Color::new(r, g, b, 1.0))
    } else if s.len() == 8 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&s[2..4], 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&s[4..6], 16).ok()? as f32 / 255.0;
        let a = u8::from_str_radix(&s[6..8], 16).ok()? as f32 / 255.0;
        Some(Color::new(r, g, b, a))
    } else {
        None
    }
}

/// Generate Iced widgets recursively from NoUI tree.
/// References to strings are borrowed directly from the AST node with lifetime 'a.
pub fn renderizar_no<'a>(
    no: &'a NoUI,
    contexto: &'a HashMap<String, String>,
) -> Element<'a, MensagemMotor> {
    let mut element: Element<'a, MensagemMotor> = match &no.tipo {
        TipoNo::Text { content, size, bold, color } => {
            let mut t = text(content.as_str());
            if let Some(s) = size {
                t = t.size(*s);
            }
            if *bold {
                t = t.font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                });
            }
            if let Some(c_str) = color {
                if let Some(col) = parse_hex_color(c_str) {
                    t = t.color(col);
                }
            }
            t.width(parse_length(&no.largura))
             .height(parse_length(&no.altura))
             .into()
        }
        TipoNo::Button { text: btn_text, on_click, color } => {
            let t = text(btn_text.as_str());
            let mut btn = button(t);
            if let Some(action) = on_click {
                btn = btn.on_press(MensagemMotor::XmlClick(action.clone()));
            }
            
            if let Some(c_str) = color {
                if let Some(col) = parse_hex_color(c_str) {
                    btn = btn.style(move |_theme, status| {
                        let bg = match status {
                            iced::widget::button::Status::Hovered => Some(Background::Color(Color {
                                r: (col.r * 1.1).min(1.0),
                                g: (col.g * 1.1).min(1.0),
                                b: (col.b * 1.1).min(1.0),
                                a: col.a,
                            })),
                            iced::widget::button::Status::Pressed => Some(Background::Color(Color {
                                r: (col.r * 0.9).min(1.0),
                                g: (col.g * 0.9).min(1.0),
                                b: (col.b * 0.9).min(1.0),
                                a: col.a,
                            })),
                            _ => Some(Background::Color(col)),
                        };
                        iced::widget::button::Style {
                            background: bg,
                            text_color: Color::WHITE,
                            border: Border::default(),
                            shadow: iced::Shadow::default(),
                        }
                    });
                }
            }
            
            btn.width(parse_length(&no.largura))
               .height(parse_length(&no.altura))
               .padding(parse_padding(&no.padding))
               .into()
        }
        TipoNo::TextInput { placeholder, value_var, on_change } => {
            let current_value = contexto.get(value_var).map(|s| s.as_str()).unwrap_or("");
            let action_clone = on_change.clone();
            
            let mut input = text_input(placeholder.as_str(), current_value)
                .on_input(move |val| MensagemMotor::XmlInputChanged {
                    action: action_clone.clone(),
                    value: val,
                });
            
            input = input.width(parse_length(&no.largura))
                         .padding(parse_padding(&no.padding));

            let mut elem: Element<'a, MensagemMotor> = input.into();
            
            if no.altura.is_some() {
                elem = container(elem)
                    .height(parse_length(&no.altura))
                    .align_y(Alignment::Center)
                    .into();
            }
            elem
        }
        TipoNo::Image { source, clip_circle } => {
            let handle = image::Handle::from_path(source.clone());
            let img = image(handle);
            
            let w_len = parse_length(&no.largura);
            let h_len = parse_length(&no.altura);

            if *clip_circle {
                let w_val = no.largura.as_ref().and_then(|s| s.parse::<f32>().ok()).unwrap_or(80.0);
                let h_val = no.altura.as_ref().and_then(|s| s.parse::<f32>().ok()).unwrap_or(80.0);
                let radius = w_val.min(h_val) / 2.0;

                let clipped_img = img.width(Length::Fixed(w_val)).height(Length::Fixed(h_val));
                container(clipped_img)
                    .width(Length::Fixed(w_val))
                    .height(Length::Fixed(h_val))
                    .clip(true)
                    .style(move |_theme| {
                        container::Style {
                            border: Border {
                                radius: iced::border::Radius::new(radius),
                                width: 0.0,
                                color: Color::TRANSPARENT,
                            },
                            ..Default::default()
                        }
                    })
                    .into()
            } else {
                img.width(w_len).height(h_len).into()
            }
        }
        TipoNo::Column => {
            let mut col = column![];
            
            if let Some(align_val) = parse_alignment(&no.align_x) {
                col = col.align_x(align_val);
            }
            
            if let Some(sp) = no.spacing {
                col = col.spacing(sp);
            }
            
            col = col.padding(parse_padding(&no.padding));

            for filho in &no.filhos {
                col = col.push(renderizar_no(filho, contexto));
            }
            
            col.width(parse_length(&no.largura))
               .height(parse_length(&no.altura))
               .into()
        }
        TipoNo::Row => {
            let mut r = row![];
            
            if let Some(align_val) = parse_alignment(&no.align_y) {
                r = r.align_y(align_val);
            }
            
            if let Some(sp) = no.spacing {
                r = r.spacing(sp);
            }
            
            r = r.padding(parse_padding(&no.padding));

            for filho in &no.filhos {
                r = r.push(renderizar_no(filho, contexto));
            }
            
            r.width(parse_length(&no.largura))
             .height(parse_length(&no.altura))
             .into()
        }
        TipoNo::Container => {
            let child: Element<'a, MensagemMotor> = if let Some(first_child) = no.filhos.first() {
                renderizar_no(first_child, contexto)
            } else {
                column![].into()
            };

            let mut c = container(child);
            c = c.width(parse_length(&no.largura))
                 .height(parse_length(&no.altura))
                 .padding(parse_padding(&no.padding));

            if let Some(ax) = parse_alignment(&no.align_x) {
                c = c.align_x(ax);
            }
            if let Some(ay) = parse_alignment(&no.align_y) {
                c = c.align_y(ay);
            }

            let bg_opt = no.background.as_ref().and_then(|bg| parse_hex_color(bg));
            let br_opt = no.border_radius;
            let bw_opt = no.border_width.unwrap_or(0.0);
            let bc_opt = no.border_color.as_ref().and_then(|bc| parse_hex_color(bc));

            if bg_opt.is_some() || br_opt.is_some() || bw_opt > 0.0 {
                c = c.style(move |_theme| {
                    container::Style {
                        background: bg_opt.map(|col| Background::Color(col)),
                        border: Border {
                            radius: iced::border::Radius::new(br_opt.unwrap_or(0.0)),
                            width: bw_opt,
                            color: bc_opt.unwrap_or(Color::TRANSPARENT),
                        },
                        ..Default::default()
                    }
                });
            }
            
            c.into()
        }
        TipoNo::Include { .. } => {
            container(text("Unresolved Include").color(Color::from_rgb(1.0, 0.0, 0.0))).into()
        }
        TipoNo::Component { name, .. } => {
            container(text(format!("Unresolved component <{}>", name)).color(Color::from_rgb(1.0, 0.0, 0.0))).into()
        }
        TipoNo::ForEach { .. } => {
            // ForEach is expanded during evaluation; nothing to render directly.
            column![].into()
        }
    };

    // Wrap elements other than Container in a Container if background/borders are specified
    if no.tipo != TipoNo::Container {
        let bg_opt = no.background.as_ref().and_then(|bg| parse_hex_color(bg));
        let br_opt = no.border_radius;
        let bw_opt = no.border_width.unwrap_or(0.0);
        let bc_opt = no.border_color.as_ref().and_then(|bc| parse_hex_color(bc));

        if bg_opt.is_some() || br_opt.is_some() || bw_opt > 0.0 {
            let mut c = container(element);
            c = c.width(parse_length(&no.largura))
                 .height(parse_length(&no.altura));
            
            if let Some(ax) = parse_alignment(&no.align_x) {
                c = c.align_x(ax);
            }
            if let Some(ay) = parse_alignment(&no.align_y) {
                c = c.align_y(ay);
            }

            c = c.style(move |_theme| {
                container::Style {
                    background: bg_opt.map(|col| Background::Color(col)),
                    border: Border {
                        radius: iced::border::Radius::new(br_opt.unwrap_or(0.0)),
                        width: bw_opt,
                        color: bc_opt.unwrap_or(Color::TRANSPARENT),
                    },
                    ..Default::default()
                }
            });
            element = c.into();
        }
    }

    element
}
