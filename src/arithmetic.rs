use clarity::vm::{ClarityName, SymbolicExpression, Value};
use linkme::distributed_slice;

use crate::cvm::{Proc, ProgramBuilder, Stack, ValueType};
use crate::words::Word;
use crate::CResult;

#[derive(Debug)]
struct Add;

#[distributed_slice(crate::words::WORDS)]
static ADD: &'static dyn Word = &Add;

impl Word for Add {
    fn name(&self) -> ClarityName {
        "+".into()
    }

    fn normalize(&self, args: &[SymbolicExpression]) -> Vec<SymbolicExpression> {
        // Converts from (+ 1 2 3 4) to (+ 1 (+ 2 (+ 3 4)))
        let mut args = args.to_vec();
        while args.len() > 2 {
            let mut tail = args.split_off(args.len() - 2);
            let b = tail.pop().unwrap();
            let a = tail.pop().unwrap();
            args.push(SymbolicExpression::list(Box::new([
                SymbolicExpression::atom(self.name()),
                a,
                b,
            ])))
        }
        args
    }

    fn emit(&self, builder: &mut ProgramBuilder) -> CResult<()> {
        match builder.consume_two()? {
            (ValueType::Int, ValueType::Int) => {
                builder.push_proc(&AddInt);
            }
            (ValueType::UInt, ValueType::UInt) => {
                builder.push_proc(&AddUInt);
            }
            _ => panic!("invalid Add"),
        }
        Ok(())
    }
}

#[derive(Debug)]
struct AddInt;

impl Proc for AddInt {
    fn execute(&self, stack: &mut Stack) -> CResult<()> {
        if let (Value::Int(a), Value::Int(b)) = stack.consume_two()? {
            stack.push(Value::Int(a + b))
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
struct AddUInt;

impl Proc for AddUInt {
    fn execute(&self, stack: &mut Stack) -> CResult<()> {
        if let (Value::UInt(a), Value::UInt(b)) = stack.consume_two()? {
            stack.push(Value::UInt(a + b))
        } else {
            panic!("error")
        }
        Ok(())
    }

    fn output(&self) -> ValueType {
        ValueType::UInt
    }
}
