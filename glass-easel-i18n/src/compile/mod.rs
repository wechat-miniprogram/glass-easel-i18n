use super::contains_i18n_translate_children;
use glass_easel_template_compiler::{
    parse::expr::Expression,
    parse::parse,
    parse::tag::{Attribute, Element, ElementKind, Node, Value},
    parse::Position,
    stringify::{Stringifier, Stringify},
};
use regex::Regex;
use serde::Deserialize;
use std::{collections::HashMap, ops::Range};
use toml;

pub struct CompiledTemplate {
    pub output: String,
    pub source_map: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub struct TransContent {
    #[serde(flatten)]
    pub map: HashMap<String, HashMap<String, String>>,
}

pub fn compile(
    path: &str,
    source: &str,
    trans_source: &str,
    included_attributes: Vec<String>,
) -> Result<CompiledTemplate, String> {
    // parse the template
    let (mut template, parse_state) = parse(path, source);
    for warning in parse_state.warnings() {
        if warning.prevent_success() {
            return Err(format!("Failed to compile template: {}", warning));
        }
    }
    let trans_content: TransContent = toml::from_str(&trans_source).unwrap();
    // transform the template to support i18n

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

    fn remove_i18n_translate_children(node_list: &mut Vec<Node>) {
        node_list.iter_mut().for_each(|node| {
            if let Node::Element(element) = node {
                if let ElementKind::Normal { children, .. } = &mut element.kind {
                    remove_i18n_translate_children(children);
                }
            }
        });
        node_list.retain(|node| match node {
            Node::UnknownMetaTag(tag, ..) if tag == "I18N translate-children" => false,
            _ => true,
        })
    }

    fn split_translated_str(translated_str: String) -> Vec<String> {
        let mut translated_str_vec: Vec<String> = Vec::new();
        let regex = Regex::new(r"\{\{.*?\}\}").unwrap();
        let mut last = 0;
        for mat in regex.find_iter(&translated_str) {
            if mat.start() != last {
                translated_str_vec.push(translated_str[last..mat.start()].to_string());
            }
            translated_str_vec.push(translated_str[mat.start()..mat.end()].to_string());
            last = mat.end();
        }
        if last < translated_str.len() {
            translated_str_vec.push(translated_str[last..].to_string());
        }
        translated_str_vec
    }

    fn translate_value(value: &mut Value, trans_content_map: &HashMap<String, String>) {
        match value {
            Value::Static { ref mut value, .. } => {
                if let Some(translation) = trans_content_map.get(&value.to_string()) {
                    *value = translation.into();
                }
            }
            Value::Dynamic {
                expression,
                double_brace_location,
                binding_map_keys,
            } => {
                fn split_expression(
                    expr: &Expression,
                    expr_vec: &mut Vec<String>,
                    placeholder_map: &mut HashMap<char, Box<Expression>>,
                    placehoder_char: &mut char,
                    start_location: &Range<Position>,
                    end_location: &Range<Position>,
                ) {
                    match expr {
                        Expression::LitStr { value, .. } => expr_vec.push(value.to_string()),
                        Expression::ToStringWithoutUndefined { value, .. } => {
                            expr_vec.push(format!("{{{{{}}}}}", placehoder_char));
                            placeholder_map.insert(placehoder_char.clone(), value.clone());
                            *placehoder_char = ((*placehoder_char as u8) + 1) as char;
                        }
                        Expression::Plus {
                            left,
                            right,
                            location,
                        } => {
                            let split = if let Expression::ToStringWithoutUndefined { .. }
                            | Expression::LitStr { .. } = &**left
                            {
                                true
                            } else if let Expression::ToStringWithoutUndefined { .. }
                            | Expression::LitStr { .. } = &**right
                            {
                                true
                            } else {
                                false
                            };
                            if split {
                                split_expression(
                                    &left,
                                    expr_vec,
                                    placeholder_map,
                                    placehoder_char,
                                    start_location,
                                    location,
                                );
                                split_expression(
                                    &right,
                                    expr_vec,
                                    placeholder_map,
                                    placehoder_char,
                                    location,
                                    end_location,
                                );
                            }
                        }
                        _ => {}
                    }
                }
                let mut expr_vec: Vec<String> = Vec::new();
                let mut placeholder_map: HashMap<char, Box<Expression>> = HashMap::new();
                let mut start_placeholder = 'A';
                split_expression(
                    &expression,
                    &mut expr_vec,
                    &mut placeholder_map,
                    &mut start_placeholder,
                    &double_brace_location.0,
                    &double_brace_location.1,
                );
                let mut expr_str = expr_vec.join("");
                if let Some(translation) = trans_content_map.get(&expr_str) {
                    expr_str = translation.clone();
                    let translated_expr_vec: Vec<String> = split_translated_str(expr_str);
                    fn get_expr(
                        regex: &Regex,
                        item: &String,
                        position: &Range<Position>,
                        placeholder_map: &HashMap<char, Box<Expression>>,
                    ) -> Box<Expression> {
                        let trimed_item = item.trim_matches(|c| c == '{' || c == '}');
                        let potential_placeholder = trimed_item.chars().next().unwrap();
                        if regex.is_match(item)
                            && trimed_item.len() == 1
                            && placeholder_map.contains_key(&potential_placeholder)
                        {
                            return Box::new(Expression::ToStringWithoutUndefined {
                                value: placeholder_map.get(&potential_placeholder).unwrap().clone(),
                                location: position.clone(),
                            });
                        } else {
                            Box::new(Expression::LitStr {
                                value: item.into(),
                                location: position.clone(),
                            })
                        }
                    }
                    let regex = Regex::new(r"\{\{.*?\}\}").unwrap();
                    let translated_expression = translated_expr_vec
                        .into_iter()
                        .map(|item| {
                            get_expr(&regex, &item, &double_brace_location.0, &placeholder_map)
                        })
                        .fold(None, |acc, x| match acc {
                            None => Some(x),
                            Some(acc) => Some(Box::new(Expression::Plus {
                                left: acc,
                                right: x,
                                location: double_brace_location.clone().0,
                            })),
                        })
                        .unwrap();
                    let translated_dynamic_value = Value::Dynamic {
                        expression: translated_expression,
                        double_brace_location: double_brace_location.clone(),
                        binding_map_keys: binding_map_keys.clone(),
                    };
                    *value = translated_dynamic_value;
                }
            }
        }
    }

    fn translate_attribute(
        attributes: &mut Vec<Attribute>,
        trans_content_map: &HashMap<String, String>,
        included_attributes: &Vec<String>,
    ) {
        for attribute in attributes {
            if included_attributes.contains(&attribute.name.name.to_string()) {
                translate_value(&mut attribute.value, trans_content_map)
            }
        }
    }

    fn translate_entire_children(
        node_list: &mut Vec<Node>,
        trans_content_map: &HashMap<String, String>,
    ) {
        if let Some(pos) = node_list.iter().position(
            |node| matches!(node, Node::UnknownMetaTag(tag, ..) if tag == "I18N translate-children"),
        ) {
            node_list.remove(pos);
        }
        let mut text_vec: Vec<String> = Vec::new();
        let mut placehoder_char = 'A';
        let mut placeholder_map: HashMap<char, Node> = HashMap::new();
        let mut node_location: Option<Range<Position>> = None;
        for node in node_list.into_iter() {
            match node {
                Node::Text(value) => {
                    if let Value::Static { value, location } = value {
                        text_vec.push(value.trim().to_string());
                        if let None = node_location {
                            node_location = Some(location.clone());
                        }
                    }
                }
                _ => {
                    text_vec.push(format!("{{{{{}}}}}", placehoder_char));
                    placeholder_map.insert(placehoder_char.clone(), node.clone());
                    placehoder_char = ((placehoder_char as u8) + 1) as char;
                }
            }
        }
        let mut text_str = text_vec.join("");
        if let Some(translation) = trans_content_map.get(&text_str) {
            text_str = translation.clone();
            let translated_text_vec = split_translated_str(text_str);
            let regex = Regex::new(r"\{\{.*?\}\}").unwrap();
            let mut new_node_list: Vec<Node> = Vec::new();
            for item in translated_text_vec {
                let trimed_item = item.trim_matches(|c| c == '{' || c == '}');
                let potential_placeholder = trimed_item.chars().next().unwrap();
                if regex.is_match(&item)
                    && trimed_item.len() == 1
                    && placeholder_map.contains_key(&potential_placeholder)
                {
                    new_node_list
                        .push(placeholder_map.get(&potential_placeholder).unwrap().clone());
                } else {
                    let static_text = Node::Text(Value::Static {
                        value: item.into(),
                        location: node_location.clone().unwrap(),
                    });
                    new_node_list.push(static_text);
                }
            }
            *node_list = new_node_list;
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

    fn translate(
        node_list: &mut Vec<Node>,
        trans_content_map: &HashMap<String, String>,
        included_attributes: &Vec<String>,
    ) {
        for node in node_list {
            match node {
                Node::Element(element) => match &mut element.kind {
                    ElementKind::Normal {
                        children,
                        attributes,
                        ..
                    } => {
                        if attributes.len() != 0 {
                            translate_attribute(attributes, trans_content_map, included_attributes);
                        }
                        if contains_i18n_translate_children(children) {
                            translate_entire_children(children, trans_content_map);
                        }
                        translate(children, trans_content_map, included_attributes);
                    }
                    _ => {}
                },
                Node::Text(value) => {
                    translate_value(value, trans_content_map);
                }
                _ => {}
            }
        }
    }

    if contains_i18n_tag(&template.content) {
        // Get the attributes that need to be translated
        println!("{:#?}", included_attributes);
        // Element::IF has two children: branches and else_branch
        let mut branches: Vec<(Range<Position>, Value, Vec<Node>)> = vec![];
        let branch_template = remove_i18n_tag(&template.content);
        let branch_position = get_first_child_position(&template.content).unwrap();
        for (lang, trans_content_map) in trans_content.map.iter() {
            let mut template_item = branch_template.clone();
            let eq_full = Box::new(Expression::EqFull {
                left: Box::new(Expression::DataField {
                    name: "locale".into(),
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
            translate(&mut template_item, trans_content_map, &included_attributes);
            branches.push((branch_position.clone(), branch_value, template_item));
        }

        // origin template only warpped with <block wx:else> </block> and is unnecessary to be translated by i18n
        let mut else_branch_template = branch_template.clone();
        remove_i18n_translate_children(&mut else_branch_template);
        let else_branch = Some((branch_position.clone(), else_branch_template));
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
