use super::contains_i18n_translate_children;
use glass_easel_template_compiler::{
    parse::expr::Expression,
    parse::parse,
    parse::tag::{Attribute, ElementKind, Node, Value},
};

pub struct UntranslatedTerms {
    pub output: Vec<String>,
}

pub fn search(
    path: &str,
    source: &str,
    included_attributes: Vec<String>,
) -> Result<UntranslatedTerms, String> {
    // parse the template
    let (template, parse_state) = parse(path, source);
    for warning in parse_state.warnings() {
        if warning.prevent_success() {
            return Err(format!("Failed to compile template: {}", warning));
        }
    }
    let mut output = vec![];
    fn collect_terms(value: &Value, terms_vec: &mut Vec<String>) {
        match value {
            Value::Static { value, .. } => {
                let untranslated_term = value.trim().to_string();
                if !terms_vec.contains(&untranslated_term) {
                    terms_vec.push(untranslated_term);
                }
            }
            Value::Dynamic { expression, .. } => {
                fn split_expression(
                    expr: &Expression,
                    expr_vec: &mut Vec<String>,
                    placehoder_char: &mut char,
                ) {
                    match expr {
                        Expression::LitStr { value, .. } => expr_vec.push(value.to_string()),
                        Expression::ToStringWithoutUndefined { .. } => {
                            expr_vec.push(format!("{{{{{}}}}}", placehoder_char));
                            *placehoder_char = ((*placehoder_char as u8) + 1) as char;
                        }
                        Expression::Plus { left, right, .. } => {
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
                                split_expression(&left, expr_vec, placehoder_char);
                                split_expression(&right, expr_vec, placehoder_char);
                            }
                        }
                        _ => {}
                    }
                }
                let mut expr_vec: Vec<String> = Vec::new();
                let mut start_placeholder = 'A';
                split_expression(&expression, &mut expr_vec, &mut start_placeholder);
                terms_vec.push(expr_vec.join(""));
            }
        }
    }
    fn collect_attribute_terms(
        attributes: &Vec<Attribute>,
        terms_vec: &mut Vec<String>,
        included_attributes: &Vec<String>,
    ) {
        for attribute in attributes {
            if included_attributes.contains(&attribute.name.name.to_string()) {
                collect_terms(&attribute.value, terms_vec)
            }
        }
    }
    fn collect_entire_children(
        node_list: &Vec<Node>,
        terms_vec: &mut Vec<String>,
        included_attributes: &Vec<String>,
    ) {
        let mut text_vec: Vec<String> = Vec::new();
        let mut placehoder_char = 'A';
        for node in node_list.into_iter() {
            match node {
                // handle <!I18N translate-children>
                Node::UnknownMetaTag(..) => {
                    continue;
                }
                Node::Text(value) => {
                    if let Value::Static { value, .. } = value {
                        text_vec.push(value.trim().to_string());
                    }
                }
                Node::Element(element) => match &element.kind {
                    ElementKind::Normal { children, .. } => {
                        text_vec.push(format!("{{{{{}}}}}", placehoder_char));
                        placehoder_char = ((placehoder_char as u8) + 1) as char;
                        search_terms(children, terms_vec, included_attributes)
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        terms_vec.push(text_vec.join(""));
    }
    fn search_terms(
        node_list: &Vec<Node>,
        terms_vec: &mut Vec<String>,
        included_attributes: &Vec<String>,
    ) {
        for node in node_list {
            match node {
                Node::Element(element) => match &element.kind {
                    ElementKind::Normal {
                        children,
                        attributes,
                        ..
                    } => {
                        if attributes.len() != 0 {
                            collect_attribute_terms(attributes, terms_vec, included_attributes);
                        }
                        if contains_i18n_translate_children(children) {
                            collect_entire_children(children, terms_vec, included_attributes);
                        } else {
                            search_terms(children, terms_vec, included_attributes);
                        }
                    }
                    ElementKind::If {
                        branches,
                        else_branch,
                    } => {
                        for branch in branches {
                            search_terms(&branch.2, terms_vec, included_attributes)
                        }
                        match else_branch {
                            Some((_, ref nodes)) => {
                                search_terms(nodes, terms_vec, included_attributes)
                            }
                            _ => {}
                        }
                    }
                    ElementKind::For { children, .. } => {
                        search_terms(children, terms_vec, included_attributes)
                    }
                    ElementKind::Pure { children, .. } => {
                        search_terms(children, terms_vec, included_attributes)
                    }
                    _ => {}
                },
                Node::Text(value) => {
                    collect_terms(value, terms_vec);
                }
                _ => {}
            }
        }
    }
    // template.content
    search_terms(&template.content, &mut output, &included_attributes);

    // sub_templates
    for sub_template in &template.globals.sub_templates {
        search_terms(&sub_template.1, &mut output, &included_attributes);
    }

    // splice empty string
    output.retain(|s| !s.trim().is_empty());

    Ok(UntranslatedTerms { output })
}
