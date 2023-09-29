use clarity::vm::{ClarityName, SymbolicExpression};
use lazy_static::lazy_static;
use linkme::distributed_slice;
use std::collections::HashMap;

use crate::CResult;
use crate::ProgramBuilder;

pub trait Word: Sync + core::fmt::Debug {
    fn name(&self) -> ClarityName;

    fn traverse(&self, args: &[SymbolicExpression], builder: &mut ProgramBuilder) -> CResult<()> {
        // default traversal just recurses on the arguments
        builder.ingest(args)
    }

    fn emit(&self, builder: &mut ProgramBuilder) -> CResult<()>;

    fn normalize(&self, args: &[SymbolicExpression]) -> Vec<SymbolicExpression> {
        args.to_vec()
    }
}

#[distributed_slice]
pub(crate) static WORDS: [&'static dyn Word] = [..];

lazy_static! {
    static ref WORDS_BY_NAME: HashMap<ClarityName, &'static dyn Word> = {
        let mut wbn = HashMap::new();

        for word in WORDS {
            wbn.insert(word.name(), &**word);
        }

        wbn
    };
}

pub fn lookup(name: &str) -> Option<&'static dyn Word> {
    WORDS_BY_NAME.get(name).copied()
}

pub fn normalize_multiple_args<W: Word>(
    w: &W,
    args: &[SymbolicExpression],
) -> Vec<SymbolicExpression> {
    // Converts from (+ 1 2 3 4) to (+ 1 (+ 2 (+ 3 4)))
    let mut args = args.to_vec();
    while args.len() > 2 {
        let mut tail = args.split_off(args.len() - 2);
        let b = tail.pop().unwrap();
        let a = tail.pop().unwrap();
        args.push(SymbolicExpression::list(Box::new([
            SymbolicExpression::atom(w.name()),
            a,
            b,
        ])))
    }
    args
}
