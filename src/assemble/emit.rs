use anyhow::Context;

use crate::assemble::assembler;

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
    writeln!(contents, "\tpushq %rbp")
        .context("Assembly generation error: error while writing function prologue to file")?;
    writeln!(contents, "\tmovq %rsp, %rbp")
        .context("Assembly generation error: error while writing function prologue to file")?;
    write_instructions(contents, function_def.instructions)
}

fn write_instructions(
    contents: &mut String,
    instructions: Vec<assembler::Instruction>,
) -> anyhow::Result<()> {
    for instruction in instructions {
        match instruction {
            assembler::Instruction::Mov { source, dest } => {
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
            assembler::Instruction::Ret => {
                writeln!(contents, "\tmovq %rbp, %rsp").context(
                    "Assembly generation error: error while writing function epilogue to file",
                )?;
                writeln!(contents, "\tpopq %rbp").context(
                    "Assembly generation error: error while writing function epilogue to file",
                )?;
                writeln!(contents, "\tret").context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
            }
            assembler::Instruction::Unary(op, operand) => {
                write!(contents, "\t").context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
                write_unary_op(contents, op)?;
                write!(contents, " ").context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
                write_operand(contents, operand)?;
                writeln!(contents).context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
            }
            assembler::Instruction::AllocateStack(offset) => {
                writeln!(contents, "\tsubq ${}, %rsp", offset).context(
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
        assembler::Operand::Register(reg) => write_register(contents, reg),
        assembler::Operand::Stack(offset) => write!(contents, "{}(%rbp)", offset)
            .context("Assembly generation error: error while writing operand to file"),
        _ => anyhow::bail!("Assembly generation error: encountered invalid operand type"),
    }
}

fn write_register(contents: &mut String, register: assembler::Reg) -> anyhow::Result<()> {
    match register {
        assembler::Reg::AX => write!(contents, "%eax")
            .context("Assembly generation error: error while writing register to file"),
        assembler::Reg::R10 => write!(contents, "%r10d")
            .context("Assembly generation error: error while writing register to file"),
    }
}

fn write_unary_op(contents: &mut String, op: assembler::UnaryOp) -> anyhow::Result<()> {
    match op {
        assembler::UnaryOp::Not => write!(contents, "notl")
            .context("Assembly generation error: error while writing unary operator to file"),
        assembler::UnaryOp::Neg => write!(contents, "negl")
            .context("Assembly generation error: error while writing unary operator to file"),
    }
}
