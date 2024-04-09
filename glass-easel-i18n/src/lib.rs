use glass_easel_template_compiler::{
    parse::parse,
    parse::tag::{Element,ElementKind, Node,Value},
    parse::expr::Expression,
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

pub fn compile(path: &str, source: &str) -> Result<CompiledTemplate, String> {
    // parse the template
    let (mut template, parse_state) = parse(path, source);
    for warning in parse_state.warnings() {
        if warning.prevent_success() {
            return Err(format!("Failed to compile template: {}", warning));
        }
    }

    let mut trans_content: Option<TransContent> = None;

    // transform the template to support i18n
    println!("template:{:#?}", template.content);
    for node in &template.content {
        match node {
            Node::UnknownMetaTag(tag, range) => {
                if tag.starts_with("I18N") {
                    if let Some(start) = tag.find("locale-files=") {
                        let start = start + "locale-files=".len();
                        let end = tag[start..].find(" ").unwrap_or_else(|| tag.len());
                        let locale_file_name =
                            format!("{}.toml", &tag[start..end].trim_matches('\"'));
                        let trans_content_str = match std::fs::read_to_string(&locale_file_name) {
                            Ok(source) => source,
                            Err(err) => {
                                return Err(format!("Failed to read locale file: {}", err));
                            }
                        };
                        let trans_content_inside: TransContent =
                            toml::from_str(&trans_content_str).unwrap();
                        println!("{:#?}", trans_content_inside.map);
                        trans_content = Some(trans_content_inside);
                    }
                }
                println!("UnknownMetaTag: {:?}, range: {:?}", tag, range);
                break;
            }
            _ => {}
        }
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

    fn translate(node_list:&mut Vec<Node>, translations: &HashMap<String, HashMap<String, String>>) {
        for node in node_list.iter_mut() {
            match node {
                Node::Element(element) =>{
                    match &mut element.kind {
                        ElementKind::Normal {children,.. }=>{
                            if contains_text_node(children) {
                                // let eq_full = Box::new(Expression::EqFull { 
                                //     left: Box::new(Expression::DataField {
                                //         name:"lang".into(),
                                //         // loaction:
                                //     }), 
                                //     right: Box::new(Expression::LitStr { 
                                //         value: "en-us".into(), 
                                //         // location: 
                                //     }), 
                                //     // location:  
                                // });
                                // let value = Value::Dynamic { expression: eq_full , double_brace_location: _ , binding_map_keys: None };
                                // let element_i18n = Element {
                                //     kind: ElementKind::If { 
                                //         branches: vec![()], 
                                //         else_branch: vec![()],
                                //     },
                                //     start_tag_location: element.start_tag_location,
                                //     close_location:element.close_location,
                                //     end_tag_location:element.end_tag_location,
                                // }
                                // *node = Node::Element(element_i18n);
                            } else {
                                translate(children,translations);
                            }
                        }
                        _=>{}
                    }
                }
                _ =>{}
            }
        }
    }

    match trans_content {
        Some(trans_content) => {
            println!("{:#?}", trans_content.map);
            translate(&mut template.content, &trans_content.map);
        }
        None => {
            println!("trans_content is None");
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
