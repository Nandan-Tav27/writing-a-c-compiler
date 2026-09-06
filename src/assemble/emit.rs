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
                let suffix = if matches!(dest, assembler::Operand::Register(assembler::Reg::CL))
                    || matches!(source, assembler::Operand::Register(assembler::Reg::CL))
                {
                    "b"
                } else {
                    "l"
                };
                write!(contents, "\tmov{} ", suffix).context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
                write_operand(contents, source, 4)?;
                write!(contents, ", ").context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
                write_operand(contents, dest, 4)?;
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
                write_operand(contents, operand, 4)?;
                writeln!(contents).context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
            }
            assembler::Instruction::AllocateStack(offset) => {
                writeln!(contents, "\tsubq ${}, %rsp", offset).context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
            }
            assembler::Instruction::Binary(op, operand1, operand2) => {
                write!(contents, "\t").context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
                write_binary_op(contents, op)?;
                write!(contents, " ").context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
                write_operand(contents, operand1, 4)?;
                write!(contents, ", ").context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
                write_operand(contents, operand2, 4)?;
                writeln!(contents).context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
            }
            assembler::Instruction::Idiv(operand) => {
                write!(contents, "\tidivl ").context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
                write_operand(contents, operand, 4)?;
                writeln!(contents).context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
            }
            assembler::Instruction::Cdq => {
                writeln!(contents, "\tcdq").context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
            }
            assembler::Instruction::Cmp(operand1, operand2) => {
                write!(contents, "\tcmpl ").context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
                write_operand(contents, operand1, 4)?;
                write!(contents, ", ").context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
                write_operand(contents, operand2, 4)?;
                writeln!(contents).context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
            }
            assembler::Instruction::Jmp(label) => {
                write!(contents, "\tjmp ").context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
                write!(contents, ".L{}", label).context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
                writeln!(contents).context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
            }
            assembler::Instruction::JmpCC(cc, label) => {
                write!(contents, "\tj").context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
                write_condition_code(contents, cc)?;
                write!(contents, " .L{}", label).context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
                writeln!(contents).context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
            }
            assembler::Instruction::SetCC(cc, operand) => {
                write!(contents, "\tset").context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
                write_condition_code(contents, cc)?;
                write!(contents, " ").context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
                write_operand(contents, operand, 1)?;
                writeln!(contents).context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
            }
            assembler::Instruction::Label(label) => {
                writeln!(contents, ".L{}:", label).context(
                    "Assembly generation error: error while writing instruction to file",
                )?;
            }
        }
    }

    Ok(())
}

fn write_operand(
    contents: &mut String,
    operand: assembler::Operand,
    bytes: u8,
) -> anyhow::Result<()> {
    match operand {
        assembler::Operand::Imm(val) => write!(contents, "${}", val)
            .context("Assembly generation error: error while writing operand to file"),
        assembler::Operand::Register(reg) => {
            if bytes == 4 {
                write_4byte_register(contents, reg)
            } else if bytes == 1 {
                write_1byte_register(contents, reg)
            } else {
                anyhow::bail!("Assembly generation error: invalid byte size provided for operand")
            }
        }
        assembler::Operand::Stack(offset) => write!(contents, "{}(%rbp)", offset)
            .context("Assembly generation error: error while writing operand to file"),
        _ => anyhow::bail!("Assembly generation error: encountered invalid operand type"),
    }
}

fn write_4byte_register(contents: &mut String, register: assembler::Reg) -> anyhow::Result<()> {
    match register {
        assembler::Reg::AX => write!(contents, "%eax")
            .context("Assembly generation error: error while writing register to file"),
        assembler::Reg::DX => write!(contents, "%edx")
            .context("Assembly generation error: error while writing register to file"),
        assembler::Reg::R10 => write!(contents, "%r10d")
            .context("Assembly generation error: error while writing register to file"),
        assembler::Reg::R11 => write!(contents, "%r11d")
            .context("Assembly generation error: error while writing register to file"),
        assembler::Reg::CL => write!(contents, "%cl")
            .context("Assembly generation error: error while writing register to file"),
    }
}

fn write_1byte_register(contents: &mut String, register: assembler::Reg) -> anyhow::Result<()> {
    match register {
        assembler::Reg::AX => write!(contents, "%al")
            .context("Assembly generation error: error while writing register to file"),
        assembler::Reg::DX => write!(contents, "%dl")
            .context("Assembly generation error: error while writing register to file"),
        assembler::Reg::R10 => write!(contents, "%r10b")
            .context("Assembly generation error: error while writing register to file"),
        assembler::Reg::R11 => write!(contents, "%r11b")
            .context("Assembly generation error: error while writing register to file"),
        _ => anyhow::bail!("Assembly generation error: invalid 1-byte register provided"),
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

fn write_binary_op(contents: &mut String, op: assembler::BinaryOp) -> anyhow::Result<()> {
    match op {
        assembler::BinaryOp::Add => write!(contents, "addl")
            .context("Assembly generation error: error while writing unary operator to file"),
        assembler::BinaryOp::Sub => write!(contents, "subl")
            .context("Assembly generation error: error while writing unary operator to file"),
        assembler::BinaryOp::Mult => write!(contents, "imull")
            .context("Assembly generation error: error while writing unary operator to file"),
        assembler::BinaryOp::Sal => write!(contents, "sall")
            .context("Assembly generation error: error while writing unary operator to file"),
        assembler::BinaryOp::Sar => write!(contents, "sarl")
            .context("Assembly generation error: error while writing unary operator to file"),
        assembler::BinaryOp::And => write!(contents, "andl")
            .context("Assembly generation error: error while writing unary operator to file"),
        assembler::BinaryOp::Xor => write!(contents, "xorl")
            .context("Assembly generation error: error while writing unary operator to file"),
        assembler::BinaryOp::Or => write!(contents, "orl")
            .context("Assembly generation error: error while writing unary operator to file"),
    }
}

fn write_condition_code(contents: &mut String, cc: assembler::CondCode) -> anyhow::Result<()> {
    match cc {
        assembler::CondCode::E => write!(contents, "e")
            .context("Assembly generation error: error while writing condition code to file"),
        assembler::CondCode::NE => write!(contents, "ne")
            .context("Assembly generation error: error while writing condition code to file"),
        assembler::CondCode::L => write!(contents, "l")
            .context("Assembly generation error: error while writing condition code to file"),
        assembler::CondCode::LE => write!(contents, "le")
            .context("Assembly generation error: error while writing condition code to file"),
        assembler::CondCode::G => write!(contents, "g")
            .context("Assembly generation error: error while writing condition code to file"),
        assembler::CondCode::GE => write!(contents, "ge")
            .context("Assembly generation error: error while writing condition code to file"),
    }
}
