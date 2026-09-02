use std::collections::HashMap;

use crate::assemble::tacky;

pub struct Program {
    pub function_def: FunctionDef,
}

pub struct FunctionDef {
    pub name: String,
    pub instructions: Vec<Instruction>,
}

pub enum Instruction {
    Mov { source: Operand, dest: Operand },
    Unary(UnaryOp, Operand),
    Binary(BinaryOp, Operand, Operand),
    Idiv(Operand),
    Cdq,
    AllocateStack(u64),
    Ret,
}

pub enum UnaryOp {
    Not,
    Neg,
}

impl From<tacky::UnaryOp> for UnaryOp {
    fn from(op: tacky::UnaryOp) -> UnaryOp {
        match op {
            tacky::UnaryOp::Complement => UnaryOp::Not,
            tacky::UnaryOp::Negation => UnaryOp::Neg,
        }
    }
}

pub enum BinaryOp {
    Add,
    Sub,
    Mult,
    Sal,
    Sar,
    And,
    Xor,
    Or,
}

#[derive(Clone)]
pub enum Operand {
    Imm(u64),
    Register(Reg),
    Pseudo(String),
    Stack(i64),
}

impl From<tacky::Value> for Operand {
    fn from(value: tacky::Value) -> Operand {
        match value {
            tacky::Value::Constant(val) => Operand::Imm(val),
            tacky::Value::Var(val) => Operand::Pseudo(val),
        }
    }
}

#[derive(Clone)]
pub enum Reg {
    AX,
    DX,
    R10,
    R11,
    CL,
}

pub fn assemble(program: tacky::Program) -> Program {
    let mut program = lower_program(program);
    let offset = replace_pseudo(&mut program);
    allocate_stack_and_temp_reg(program, offset)
}

fn lower_program(program: tacky::Program) -> Program {
    let function_def = lower_function_def(program.function_def);
    Program { function_def }
}

fn lower_function_def(function_def: tacky::FunctionDef) -> FunctionDef {
    let name = function_def.name;
    let mut instrs: Vec<Instruction> = Vec::new();
    for instr in function_def.body {
        lower_instruction(instr, &mut instrs);
    }
    FunctionDef {
        name,
        instructions: instrs,
    }
}

fn lower_instruction(instr: tacky::Instruction, instrs: &mut Vec<Instruction>) {
    match instr {
        tacky::Instruction::Ret(val) => {
            instrs.push(Instruction::Mov {
                source: val.into(),
                dest: Operand::Register(Reg::AX),
            });
            instrs.push(Instruction::Ret);
        }
        tacky::Instruction::Unary {
            unary_operator,
            src,
            dest,
        } => {
            let dest: Operand = dest.into();
            instrs.push(Instruction::Mov {
                source: src.into(),
                dest: dest.clone(),
            });
            instrs.push(Instruction::Unary(unary_operator.into(), dest));
        }
        tacky::Instruction::Binary {
            binary_operator,
            src1,
            src2,
            dest,
        } => match binary_operator {
            tacky::BinaryOp::Add => {
                let dest: Operand = dest.into();
                instrs.push(Instruction::Mov {
                    source: src1.into(),
                    dest: dest.clone(),
                });
                instrs.push(Instruction::Binary(BinaryOp::Add, src2.into(), dest));
            }
            tacky::BinaryOp::Subtract => {
                let dest: Operand = dest.into();
                instrs.push(Instruction::Mov {
                    source: src1.into(),
                    dest: dest.clone(),
                });
                instrs.push(Instruction::Binary(BinaryOp::Sub, src2.into(), dest));
            }
            tacky::BinaryOp::Multiply => {
                let dest: Operand = dest.into();
                instrs.push(Instruction::Mov {
                    source: src1.into(),
                    dest: dest.clone(),
                });
                instrs.push(Instruction::Binary(BinaryOp::Mult, src2.into(), dest));
            }
            tacky::BinaryOp::Divide => {
                instrs.push(Instruction::Mov {
                    source: src1.into(),
                    dest: Operand::Register(Reg::AX),
                });
                instrs.push(Instruction::Cdq);
                instrs.push(Instruction::Idiv(src2.into()));
                instrs.push(Instruction::Mov {
                    source: Operand::Register(Reg::AX),
                    dest: dest.into(),
                });
            }
            tacky::BinaryOp::Remainder => {
                instrs.push(Instruction::Mov {
                    source: src1.into(),
                    dest: Operand::Register(Reg::AX),
                });
                instrs.push(Instruction::Cdq);
                instrs.push(Instruction::Idiv(src2.into()));
                instrs.push(Instruction::Mov {
                    source: Operand::Register(Reg::DX),
                    dest: dest.into(),
                });
            }
            tacky::BinaryOp::LeftShift => {
                let dest: Operand = dest.into();
                instrs.push(Instruction::Mov {
                    source: src1.into(),
                    dest: dest.clone(),
                });
                instrs.push(Instruction::Binary(BinaryOp::Sal, src2.into(), dest));
            }
            tacky::BinaryOp::RightShift => {
                let dest: Operand = dest.into();
                instrs.push(Instruction::Mov {
                    source: src1.into(),
                    dest: dest.clone(),
                });
                instrs.push(Instruction::Binary(BinaryOp::Sar, src2.into(), dest));
            }
            tacky::BinaryOp::BitwiseAND => {
                let dest: Operand = dest.into();
                instrs.push(Instruction::Mov {
                    source: src1.into(),
                    dest: dest.clone(),
                });
                instrs.push(Instruction::Binary(BinaryOp::And, src2.into(), dest));
            }
            tacky::BinaryOp::BitwiseXOR => {
                let dest: Operand = dest.into();
                instrs.push(Instruction::Mov {
                    source: src1.into(),
                    dest: dest.clone(),
                });
                instrs.push(Instruction::Binary(BinaryOp::Xor, src2.into(), dest));
            }
            tacky::BinaryOp::BitwiseOR => {
                let dest: Operand = dest.into();
                instrs.push(Instruction::Mov {
                    source: src1.into(),
                    dest: dest.clone(),
                });
                instrs.push(Instruction::Binary(BinaryOp::Or, src2.into(), dest));
            }
        },
    }
}

