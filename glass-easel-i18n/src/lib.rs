use glass_easel_template_compiler::{parse::parse, stringify::{Stringifier, Stringify}};

mod js_bindings;

pub struct CompiledTemplate {
    pub output: String,
    pub source_map: Vec<u8>,
}

pub fn compile(path: &str, source: &str) -> Result<CompiledTemplate, String> {
    // parse the template
    let (template, parse_state) = parse(path, source);
    for warning in parse_state.warnings() {
        if warning.prevent_success() {
            return Err(format!("Failed to compile template: {}", warning));
        }
    }

    // TODO

    // stringify the template
    let mut stringifier = Stringifier::new(String::new(), path, source);
    template.stringify_write(&mut stringifier).map_err(|_| "Failed to write output")?;
    let (output, sm) = stringifier.finish();
    let mut source_map = vec![];
    sm.to_writer(&mut source_map).map_err(|_| "Failed to write output")?;
    Ok(CompiledTemplate { output, source_map })
}
