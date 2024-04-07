mod lex;
mod line_pos;
mod pre_grammar;

use anyhow::{anyhow, Result};
use clap::Parser;
use std::path::Path;

use crate::{line_pos::LinePos, pre_grammar::PreGrammar};

#[derive(Parser)]
struct Args {
    input_file: String,
    #[clap(short, long)]
    output_file: String,
}
fn write_to_output(output_file: &str, tokens: pre_grammar::TokensOutput) -> Result<()> {
    let output_file = Path::new(output_file);
    let output = std::fs::File::create(output_file)?;
    let dir = output_file
        .parent()
        .ok_or_else(|| anyhow!("Invalid output file"))?;
    std::fs::create_dir_all(dir)?;
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
    let line_pos = LinePos::new(&content);
    let mut pre_grammar = PreGrammar::new(&content);
    pre_grammar.parse();
    match pre_grammar.output() {
        Ok(tokens) => write_to_output(&args.output_file, tokens),
        Err(errors) => {
            for e in errors {
                line_pos.display_error(&e);
            }
            Err(anyhow!("Syntax errors"))
        }
    }
}