fn replace_pseudo(program: &mut Program) -> i64 {
    let mut offset: i64 = 0;
    let mut pseudo_offset_map: HashMap<String, i64> = HashMap::new();

    for instr in &mut program.function_def.instructions {
        match instr {
            Instruction::Mov { source, dest } => {
                replace_pseudo_with_stack(source, &mut pseudo_offset_map, &mut offset);
                replace_pseudo_with_stack(dest, &mut pseudo_offset_map, &mut offset);
            }
            Instruction::Unary(_, operand) => {
                replace_pseudo_with_stack(operand, &mut pseudo_offset_map, &mut offset);
            }
            Instruction::Binary(_, operand_1, operand_2) => {
                replace_pseudo_with_stack(operand_1, &mut pseudo_offset_map, &mut offset);
                replace_pseudo_with_stack(operand_2, &mut pseudo_offset_map, &mut offset);
            }
            Instruction::Idiv(operand) => {
                replace_pseudo_with_stack(operand, &mut pseudo_offset_map, &mut offset);
            }
            _ => {}
        }
    }

    offset
}

fn replace_pseudo_with_stack(
    operand: &mut Operand,
    map: &mut HashMap<String, i64>,
    offset: &mut i64,
) {
    if let Operand::Pseudo(name) = operand {
        let stack_offset = *map.entry(name.clone()).or_insert_with(|| {
            *offset -= 4;
            *offset
        });
        *operand = Operand::Stack(stack_offset);
    }
}

