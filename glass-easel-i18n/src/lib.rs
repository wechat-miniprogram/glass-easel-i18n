use glass_easel_template_compiler::parse::tag::Node;
use regex::Regex;

mod compile;
mod js_bindings;
mod search;

pub use compile::*;
pub use search::*;

pub fn contains_i18n_translate_children(node_list: &Vec<Node>) -> bool {
    for node in node_list {
        match node {
            Node::UnknownMetaTag(tag, ..) => {
                if tag == "I18N translate-children" {
                    return true;
                }
                break;
            }
            _ => {}
        }
    }
    false
}

pub fn get_included_attributes(node_list: &Vec<Node>) -> Vec<String> {
    for node in node_list {
        match node {
            Node::UnknownMetaTag(tag, ..) => {
                if tag.starts_with("I18N") {
                    let regex = Regex::new(r#"I18N translate-attributes="([^"]*)""#).unwrap();
                    let caps = regex.captures(tag).unwrap();
                    return caps[1].split_whitespace().map(|s| s.to_string()).collect();
                }
                break;
            }
            _ => {}
        }
    }
    Vec::new()
}
