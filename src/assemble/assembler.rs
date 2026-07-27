use crate::assemble::parse;

pub struct Program {
    pub function_def: FunctionDef,
}

pub struct FunctionDef {
    pub name: String,
    pub instructions: Vec<Instruction>,
}

pub enum Instruction {
    Mov { source: Operand, dest: Operand },
    Ret,
}

pub enum Operand {
    Imm(u64),
    Register,
}

pub fn assemble(program: parse::Program) -> Program {
    lower_program(program)
}

fn lower_program(program: parse::Program) -> Program {
    let function_def = lower_function_def(program.function_def);
    Program { function_def }
}

fn lower_function_def(function_def: parse::FunctionDef) -> FunctionDef {
    let name = function_def.name;
    let instructions = lower_statement(function_def.body);
    FunctionDef { name, instructions }
}

fn lower_statement(statement: parse::Statement) -> Vec<Instruction> {
    let parse::Statement::Return(parse::Expression::Constant(source)) = statement;
    let source = Operand::Imm(source);
    let dest = Operand::Register;
    let mut instructions: Vec<Instruction> = Vec::new();
    instructions.push(Instruction::Mov { source, dest });
    instructions.push(Instruction::Ret);
    instructions
}
