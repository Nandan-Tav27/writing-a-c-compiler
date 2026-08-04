use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Context;

use crate::assemble::{assembler, emit, lex, parse, tacky};

#[derive(Debug, PartialEq)]
pub enum Stage {
    Lex,
    Parse,
    Tacky,
    Codegen,
    Full,
}

pub fn preprocess(file_path: &Path) -> anyhow::Result<PathBuf> {
    if !file_path.try_exists()? {
        anyhow::bail!("file does not exist: {:?}", file_path);
    }

    if !file_path.is_file() || file_path.extension().and_then(|e| e.to_str()) != Some("c") {
        anyhow::bail!("file should have the extension '.c'");
    }

    let preprocessed_file_path = file_path.with_extension("i");

    let res = Command::new("gcc")
        .args([
            "-E",
            "-P",
            file_path.to_str().unwrap(),
            "-o",
            preprocessed_file_path.to_str().unwrap(),
        ])
        .status()
        .context("failed to run preprocessor")?;

    if !res.success() {
        anyhow::bail!("preprocessor exited with status: {:?}", res);
    }

    Ok(preprocessed_file_path)
}

pub fn assemble(file_path: PathBuf, stage: Stage) -> anyhow::Result<Option<PathBuf>> {
    let assembled_file_path = file_path.with_extension("s");

    // lex
    let tokens: Vec<lex::Token> = lex::lex(&file_path)?;
    for token in &tokens {
        println!("{:?}\n", token);
    }

    if stage == Stage::Lex {
        fs::remove_file(file_path)?;
        return Ok(None);
    }

    // parse
    let program = parse::parse(&tokens)?;
    program.print_program();

    if stage == Stage::Parse {
        fs::remove_file(file_path)?;
        return Ok(None);
    }

    // transform to TACKY
    let tacky = tacky::transform(program);
    tacky.print_program();

    if stage == Stage::Tacky {
        fs::remove_file(file_path)?;
        return Ok(None);
    }

    // codegen
    let asm_ast = assembler::assemble(tacky);

    if stage == Stage::Codegen {
        fs::remove_file(file_path)?;
        return Ok(None);
    }

    // emit
    emit::emit(&assembled_file_path, asm_ast)?;

    fs::remove_file(file_path)?;

    Ok(Some(assembled_file_path))
}

pub fn link(file_path: PathBuf) -> anyhow::Result<PathBuf> {
    let object_file_path = file_path.with_extension("");

    let res = Command::new("gcc")
        .args([
            file_path.to_str().unwrap(),
            "-o",
            object_file_path.to_str().unwrap(),
        ])
        .status()
        .context("failed to run linker")?;

    if !res.success() {
        anyhow::bail!("linker exited with status: {:?}", res);
    }

    // delete assembly file
    fs::remove_file(file_path)?;

    Ok(object_file_path)
}
