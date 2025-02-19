use crate::{has_i18n_translate_children, is_i18n_tag, parse_additional_template};

use super::contains_i18n_translate_children;
use glass_easel_template_compiler::{
    parse::{
        expr::Expression,
        parse,
        tag::{Comment, Element, ElementKind, Node, NormalAttribute, Value},
        Position, TemplateStructure,
    },
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

#[derive(Debug)]
pub struct OrderedTransContent {
    pub map: Vec<(String, HashMap<String, String>)>,
}

pub fn compile(
    path: &str,
    source: &str,
    trans_source: &str,
    included_attributes: &[String],
) -> Result<CompiledTemplate, String> {
    // parse the template
    let (mut template, parse_state) = parse(path, source);
    for warning in parse_state.warnings() {
        if warning.prevent_success() {
            return Err(format!("Failed to compile template: {}", warning));
        }
    }
    let mut trans_content: TransContent = toml::from_str(&trans_source).unwrap();
    // merge the global.locale
    fn merge_map(locale_map: &mut HashMap<String, HashMap<String, String>>) {
        let mut to_modify = Vec::new();
        let mut to_remove = Vec::new();
        for (key, value) in locale_map.iter() {
            if key.starts_with("global.") {
                let new_key = key.replace("global.", "");
                to_remove.push(key.clone());
                for (msg_id, msg_str) in value.iter() {
                    to_modify.push((new_key.clone(), msg_id.clone(), msg_str.clone()));
                }
            }
        }
        for (new_key, msg_id, msg_str) in to_modify {
            locale_map
                .entry(new_key)
                .and_modify(|modify_map| {
                    modify_map.entry(msg_id.clone()).or_insert(msg_str.clone());
                })
                .or_insert_with(|| {
                    let mut insert_map = HashMap::new();
                    insert_map.insert(msg_id, msg_str);
                    insert_map
                });
        }
        for key in to_remove {
            locale_map.remove(&key);
        }
    }
    merge_map(&mut trans_content.map);

    // transform the template to support i18n
    fn contains_i18n_tag(node_list: &Vec<Node>) -> bool {
        for node in node_list {
            match node {
                Node::UnknownMetaTag(tag, ..) => {
                    if is_i18n_tag(tag) {
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
        if let Some(pos) = new_list
            .iter()
            .position(|node| matches!(node, Node::UnknownMetaTag(tag, ..) if is_i18n_tag(tag)))
        {
            new_list.remove(pos);
        }
        new_list
    }

    fn remove_i18n_translate_children(node_list: &mut Vec<Node>) {
        fn rec(node: &mut Node) {
            let should_remove = match node {
                Node::UnknownMetaTag(tag, ..) if has_i18n_translate_children(tag) => true,
                _ => false,
            };
            if should_remove {
                *node = Node::Comment(Comment::new("", node.location()));
            }
            if let Node::Element(element) = node {
                for child in element.iter_children_mut() {
                    rec(child);
                }
            }
        }
        for node in node_list {
            rec(node);
        }
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
                ref mut expression,
                double_brace_location,
                ..
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
                    *expression = translated_expression;
                }
            }
            _ => {}
        }
    }

    fn translate_option_value(
        value: &mut Option<Value>,
        trans_content_map: &HashMap<String, String>,
    ) {
        if let Some(value) = value {
            translate_value(value, trans_content_map);
        }
    }

    fn translate_attribute(
        attributes: &mut Vec<NormalAttribute>,
        trans_content_map: &HashMap<String, String>,
        included_attributes: &[String],
    ) {
        for attribute in attributes {
            if included_attributes.contains(&attribute.name.name.to_string()) {
                translate_option_value(&mut attribute.value, trans_content_map)
            }
        }
    }

    fn translate_entire_children(
        node_list: &mut Vec<Node>,
        trans_content_map: &HashMap<String, String>,
    ) {
        if let Some(pos) = node_list.iter().position(
            |node| matches!(node, Node::UnknownMetaTag(tag, ..) if has_i18n_translate_children(tag)),
        ) {
            node_list.remove(pos);
        }
        let mut text_vec: Vec<String> = Vec::new();
        let mut placehoder_char = 'A';
        let mut placeholder_map: HashMap<char, Node> = HashMap::new();
        let mut first_text_node: Option<Node> = None;
        for node in node_list.into_iter() {
            match node {
                Node::Text(value) => {
                    if let Value::Static { value, .. } = value {
                        text_vec.push(value.trim().to_string());
                        if let None = first_text_node {
                            first_text_node = Some((*node).clone());
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
        let Some(first_text_node) = first_text_node else {
            return;
        };
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
                    let mut static_text = first_text_node.clone();
                    let Node::Text(Value::Static { ref mut value, .. }) = static_text else {
                        unreachable!()
                    };
                    *value = item.into();
                    new_node_list.push(static_text);
                }
            }
            *node_list = new_node_list;
        }
    }

    // the Position of else_branch or branches just placed by the position of the template's first child
    fn get_first_child_position(template: &Vec<Node>) -> Option<Range<Position>> {
        let position: Option<Range<Position>>;
        match template.get(0)? {
            Node::Element(element) => {
                position = Some(element.tag_location.close.clone());
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
                _ => {
                    return None;
                }
            },
            Node::Comment(x) => {
                position = Some(x.location.clone());
            }
            Node::UnknownMetaTag(x) => {
                position = Some(x.location.clone());
            }
            _ => {
                return None;
            }
        }
        position
    }

    fn translate(
        node_list: &mut Vec<Node>,
        trans_content_map: &HashMap<String, String>,
        included_attributes: &[String],
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
                    ElementKind::If {
                        branches,
                        else_branch,
                        ..
                    } => {
                        for branch in branches {
                            translate(&mut branch.2, trans_content_map, included_attributes)
                        }
                        match else_branch {
                            Some((_, ref mut nodes)) => {
                                translate(nodes, trans_content_map, included_attributes)
                            }
                            _ => {}
                        }
                    }
                    ElementKind::For { children, .. } => {
                        translate(children, trans_content_map, included_attributes)
                    }
                    ElementKind::Pure { children, .. } => {
                        translate(children, trans_content_map, included_attributes)
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
    fn translate_template(
        template: Vec<Node>,
        trans_content: &OrderedTransContent,
        included_attributes: &[String],
    ) -> Element {
        // generate branch content
        let mut branches: Vec<(Range<Position>, Value, Vec<Node>)> = vec![];
        let branch_position = get_first_child_position(&template).unwrap_or_default();
        for (lang, trans_content_map) in trans_content.map.iter() {
            let mut template_item = template.clone();
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
            let branch_value =
                Value::new_expression(eq_full, (branch_position.clone(), branch_position.clone()));
            translate(&mut template_item, trans_content_map, included_attributes);
            branches.push((branch_position.clone(), branch_value, template_item));
        }
        let mut else_branch_template = template.clone();
        remove_i18n_translate_children(&mut else_branch_template);
        let else_branch = Some((branch_position.clone(), else_branch_template));

        // generate a new node
        let mut if_block_template =
            parse_additional_template(r#"<block wx:if="" /><block wx:else="" />"#);
        let Node::Element(mut if_block) = if_block_template.content.pop().unwrap() else {
            panic!()
        };
        if_block.tag_location.start = (branch_position.clone(), branch_position.clone());
        if_block.tag_location.close = branch_position.clone();
        if_block.tag_location.end = Some((branch_position.clone(), branch_position.clone()));
        let ElementKind::If {
            branches: new_branches,
            else_branch: new_else_branch,
            ..
        } = &mut if_block.kind
        else {
            panic!()
        };
        *new_branches = branches;
        *new_else_branch = else_branch;

        if_block
    }

    if contains_i18n_tag(&template.content) {
        let mut trans_content = OrderedTransContent {
            map: trans_content.map.into_iter().collect(),
        };
        trans_content.map.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        // template.content
        let branch_template = remove_i18n_tag(&template.content);
        let template_content_i18n =
            translate_template(branch_template, &trans_content, &included_attributes);
        template.content = vec![Node::Element(template_content_i18n)];

        // sub_templates
        for sub_template in &mut template.globals.sub_templates {
            // search_terms(&sub_template.1, &mut output, &included_attributes);
            let sub_template_branch = sub_template.content.clone();
            let sub_template_i18n =
                translate_template(sub_template_branch, &trans_content, &included_attributes);
            sub_template.content = vec![Node::Element(sub_template_i18n)]
        }
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
