mod error;
mod lex;
mod pre_grammar;

use anyhow::{anyhow, Result};
use clap::Parser;
use lex::Token;
use serde::Serialize;
use std::path::Path;

#[derive(Parser)]
struct Args {
    /// Input file (PL/0 code)
    input_file: String,
    /// Output file (JSON format tokens)
    /// if not specified, check for errors only
    #[clap(short, long)]
    output_file: Option<String>,
}
fn write_to_output(output_file: &str, tokens: Vec<Token>) -> Result<()> {
    #[derive(Serialize)]
    struct TokenOutput {
        tokens: Vec<Token>,
    }
    let tokens = TokenOutput { tokens };
    let output_file = Path::new(output_file);
    let dir = output_file
        .parent()
        .ok_or_else(|| anyhow!("Invalid output file"))?;
    std::fs::create_dir_all(dir)?;
    let output = std::fs::File::create(output_file)?;
    let output = std::io::BufWriter::new(output);

    serde_json::to_writer_pretty(output, &tokens)?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    let input_file = Path::new(&args.input_file);
    if !input_file.is_file() {
        println!("File does not exist: {}", input_file.display());
        return Ok(());
    }
    let content = std::fs::read_to_string(input_file)?;
    let mut errors = error::ErrorRecorder::new();
    let tokens = pre_grammar::parse(&content, &mut errors);
    errors.print_with(&input_file.display().to_string(), &content);
    if errors.no_error() {
        if let Some(output_file) = args.output_file {
            write_to_output(&output_file, tokens)?;
        }
        Ok(())
    } else {
        Err(anyhow!("Error detected"))
    }
}
