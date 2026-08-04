use crate::assemble::parse;

pub struct Program {
    pub function_def: FunctionDef,
}

impl Program {
    pub fn print_program(&self) {
        println!("\nTACKY:");
        self.function_def.print_function_def();
    }
}

pub struct FunctionDef {
    pub name: String,
    pub body: Vec<Instruction>,
}

impl FunctionDef {
    fn print_function_def(&self) {
        println!("{}:", self.name);
        for instr in &self.body {
            match instr {
                Instruction::Ret(val) => {
                    println!("Return({:?})", val);
                }
                Instruction::Unary {
                    unary_operator,
                    src,
                    dest,
                } => {
                    println!("Unary({:?}, {:?}, {:?})", unary_operator, src, dest);
                }
            }
        }
    }
}

pub enum Instruction {
    Ret(Value),
    // dest must be Value::Var
    Unary {
        unary_operator: UnaryOp,
        src: Value,
        dest: Value,
    },
}

#[derive(Debug)]
pub enum Value {
    Constant(u64),
    Var(String),
}

#[derive(Debug)]
pub enum UnaryOp {
    Complement,
    Negation,
}

impl From<parse::UnaryOp> for UnaryOp {
    fn from(op: parse::UnaryOp) -> Self {
        match op {
            parse::UnaryOp::Complement => UnaryOp::Complement,
            parse::UnaryOp::Negation => UnaryOp::Negation,
        }
    }
}

pub struct TackyTransformer {
    var_count: usize,
}

impl TackyTransformer {
    fn lower_program(&mut self, program: parse::Program) -> Program {
        let function_def = self.lower_function_def(program.function_def);
        Program { function_def }
    }

    fn lower_function_def(&mut self, function_def: parse::FunctionDef) -> FunctionDef {
        let name = function_def.name;
        let body = self.lower_statement(function_def.body);
        FunctionDef { name, body }
    }

    fn lower_statement(&mut self, statement: parse::Statement) -> Vec<Instruction> {
        let mut instrs: Vec<Instruction> = Vec::new();
        match statement {
            parse::Statement::Return(exp) => {
                let val = self.lower_expression(exp, &mut instrs);
                instrs.push(Instruction::Ret(val));
            }
        }
        instrs
    }

    fn lower_expression(&mut self, exp: parse::Expression, instrs: &mut Vec<Instruction>) -> Value {
        match exp {
            parse::Expression::Constant(val) => Value::Constant(val),
            parse::Expression::Unary(op, exp) => {
                let unary_operator: UnaryOp = op.into();
                let src = self.lower_expression(*exp, instrs);
                let var = format!("tmp.{}", self.var_count);
                let dest = Value::Var(var.clone());
                self.var_count += 1;
                let instr = Instruction::Unary {
                    unary_operator,
                    src,
                    dest,
                };
                instrs.push(instr);
                Value::Var(var)
            }
        }
    }
}

pub fn transform(program: parse::Program) -> Program {
    let mut transformer = TackyTransformer { var_count: 0 };
    transformer.lower_program(program)
}
