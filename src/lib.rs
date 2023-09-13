use std::error::Error;

use sexp::{self, Atom, Sexp};
use walrus::ir::BinaryOp;
use walrus::{FunctionBuilder, InstrSeqBuilder, Module, ModuleConfig, ValType};
use wasmtime::{Engine, Linker, Store, Val};

pub struct Transpiled {
    module: wasmtime::Module,
    engine: Engine,
}

enum ClarityAtom {
    Int(i64),
    Uint(i64),
}

impl Emit for ClarityAtom {
    fn emit(&self, builder: &mut InstrSeqBuilder) {
        match self {
            ClarityAtom::Int(a) => {
                builder.i64_const(*a);
            }
            _ => todo!(),
        }
    }
}

impl ClarityAtom {
    fn parse(atom: &Atom) -> Self {
        match atom {
            Atom::I(n) => ClarityAtom::Int(*n as i64),
            _ => todo!(),
        }
    }
}

struct Parsed {
    ast: AST,
    input: &'static [ValType],
    output: &'static [ValType],
}

enum AST {
    Atom(ClarityAtom),
    Procedure(Box<dyn Procedure>),
}

impl Emit for AST {
    fn emit(&self, builder: &mut InstrSeqBuilder) {
        match self {
            AST::Atom(a) => a.emit(builder),
            AST::Procedure(p) => p.emit(builder),
        }
    }
}

pub trait Procedure: Sync + Emit {
    fn input(&self) -> &'static [ValType];
    fn output(&self) -> &'static [ValType];
}

struct Add(Vec<Parsed>);

impl Procedure for Add {
    fn input(&self) -> &'static [ValType] {
        // todo: make dynamic
        &[ValType::I64, ValType::I64]
    }

    fn output(&self) -> &'static [ValType] {
        &[ValType::I64]
    }
}

impl Emit for Add {
    fn emit(&self, builder: &mut InstrSeqBuilder) {
        for a in &self.0 {
            a.emit(builder)
        }
        builder.binop(BinaryOp::I64Add);
    }
}

fn resolve_proc(mut body: Vec<Sexp>) -> Result<Box<dyn Procedure>, Box<dyn Error>> {
    let first = body.remove(0);
    let args = body;
    match first {
        Sexp::Atom(Atom::S(s)) => {
            if s == "+" {
                let parsed: Result<Vec<Parsed>, _> =
                    args.into_iter().map(Parsed::from_sexp).collect();
                Ok(Box::new(Add(parsed?)))
            } else {
                todo!()
            }
        }
        _ => todo!(),
    }
}

impl Parsed {
    fn from_sexp(sexp: Sexp) -> Result<Self, Box<dyn Error>> {
        match sexp {
            Sexp::Atom(ref a) => Ok(Parsed {
                ast: AST::Atom(ClarityAtom::parse(a)),
                input: &[],
                output: &[ValType::I64],
            }),
            Sexp::List(list) => {
                if list.is_empty() {
                    panic!("Invalid syntax")
                };
                let proc = resolve_proc(list)?;

                let input = proc.input();
                let output = proc.output();

                Ok(Parsed {
                    ast: AST::Procedure(proc),
                    input,
                    output,
                })
            }
        }
    }

    fn new(text: &str) -> Result<Self, Box<dyn Error>> {
        Self::from_sexp(sexp::parse(text)?)
    }
}

impl Emit for Parsed {
    fn emit(&self, builder: &mut InstrSeqBuilder) {
        match &self.ast {
            AST::Atom(a) => a.emit(builder),
            AST::Procedure(a) => a.emit(builder),
        }
    }
}

pub trait Emit {
    fn emit(&self, builder: &mut InstrSeqBuilder);
}

pub struct ReturnBuf([Val; 2]);

impl ReturnBuf {
    fn new() -> Self {
        ReturnBuf([Val::I32(0), Val::I32(0)])
    }
}

pub trait ReturnBufInterop {
    fn from_buf(buf: &ReturnBuf) -> Self;
    fn req_space(buf: &mut ReturnBuf) -> &mut [Val];
}

impl ReturnBufInterop for i128 {
    fn from_buf(buf: &ReturnBuf) -> Self {
        match &buf.0 {
            &[Val::I64(lo), Val::I64(hi), ..] => lo as i128 + ((hi as i128) >> 64),
            _ => todo!(),
        }
    }
    fn req_space(buf: &mut ReturnBuf) -> &mut [Val] {
        &mut buf.0[..2]
    }
}

impl ReturnBufInterop for i64 {
    fn from_buf(buf: &ReturnBuf) -> Self {
        match &buf.0 {
            &[Val::I64(i), ..] => i,
            _ => todo!(),
        }
    }

    fn req_space(buf: &mut ReturnBuf) -> &mut [Val] {
        &mut buf.0[..1]
    }
}

impl Transpiled {
    pub fn new(text: &str) -> Result<Self, Box<dyn Error>> {
        let parsed = Parsed::new(text)?;
        let config = ModuleConfig::new();
        let mut module = Module::with_config(config);

        // toplevel fn builder
        let mut builder = FunctionBuilder::new(&mut module.types, &[], parsed.output);

        parsed.emit(&mut builder.func_body());

        let toplevel = builder.finish(vec![], &mut module.funcs);

        module.exports.add(".toplevel", toplevel);

        let code = module.emit_wasm();

        let wat = wabt::wasm2wat(&code)?;

        println!("{wat}");

        let engine = Engine::default();
        let module = wasmtime::Module::new(&engine, code)?;

        Ok(Transpiled { module, engine })
    }

    pub fn call_toplevel<R>(&self) -> Result<R, Box<dyn Error>>
    where
        R: ReturnBufInterop,
    {
        let mut store = Store::new(&self.engine, ());
        let linker = Linker::new(&self.engine);
        let instance = linker.instantiate(&mut store, &self.module)?;

        println!("a");

        let toplevel = instance.get_func(&mut store, ".toplevel").unwrap();

        let mut ret = ReturnBuf::new();

        println!("b");

        toplevel.call(&mut store, &mut [], R::req_space(&mut ret))?;

        println!("c");

        Ok(R::from_buf(&ret))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addition() -> Result<(), Box<dyn Error>> {
        let transp = Transpiled::new("(+ 3 3)")?;

        assert_eq!(transp.call_toplevel::<i64>()?, 6);

        Ok(())
    }

    #[test]
    fn atom() -> Result<(), Box<dyn Error>> {
        let transp = Transpiled::new("-4")?;

        assert_eq!(transp.call_toplevel::<i64>()?, -4);

        Ok(())
    }
}
