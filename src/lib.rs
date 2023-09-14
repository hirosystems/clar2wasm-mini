// use std::error::Error;
// use std::ops::Deref;

// use regex;
// use sexp::{self, Atom, Sexp};
// use walrus::ir::BinaryOp;
// use walrus::{FunctionBuilder, InstrSeqBuilder, Module, ModuleConfig, ValType};
// use wasmtime::{Engine, Linker, Store};

// mod returnbuf;

mod arithmetic;
mod cvm;
mod words;

#[cfg(test)]
mod test_utils;

pub type CResult<T> = Result<T, Box<dyn std::error::Error>>;

pub use cvm::*;

// use returnbuf::*;

// pub struct Transpiled {
//     module: wasmtime::Module,
//     engine: Engine,
// }

// enum ClarAtom {
//     Int(i128),
//     UInt(u128),
// }

// impl ClarType {
//     fn to_wasm_type(&self) -> &[ValType] {
//         match self {
//             ClarType::Int | ClarType::UInt => &[ValType::I64, ValType::I64],
//         }
//     }
// }

// impl Deref for ClarSign {
//     type Target = [ValType];
//     fn deref(&self) -> &Self::Target {
//         &self.walrus
//     }
// }

// impl Emit for ClarAtom {
//     fn emit(&self, builder: &mut InstrSeqBuilder) {
//         match self {
//             ClarAtom::Int(a) => {
//                 builder.i64_const(*a as i64);
//                 builder.i64_const((*a << 64) as i64);
//             }
//             ClarAtom::UInt(a) => {
//                 builder.i64_const(*a as i64);
//                 builder.i64_const((*a << 64) as i64);
//             }
//         }
//     }
// }

// impl ClarAtom {
//     fn parse(atom: &Atom) -> Self {
//         match atom {
//             Atom::I(n) => ClarAtom::Int(*n as i128),
//             Atom::S(s) => {
//                 if let Some(cap) = regex::Regex::new("u([0-9])$").unwrap().captures(s) {
//                     ClarAtom::UInt(u128::from_str_radix(&cap[1], 10).unwrap())
//                 } else {
//                     todo!()
//                 }
//             }
//             _ => todo!(),
//         }
//     }
// }

// struct Parsed {
//     ast: AST,
//     input: ClarSign,
//     output: ClarSign,
// }

// enum AST {
//     Atom(ClarAtom),
//     Procedure(Box<dyn Procedure>),
// }

// impl Emit for AST {
//     fn emit(&self, builder: &mut InstrSeqBuilder) {
//         match self {
//             AST::Atom(a) => a.emit(builder),
//             AST::Procedure(p) => p.emit(builder),
//         }
//     }
// }

// pub trait Procedure: Sync + Emit {
//     fn input(&self) -> ClarSign;
//     fn output(&self) -> ClarSign;
// }

// struct Add(Vec<Parsed>);

// impl Procedure for Add {
//     fn input(&self) -> ClarSign {
//         // todo: make dynamic
//         ClarSign::new(&[ClarType::Int, ClarType::Int])
//     }

//     fn output(&self) -> ClarSign {
//         ClarSign::new(&[ClarType::Int])
//     }
// }

// fn resolve_proc(mut body: Vec<Sexp>) -> Result<Box<dyn Procedure>, Box<dyn Error>> {
//     let first = body.remove(0);
//     let args = body;
//     match first {
//         Sexp::Atom(Atom::S(s)) => {
//             if s == "+" {
//                 let parsed: Result<Vec<Parsed>, _> =
//                     args.into_iter().map(Parsed::from_sexp).collect();
//                 Ok(Box::new(Add(parsed?)))
//             } else {
//                 todo!()
//             }
//         }
//         _ => todo!(),
//     }
// }

// impl Parsed {
//     fn from_sexp(sexp: Sexp) -> Result<Self, Box<dyn Error>> {
//         match sexp {
//             Sexp::Atom(ref a) => Ok(Parsed {
//                 ast: AST::Atom(ClarAtom::parse(a)),
//                 input: ClarSign::empty(),
//                 output: ClarSign::new(&[ClarType::Int]),
//             }),
//             Sexp::List(list) => {
//                 if list.is_empty() {
//                     panic!("Invalid syntax")
//                 };
//                 let proc = resolve_proc(list)?;

//                 let input = proc.input();
//                 let output = proc.output();

//                 Ok(Parsed {
//                     ast: AST::Procedure(proc),
//                     input,
//                     output,
//                 })
//             }
//         }
//     }

//     fn new(text: &str) -> Result<Self, Box<dyn Error>> {
//         Self::from_sexp(sexp::parse(text)?)
//     }
// }

// impl Emit for Parsed {
//     fn emit(&self, builder: &mut InstrSeqBuilder) {
//         match &self.ast {
//             AST::Atom(a) => a.emit(builder),
//             AST::Procedure(a) => a.emit(builder),
//         }
//     }
// }

// pub trait Emit {
//     fn emit(&self, builder: &mut InstrSeqBuilder);
// }

// impl Transpiled {
//     pub fn new(text: &str) -> Result<Self, Box<dyn Error>> {
//         let parsed = Parsed::new(text)?;
//         let config = ModuleConfig::new();
//         let mut module = Module::with_config(config);

//         // toplevel fn builder
//         let mut builder = FunctionBuilder::new(&mut module.types, &[], &*parsed.output);

//         parsed.emit(&mut builder.func_body());

//         let toplevel = builder.finish(vec![], &mut module.funcs);

//         module.exports.add(".toplevel", toplevel);

//         let code = module.emit_wasm();

//         println!("{}", wabt::wasm2wat(&code)?);

//         let engine = Engine::default();
//         let module = wasmtime::Module::new(&engine, code)?;

//         Ok(Transpiled { module, engine })
//     }

//     pub fn call_toplevel<R>(&self) -> Result<R, Box<dyn Error>>
//     where
//         R: ReturnBufInterop,
//     {
//         let mut store = Store::new(&self.engine, ());
//         let linker = Linker::new(&self.engine);
//         let instance = linker.instantiate(&mut store, &self.module)?;

//         let toplevel = instance.get_func(&mut store, ".toplevel").unwrap();

//         let mut ret = ReturnBuf::new();

//         toplevel.call(&mut store, &mut [], R::req_space(&mut ret))?;

//         Ok(R::from_buf(&ret))
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn addition() -> Result<(), Box<dyn Error>> {
//         let transp = Transpiled::new("(+ 3 -2)")?;
//         assert_eq!(transp.call_toplevel::<i64>()?, 1);

//         Ok(())
//     }

//     #[test]
//     #[ignore]
//     fn nested_addition() -> Result<(), Box<dyn Error>> {
//         let transp = Transpiled::new("(+ (+ 3 3) 3)")?;
//         assert_eq!(transp.call_toplevel::<i64>()?, 9);

//         Ok(())
//     }

//     #[test]
//     fn atom() -> Result<(), Box<dyn Error>> {
//         let transp = Transpiled::new("4")?;
//         assert_eq!(transp.call_toplevel::<i128>()?, 4);

//         let transp = Transpiled::new("-4")?;
//         assert_eq!(transp.call_toplevel::<i128>()?, -4);

//         let transp = Transpiled::new("u4")?;
//         assert_eq!(transp.call_toplevel::<u128>()?, 4);

//         Ok(())
//     }
// }
