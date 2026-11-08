//! Minimal canonical-document CLI used by PHP and CI.

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::exit,
    clippy::absolute_paths,
    clippy::excessive_nesting,
    reason = "CLI output and process boundary"
)]

use std::fs;
use std::io::{self, Read};
use std::process::ExitCode;

use rich_editor_auy::error::EditError;
use rich_editor_auy::model::Doc;
use rich_editor_auy::schema::Schema;
use rich_editor_auy::wire::{Envelope, ParseOptions, from_json};

#[allow(clippy::too_many_lines, reason = "single command dispatcher")]
fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_else(|| String::from("validate"));
    if command == "diff" {
        return diff(&args.collect::<Vec<String>>());
    }
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        eprintln!("error: could not read stdin");
        return ExitCode::FAILURE;
    }
    let Ok(schema) = Schema::basic() else {
        eprintln!("error: could not build schema");
        return ExitCode::FAILURE;
    };
    let parsed = parse_document(&input, &schema);
    match command.as_str() {
        "validate" => match parsed {
            Ok(doc) => {
                let report = doc.validate(&schema);
                if report.is_ok() {
                    println!("ok");
                    ExitCode::SUCCESS
                } else {
                    for issue in &report.issues {
                        eprintln!(
                            "{} at {} ({})",
                            issue.invariant,
                            issue.position.map_or(0, |position| position.0),
                            issue.path
                        );
                    }
                    ExitCode::FAILURE
                }
            }
            Err(error) => {
                eprintln!("error: {error}");
                ExitCode::FAILURE
            }
        },
        "format" => match parsed {
            Ok(doc) => {
                println!("{}", doc.to_json_string(&schema));
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("error: {error}");
                ExitCode::FAILURE
            }
        },
        "a11y-tree" => match parsed {
            Ok(doc) => {
                println!("{}", doc.a11y_tree(&schema).to_json_string(&schema));
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("error: {error}");
                ExitCode::FAILURE
            }
        },
        "a11y" => match parsed {
            Ok(doc) => {
                let structural = doc.validate(&schema);
                if !structural.is_ok() {
                    for issue in &structural.issues {
                        eprintln!(
                            "{} at {} ({})",
                            issue.invariant,
                            issue.position.map_or(0, |position| position.0),
                            issue.path
                        );
                    }
                    return ExitCode::FAILURE;
                }
                let report = doc.validate_a11y(&schema);
                if report.is_ok() {
                    println!("ok");
                    ExitCode::SUCCESS
                } else {
                    for issue in &report.errors {
                        eprintln!(
                            "{} at {} ({})",
                            issue.code.as_str(),
                            issue.range.start.0,
                            issue.path
                        );
                    }
                    ExitCode::FAILURE
                }
            }
            Err(error) => {
                eprintln!("error: {error}");
                ExitCode::FAILURE
            }
        },
        _ => {
            eprintln!(
                "usage: rich-editor-auy-cli [validate|format|a11y|a11y-tree|diff LEFT RIGHT]"
            );
            ExitCode::FAILURE
        }
    }
}

fn diff(paths: &[String]) -> ExitCode {
    if paths.len() != 2 {
        eprintln!("usage: rich-editor-auy-cli diff LEFT RIGHT");
        return ExitCode::FAILURE;
    }
    let Ok(schema) = Schema::basic() else {
        eprintln!("error: could not build schema");
        return ExitCode::FAILURE;
    };
    let mut documents = Vec::new();
    for path in paths {
        let Ok(bytes) = fs::read_to_string(path) else {
            eprintln!("error: could not read {path}");
            return ExitCode::FAILURE;
        };
        match from_json(&bytes, &schema, &ParseOptions::default()) {
            Ok(doc) => documents.push(doc),
            Err(error) => {
                eprintln!(
                    "error: {} at {} ({}) in {path}",
                    error.code, error.offset, error.path
                );
                return ExitCode::FAILURE;
            }
        }
    }
    let Some(left) = documents.first() else {
        return ExitCode::FAILURE;
    };
    let Some(right) = documents.get(1) else {
        return ExitCode::FAILURE;
    };
    if left == right {
        println!("equal");
        return ExitCode::SUCCESS;
    }
    println!(
        "left_blocks={} right_blocks={}",
        left.blocks().len(),
        right.blocks().len()
    );
    let count = left.blocks().len().max(right.blocks().len());
    for index in 0..count {
        let left_block = left.blocks().get(index);
        let right_block = right.blocks().get(index);
        if left_block != right_block {
            println!("block {index}: differs");
        }
    }
    ExitCode::FAILURE
}

fn parse_document(input: &str, schema: &Schema) -> Result<Doc, EditError> {
    if input.trim_start().starts_with("{\"format\"") {
        Envelope::from_json(input, schema, &ParseOptions::default()).map(|envelope| envelope.doc)
    } else {
        from_json(input, schema, &ParseOptions::default()).map_err(EditError::Parse)
    }
}
