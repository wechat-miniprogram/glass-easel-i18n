use compact_str::CompactString;
use glass_easel_template_compiler::{
    parse::expr::Expression,
    parse::parse,
    parse::tag::{Element, ElementKind, Node, Value},
    parse::Position,
    stringify::{Stringifier, Stringify},
};
use serde::Deserialize;
use std::{collections::HashMap, ops::Range};
use toml;
mod js_bindings;

pub struct CompiledTemplate {
    pub output: String,
    pub source_map: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub struct TransContent {
    #[serde(flatten)]
    pub map: HashMap<String, HashMap<String, String>>,
}

pub fn compile(path: &str, source: &str, trans_source: &str) -> Result<CompiledTemplate, String> {
    // parse the template
    let (mut template, parse_state) = parse(path, source);
    for warning in parse_state.warnings() {
        if warning.prevent_success() {
            return Err(format!("Failed to compile template: {}", warning));
        }
    }
    let trans_content: TransContent = toml::from_str(&trans_source).unwrap();

    // transform the template to support i18n
    println!("template:{:#?}", template.content);

    fn contains_i18n_tag(node_list: &Vec<Node>) -> bool {
        for node in node_list {
            match node {
                Node::UnknownMetaTag(tag, ..) => {
                    if tag.starts_with("I18N") {
                        return true;
                    }
                    break;
                }
                _ => {}
            }
        }
        false
    }

    fn remove_i18n_tag(node_list: &Vec<Node>) -> Vec<Node> {
        let mut new_list = node_list.clone();
        if let Some(pos) = new_list.iter().position(
            |node| matches!(node, Node::UnknownMetaTag(tag, ..) if tag.starts_with("I18N")),
        ) {
            new_list.remove(pos);
        }
        new_list
    }

    fn contains_text_node(node_list: &Vec<Node>) -> bool {
        for node in node_list.iter() {
            match node {
                Node::Text(_) => return true,
                _ => {}
            }
        }
        false
    }

    fn translate_element_i18n(element: &mut Element, trans_content_map: &HashMap<String, String>) {
        match &mut element.kind {
            ElementKind::Normal { children, .. } => {
                if let Node::Text(ref mut text_node) = children[0] {
                    match text_node {
                        Value::Static { ref mut value, .. } => {
                            if let Some(translation) = trans_content_map.get(&value.to_string()) {
                                *value = translation.into();
                            }
                        }
                        _ => {}
                    }
                };
            }
            _ => {}
        }
    }

    // the Position of else_branch or branches just placed by the position of the template's first child
    fn get_first_child_position(template: &Vec<Node>) -> Option<Range<Position>> {
        let position: Option<Range<Position>>;
        match &template[0] {
            Node::Element(element) => {
                position = Some(element.close_location.clone());
            }
            Node::Text(value) => match value {
                Value::Dynamic {
                    double_brace_location,
                    ..
                } => {
                    let (first_location, _) = double_brace_location;
                    position = Some(first_location.clone());
                }
                Value::Static { location, .. } => {
                    position = Some(location.clone());
                }
            },
            Node::Comment(_, location) => {
                position = Some(location.clone());
            }
            Node::UnknownMetaTag(_, location) => {
                position = Some(location.clone());
            }
        }
        position
    }

    fn translate(node_list: &mut Vec<Node>, trans_content_map: &HashMap<String, String>) {
        for node in node_list {
            match node {
                Node::Element(element) => {
                    match &mut element.kind {
                        ElementKind::Normal { children, .. } => {
                            // current element only has one text child
                            if contains_text_node(&children) && children.len() == 1 {
                                translate_element_i18n(element, trans_content_map);
                            } else {
                                translate(children, trans_content_map);
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    if contains_i18n_tag(&template.content) {
        // let tenmplate_position = Rc::new(get_first_child_position(&template.content).unwrap());
        // Element::IF has two children: branches and else_branch
        let mut branches: Vec<(Range<Position>, Value, Vec<Node>)> = vec![];
        let branch_template = remove_i18n_tag(&template.content);
        let branch_position = get_first_child_position(&template.content).unwrap();
        for (lang, trans_content_map) in trans_content.map.iter() {
            let mut template_item = branch_template.clone();
            let eq_full = Box::new(Expression::EqFull {
                left: Box::new(Expression::DataField {
                    name: "lang".into(),
                    location: branch_position.clone(),
                }),
                right: Box::new(Expression::LitStr {
                    value: lang.into(),
                    location: branch_position.clone(),
                }),
                location: branch_position.clone(),
            });
            let branch_value = Value::Dynamic {
                expression: eq_full,
                double_brace_location: (branch_position.clone(), branch_position.clone()),
                binding_map_keys: None,
            };
            translate(&mut template_item, trans_content_map);
            branches.push((branch_position.clone(), branch_value, template_item));
        }

        // origin template only warpped with <block wx:else> </block> and is unnecessary to be translated by i18n
        let else_branch = Some((branch_position.clone(), branch_template.clone()));
        let template_i18n = Element {
            kind: ElementKind::If {
                branches,
                else_branch,
            },
            start_tag_location: (branch_position.clone(), branch_position.clone()),
            close_location: branch_position.clone(),
            end_tag_location: Some((branch_position.clone(), branch_position)),
        };
        template.content = vec![Node::Element(template_i18n)];
        // translate(&mut template.content, &trans_content.map);
    }

    // stringify the template
    let mut stringifier = Stringifier::new(String::new(), path, source);
    template
        .stringify_write(&mut stringifier)
        .map_err(|_| "Failed to write output")?;
    let (output, sm) = stringifier.finish();
    let mut source_map = vec![];
    sm.to_writer(&mut source_map)
        .map_err(|_| "Failed to write output")?;
    Ok(CompiledTemplate { output, source_map })
}
