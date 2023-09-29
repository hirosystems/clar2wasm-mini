use clarity::vm::{ClarityName, SymbolicExpression, Value};
use linkme::distributed_slice;

use crate::words::{normalize_multiple_args, Word};
use crate::CResult;
use crate::{Proc, ProgramBuilder, Stack, ValueType};

#[derive(Debug)]
struct Mul;

#[distributed_slice(crate::words::WORDS)]
static MUL: &'static dyn Word = &Mul;

impl Word for Mul {
    fn name(&self) -> ClarityName {
        "*".into()
    }

    fn normalize(&self, args: &[SymbolicExpression]) -> Vec<SymbolicExpression> {
        normalize_multiple_args(self, args)
    }

    fn emit(&self, builder: &mut ProgramBuilder) -> CResult<()> {
        match builder.consume_two()? {
            (ValueType::Int, ValueType::Int) => {
                builder.push_proc(&MulInt);
            }
            (ValueType::UInt, ValueType::UInt) => {
                builder.push_proc(&MulUInt);
            }
            _ => panic!("invalid Mul"),
        }
        Ok(())
    }
}

#[derive(Debug)]
struct MulInt;

impl Proc for MulInt {
    fn execute(&self, stack: &mut Stack) -> CResult<()> {
        if let (Value::Int(a), Value::Int(b)) = stack.consume_two()? {
            stack.push(Value::Int(a * b))
        } else {
            panic!("error")
        }
        Ok(())
    }

    fn output(&self) -> ValueType {
        ValueType::Int
    }
}

#[derive(Debug)]
struct MulUInt;

impl Proc for MulUInt {
    fn execute(&self, stack: &mut Stack) -> CResult<()> {
        if let (Value::UInt(a), Value::UInt(b)) = stack.consume_two()? {
            stack.push(Value::UInt(a * b))
        } else {
            panic!("error")
        }
        Ok(())
    }

    fn output(&self) -> ValueType {
        ValueType::UInt
    }
}
