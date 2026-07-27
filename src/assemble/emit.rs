use anyhow::Context;

use crate::assemble::assembler::{self, Instruction};

use std::{fmt::Write, fs, path::Path};

pub fn emit(file_path: &Path, program: assembler::Program) -> anyhow::Result<()> {
    let mut contents = String::new();
    write_program(&mut contents, program)?;
    fs::write(file_path, contents).context("Assembly generation error: error while writing to file")
}

fn write_program(contents: &mut String, program: assembler::Program) -> anyhow::Result<()> {
    write_function_def(contents, program.function_def)?;
    writeln!(contents, "\t.section .note.GNU-stack,\"\",@progbits")
        .context("Assembly generation error: error while writing program to file")
}

fn write_function_def(
    contents: &mut String,
    function_def: assembler::FunctionDef,
) -> anyhow::Result<()> {
    writeln!(contents, "\t.globl {}", function_def.name)
        .context("Assembly generation error: error while writing function defintion to file")?;
    writeln!(contents, "{}:", function_def.name)
        .context("Assembly generation error: error while writing function definition to file")?;
    write_instructions(contents, function_def.instructions)
}

fn write_instructions(
    contents: &mut String,
    instructions: Vec<assembler::Instruction>,
) -> anyhow::Result<()> {
    for instruction in instructions {
        match instruction {
            Instruction::Mov { source, dest } => {
                write!(contents, "\tmovl ").context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
                write_operand(contents, source)?;
                write!(contents, ", ").context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
                write_operand(contents, dest)?;
                writeln!(contents).context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
            }
            Instruction::Ret => {
                writeln!(contents, "\tret").context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
            }
        }
    }

    Ok(())
}

fn write_operand(contents: &mut String, operand: assembler::Operand) -> anyhow::Result<()> {
    match operand {
        assembler::Operand::Imm(val) => write!(contents, "${}", val)
            .context("Assembly generation error: error while writing operand to file"),
        assembler::Operand::Register => write!(contents, "%eax")
            .context("Assembly generation error: error while writing operand to file"),
    }
}
