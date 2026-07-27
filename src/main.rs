mod assemble;
mod driver;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    file_path: PathBuf,
    #[arg(long, conflicts_with_all = ["parse", "codegen", "s"])]
    lex: bool,
    #[arg(long, conflicts_with_all = ["lex", "codegen", "s"])]
    parse: bool,
    #[arg(long, conflicts_with_all = ["lex", "parse", "s"])]
    codegen: bool,
    #[arg(short = 'S', conflicts_with_all = ["lex", "parse", "codegen"])]
    s: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let stage = if cli.lex {
        driver::Stage::Lex
    } else if cli.parse {
        driver::Stage::Parse
    } else if cli.codegen {
        driver::Stage::Codegen
    } else {
        driver::Stage::Full
    };

    let preprocessed_file = driver::preprocess(&cli.file_path)?;
    let assembled_file = driver::assemble(preprocessed_file, stage)?.unwrap();
    let object_file = driver::link(assembled_file)?;

    Ok(())
}
