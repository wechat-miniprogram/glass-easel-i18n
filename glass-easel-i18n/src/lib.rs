use glass_easel_template_compiler::parse::{tag::{Node, UnknownMetaTag, Value}, Template};

mod compile;
mod js_bindings;
mod search;

pub use compile::*;
pub use search::*;

fn parse_additional_template(src: &str) -> Template {
    let (template, _) = glass_easel_template_compiler::parse::parse("", src);
    template
}

fn is_i18n_tag(tag: &UnknownMetaTag) -> bool {
    tag.tag_name.len() == 1 && tag.tag_name[0].name == "I18N"
}

fn get_i18n_attr_value<'a>(tag: &'a UnknownMetaTag, attr_name: &str) -> Option<Option<&'a Value>> {
    if !is_i18n_tag(tag) { return None; }
    tag.attributes.iter().find(|x| {
        x.colon_separated_name.len() == 1 && x.colon_separated_name[0].name == attr_name
    }).map(|x| x.value.as_ref())
}

fn has_i18n_translate_children(tag: &UnknownMetaTag) -> bool {
    get_i18n_attr_value(tag, "translate-children").is_some()
}

pub fn contains_i18n_translate_children(node_list: &Vec<Node>) -> bool {
    for node in node_list {
        match node {
            Node::UnknownMetaTag(tag, ..) => {
                if has_i18n_translate_children(tag) {
                    return true;
                }
                break;
            }
            _ => {}
        }
    }
    false
}
