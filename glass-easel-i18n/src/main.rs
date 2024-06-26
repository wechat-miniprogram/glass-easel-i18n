use std::{fs, path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand};
use glass_easel_i18n::*;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile the template
    Compile {
        /// Path of the tamplate file
        path: PathBuf,
    },
    /// Search the untranslated terms
    Search {
        /// Path of the tamplate file
        path: PathBuf,
        /// Place holder of the untranslated terms
        place_holder: String,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Commands::Compile { path } => {
            let Some(file_name) = path.file_name() else {
                eprintln!("Not a file");
                return ExitCode::FAILURE;
            };
            let Some(file_name) = file_name.to_str() else {
                eprintln!("Not a UTF-8 file name");
                return ExitCode::FAILURE;
            };
            let source = match std::fs::read_to_string(&path) {
                Ok(source) => source,
                Err(err) => {
                    eprintln!("Failed to read source file: {}", err);
                    return ExitCode::FAILURE;
                }
            };
            let trans_source_path = path.with_extension("toml");
            let trans_source = match std::fs::read_to_string(&trans_source_path) {
                Ok(source) => source,
                Err(err) => {
                    eprintln!("Failed to read translate source file: {}", err);
                    return ExitCode::FAILURE;
                }
            };
            // Usage: test binary
            let included_attributes: Vec<String> = vec!["title".to_string()];
            // Call compile as a binary file, convenient for debugging on the rust side
            match compile(file_name, &source, &trans_source, included_attributes) {
                Ok(r) => {
                    println!("{}", r.output);
                    match fs::write("output.wxml", r.output) {
                        Ok(()) => println!("output success"),
                        Err(err) => println!("output fail:{}", err),
                    }
                }
                Err(err) => {
                    eprintln!("{}", err);
                    return ExitCode::FAILURE;
                }
            }
        }
        Commands::Search { path, place_holder } => {
            let Some(file_name) = path.file_name() else {
                eprintln!("Not a file");
                return ExitCode::FAILURE;
            };
            let Some(file_name) = file_name.to_str() else {
                eprintln!("Not a UTF-8 file name");
                return ExitCode::FAILURE;
            };
            let source = match std::fs::read_to_string(&path) {
                Ok(source) => source,
                Err(err) => {
                    eprintln!("Failed to read source file: {}", err);
                    return ExitCode::FAILURE;
                }
            };
            // Usage: test binary
            let included_attributes: Vec<String> = vec!["title".to_string()];
            match search(file_name, &source, included_attributes) {
                Ok(untranslated_terms) => {
                    println!("{:#?}", untranslated_terms.output);
                    let mut po_terms = String::new();
                    for term in untranslated_terms.output {
                        let po_term =
                            format!("msgid \"{}\"\nmsgstr \"{}\"\n\n", term, place_holder);
                        po_terms.push_str(&po_term);
                    }
                    match fs::write("output.po", po_terms) {
                        Ok(()) => println!("output success"),
                        Err(err) => println!("output fail:{}", err),
                    }
                }
                Err(err) => {
                    eprintln!("{}", err);
                    return ExitCode::FAILURE;
                }
            }
        }
    };
    ExitCode::SUCCESS
}
