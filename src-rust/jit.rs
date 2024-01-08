use serde::{Serialize, Deserialize};
use std::{ptr::null_mut, sync::Arc};
use crate::llvm::{
    orc2::{*, lljit::*}, *,  
};

#[derive(Debug)]
pub struct Jit {
    lljit: *mut LLVMOrcOpaqueLLJIT,
}

#[derive(Debug, Clone)]
pub struct JitFnPure {
    rc: Arc<Jit>,
    ty: (*mut LLVMType, *mut LLVMType),
    state: JitVal,
    inner: fn (*const u8, *mut u8, *mut u8),
}

impl JitFnPure {
    pub fn init(&self) -> JitFnMut {
        JitFnMut {
            rc: Arc::clone(&self.rc), 
            ty: self.ty, 
            state: self.state.clone(), 
            inner: self.inner,
        }
    }
}

#[derive(Debug, Clone)]
pub struct JitFnMut {
    rc: Arc<Jit>,
    ty: (*mut LLVMType, *mut LLVMType),
    state: JitVal,
    inner: fn (*const u8, *mut u8, *mut u8),
}

impl JitFnMut {
    pub fn apply(&mut self, input: JitVal) -> JitVal {
        let mut out = JitVal::new(Arc::clone(&self.rc), self.ty.1);
        (self.inner)(
            input.inner.as_ptr(), 
            self.state.inner.as_mut_ptr(), 
            out.inner.as_mut_ptr()
        ); 
        return out;
    }
}

#[derive(Debug, Clone)]
pub struct JitVal {
    rc: Arc<Jit>,
    ty: *mut LLVMType,
    inner: Vec<u8>,
}

impl JitVal {
    fn new(rc: Arc<Jit>, ty: *mut LLVMType) -> Self {
        todo!()
    }
}

impl Jit {
    pub fn new() -> Jit {
        let lljit = unsafe {
            let builder = LLVMOrcCreateLLJITBuilder();
            let mut ptr = null_mut();
            LLVMOrcCreateLLJIT(&mut ptr, builder);
            LLVMOrcDisposeLLJITBuilder(builder);
            ptr
        };
        Self { lljit }
    }
    pub fn compile(self: &Arc<Self>) -> JitFnPure {
        todo!()
    }
}