fn allocate_stack_and_temp_reg(program: Program, offset: i64) -> Program {
    let mut updated_instructions: Vec<Instruction> = Vec::new();
    let alloc_stack = Instruction::AllocateStack(offset.unsigned_abs());
    updated_instructions.push(alloc_stack);
    let FunctionDef { name, instructions } = program.function_def;
    for instr in instructions {
        match instr {
            Instruction::Mov {
                source: Operand::Stack(s),
                dest: Operand::Stack(d),
            } => {
                let instr1 = Instruction::Mov {
                    source: Operand::Stack(s),
                    dest: Operand::Register(Reg::R10),
                };
                let instr2 = Instruction::Mov {
                    source: Operand::Register(Reg::R10),
                    dest: Operand::Stack(d),
                };
                updated_instructions.push(instr1);
                updated_instructions.push(instr2);
            }
            Instruction::Idiv(Operand::Imm(val)) => {
                let instr1 = Instruction::Mov {
                    source: Operand::Imm(val),
                    dest: Operand::Register(Reg::R10),
                };
                let instr2 = Instruction::Idiv(Operand::Register(Reg::R10));
                updated_instructions.push(instr1);
                updated_instructions.push(instr2);
            }
            Instruction::Binary(BinaryOp::Add, Operand::Stack(o1), Operand::Stack(o2)) => {
                let instr1 = Instruction::Mov {
                    source: Operand::Stack(o1),
                    dest: Operand::Register(Reg::R10),
                };
                let instr2 = Instruction::Binary(
                    BinaryOp::Add,
                    Operand::Register(Reg::R10),
                    Operand::Stack(o2),
                );
                updated_instructions.push(instr1);
                updated_instructions.push(instr2);
            }
            Instruction::Binary(BinaryOp::Sub, Operand::Stack(o1), Operand::Stack(o2)) => {
                let instr1 = Instruction::Mov {
                    source: Operand::Stack(o1),
                    dest: Operand::Register(Reg::R10),
                };
                let instr2 = Instruction::Binary(
                    BinaryOp::Sub,
                    Operand::Register(Reg::R10),
                    Operand::Stack(o2),
                );
                updated_instructions.push(instr1);
                updated_instructions.push(instr2);
            }
            Instruction::Binary(BinaryOp::Mult, o1, Operand::Stack(o2)) => {
                let instr1 = Instruction::Mov {
                    source: Operand::Stack(o2),
                    dest: Operand::Register(Reg::R11),
                };
                let instr2 = Instruction::Binary(BinaryOp::Mult, o1, Operand::Register(Reg::R11));
                let instr3 = Instruction::Mov {
                    source: Operand::Register(Reg::R11),
                    dest: Operand::Stack(o2),
                };
                updated_instructions.push(instr1);
                updated_instructions.push(instr2);
                updated_instructions.push(instr3);
            }
            Instruction::Binary(BinaryOp::Sal, Operand::Stack(o1), Operand::Stack(o2)) => {
                let instr1 = Instruction::Mov {
                    source: Operand::Stack(o1),
                    dest: Operand::Register(Reg::CL),
                };
                let instr2 = Instruction::Binary(
                    BinaryOp::Sal,
                    Operand::Register(Reg::CL),
                    Operand::Stack(o2),
                );
                updated_instructions.push(instr1);
                updated_instructions.push(instr2);
            }
            Instruction::Binary(BinaryOp::Sar, Operand::Stack(o1), Operand::Stack(o2)) => {
                let instr1 = Instruction::Mov {
                    source: Operand::Stack(o1),
                    dest: Operand::Register(Reg::CL),
                };
                let instr2 = Instruction::Binary(
                    BinaryOp::Sar,
                    Operand::Register(Reg::CL),
                    Operand::Stack(o2),
                );
                updated_instructions.push(instr1);
                updated_instructions.push(instr2);
            }
            Instruction::Binary(BinaryOp::And, Operand::Stack(o1), Operand::Stack(o2)) => {
                let instr1 = Instruction::Mov {
                    source: Operand::Stack(o1),
                    dest: Operand::Register(Reg::R10),
                };
                let instr2 = Instruction::Binary(
                    BinaryOp::And,
                    Operand::Register(Reg::R10),
                    Operand::Stack(o2),
                );
                updated_instructions.push(instr1);
                updated_instructions.push(instr2);
            }
            Instruction::Binary(BinaryOp::Xor, Operand::Stack(o1), Operand::Stack(o2)) => {
                let instr1 = Instruction::Mov {
                    source: Operand::Stack(o1),
                    dest: Operand::Register(Reg::R10),
                };
                let instr2 = Instruction::Binary(
                    BinaryOp::Xor,
                    Operand::Register(Reg::R10),
                    Operand::Stack(o2),
                );
                updated_instructions.push(instr1);
                updated_instructions.push(instr2);
            }
            Instruction::Binary(BinaryOp::Or, Operand::Stack(o1), Operand::Stack(o2)) => {
                let instr1 = Instruction::Mov {
                    source: Operand::Stack(o1),
                    dest: Operand::Register(Reg::R10),
                };
                let instr2 = Instruction::Binary(
                    BinaryOp::Or,
                    Operand::Register(Reg::R10),
                    Operand::Stack(o2),
                );
                updated_instructions.push(instr1);
                updated_instructions.push(instr2);
            }
            _ => updated_instructions.push(instr),
        }
    }

    Program {
        function_def: FunctionDef {
            name,
            instructions: updated_instructions,
        },
    }
}
