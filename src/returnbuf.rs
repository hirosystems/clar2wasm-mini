use wasmtime::Val;

pub struct ReturnBuf([Val; 2]);

impl ReturnBuf {
    pub fn new() -> Self {
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
            &[Val::I64(lo), Val::I64(hi), ..] => {
                println!("lo {lo}, hi {hi}");
                lo as i128 + ((hi as i128) >> 64)
            }
            _ => todo!(),
        }
    }
    fn req_space(buf: &mut ReturnBuf) -> &mut [Val] {
        &mut buf.0[..2]
    }
}

impl ReturnBufInterop for u128 {
    fn from_buf(buf: &ReturnBuf) -> Self {
        match &buf.0 {
            &[Val::I64(lo), Val::I64(hi), ..] => lo as u128 + ((hi as u128) >> 64),
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

impl ReturnBufInterop for u64 {
    fn from_buf(buf: &ReturnBuf) -> Self {
        match &buf.0 {
            &[Val::I64(i), ..] => i as u64,
            _ => todo!(),
        }
    }

    fn req_space(buf: &mut ReturnBuf) -> &mut [Val] {
        &mut buf.0[..1]
    }
}
