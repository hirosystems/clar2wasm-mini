use clarity::vm::ast::ContractAST;
use clarity::vm::Value;
use clarity::vm::{SymbolicExpression, SymbolicExpressionType};

use crate::words;
use crate::CResult;

#[derive(Debug, Default)]
pub struct Stack(Vec<Value>);

#[derive(Debug)]
pub enum ValueType {
    Int,
    UInt,
}

impl From<&Value> for ValueType {
    fn from(value: &Value) -> Self {
        match value {
            Value::Int(_) => ValueType::Int,
            Value::UInt(_) => ValueType::UInt,
            _ => todo!(),
        }
    }
}

#[derive(Debug)]
pub enum Instruction {
    Literal(Value),
    Proc(&'static dyn Proc),
}

pub trait Proc: core::fmt::Debug {
    fn execute(&self, stack: &mut Stack) -> CResult<()>;
    fn output(&self) -> ValueType;
}

#[derive(Debug)]
pub struct Program(Vec<Instruction>);

pub struct ProgramBuilder {
    instructions: Vec<Instruction>,
    typestack: Vec<ValueType>,
}

#[derive(Debug)]
pub struct Execution<'a> {
    program: &'a Program,
    instruction_ptr: usize,
    stack: Stack,
}

impl<'a> Execution<'a> {
    fn eval(&mut self) -> CResult<Option<Value>> {
        while let Some(inst) = self.program.0.get(self.instruction_ptr) {
            match inst {
                Instruction::Literal(lit) => self.stack.push(lit.clone()),
                Instruction::Proc(p) => p.execute(&mut self.stack)?,
            }
            self.instruction_ptr += 1;
        }
        Ok(self.stack.0.pop())
    }
}

impl Program {
    pub fn eval(&mut self) -> CResult<Option<Value>> {
        let mut exec = Execution {
            program: self,
            instruction_ptr: 0,
            stack: Stack::default(),
        };
        exec.eval()
    }

    pub fn from_ast(ast: &ContractAST) -> CResult<Self> {
        let mut builder = ProgramBuilder {
            instructions: vec![],
            typestack: vec![],
        };

        builder.ingest(&ast.expressions)?;
        Ok(builder.build())
    }
}

impl ProgramBuilder {
    pub fn build(self) -> Program {
        Program(self.instructions)
    }

    pub fn ingest(&mut self, expressions: &[SymbolicExpression]) -> CResult<()> {
        for e in expressions {
            match &e.expr {
                SymbolicExpressionType::AtomValue(_a) => todo!("a"),
                SymbolicExpressionType::List(list) => match &**list {
                    [SymbolicExpression {
                        expr: SymbolicExpressionType::Atom(ref a),
                        ..
                    }, args @ ..] => {
                        let name = a.as_str();

                        if let Some(word) = words::lookup(name) {
                            let normalized = word.normalize(args);
                            word.traverse(&normalized, self)?;
                            word.emit(self)?;
                        } else {
                            panic!("unknown word {name}");
                        }
                    }
                    e => todo!("unhandled case {e:?}"),
                },
                SymbolicExpressionType::LiteralValue(a) => {
                    self.typestack.push(ValueType::from(a));
                    self.instructions.push(Instruction::Literal(a.clone()));
                }
                e @ _ => todo!("z {e:?}"),
            }
        }
        Ok(())
    }

    pub fn peek_two(&self) -> CResult<(&ValueType, &ValueType)> {
        let peek = &self.typestack[..2];
        Ok((&peek[0], &peek[1]))
    }

    pub fn consume_two(&mut self) -> CResult<(ValueType, ValueType)> {
        let b = self.typestack.pop().unwrap();
        let a = self.typestack.pop().unwrap();
        Ok((a, b))
    }

    pub fn push_proc(&mut self, proc: &'static dyn Proc) {
        self.typestack.push(proc.output());
        self.instructions.push(Instruction::Proc(proc))
    }
}

impl Stack {
    pub fn consume_two(&mut self) -> CResult<(Value, Value)> {
        let b = self.0.pop().unwrap();
        let a = self.0.pop().unwrap();
        Ok((a, b))
    }

    pub fn push(&mut self, val: Value) {
        self.0.push(val)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::{eval_ast, parse_to_ast};

    fn test_both(source: &str) -> CResult<()> {
        let ast = parse_to_ast(source)?;
        let mut prog = Program::from_ast(&ast)?;

        println!("PROG\n{:?}", prog);

        let a = eval_ast(&ast)?;
        let b = prog.eval()?;

        assert_eq!(a, b);
        Ok(())
    }

    #[test]
    fn test_trivial() -> CResult<()> {
        test_both("2")
    }

    #[test]
    fn test_simple() -> CResult<()> {
        test_both("(+ 1 1)")
    }

    #[test]
    fn test_nested() -> CResult<()> {
        test_both("(+ (+ 1 2) (+ 3 4))")
    }

    #[test]
    fn test_unsigned() -> CResult<()> {
        test_both("(+ (+ u1 u2) (+ u3 u4))")
    }

    #[test]
    fn test_multivalue() -> CResult<()> {
        test_both("(+ 1 2 3)")
    }
}
