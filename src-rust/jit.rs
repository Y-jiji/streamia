use serde::{Serialize, Deserialize};
use std::{ptr::null_mut, sync::{LazyLock, Mutex}};
use crate::{llvm::{
    orc2::{*, lljit::*},   
    error::*, target::*
}, ctx::Expr};

fn __init_native__() { unsafe {
    static BARRIER: LazyLock<Mutex<bool>> = LazyLock::new(
        || Mutex::new(false)
    );
    // make sure: 
    // + this protected section will run only once 
    // + after quiting, internal procedures must be completed by some thread
    let guard = BARRIER.lock().unwrap();
    if !*guard {
        LLVM_InitializeNativeAsmParser();
        LLVM_InitializeNativeAsmPrinter();
        LLVM_InitializeNativeTarget();
        LLVM_InitializeAllTargetInfos();
    }
    drop(guard);
} }

#[derive(Debug)]
pub struct Jit {
    lljit: *mut LLVMOrcOpaqueLLJIT,
}

unsafe fn err_to_string(raw: LLVMErrorRef) -> String {
    let mut msg = LLVMGetErrorMessage(raw);
    let mut vec = vec![];
    while *msg != b'\0' as i8 {
        vec.push(*msg as u8);
        msg = msg.offset(1);
    }
    LLVMConsumeError(raw);
    return String::from_utf8_lossy(&vec).to_string();
}

pub struct JVal<'a> {
    x: &'a (),
}

impl Jit {
    pub fn new() -> Result<Jit, String> { unsafe {
        __init_native__(); // initialize machine

        // target machine builder
        let mut builder_trg = null_mut();
        let err = LLVMOrcJITTargetMachineBuilderDetectHost(
            &mut builder_trg);
        if builder_trg.is_null() { Err(
            String::new() + 
            "LLVMOrcJITTargetMachineBuilderDetectHost: " + 
            &err_to_string(err)
        )? }
        
        // jit builder
        // SAFETY: builder_trg owned by builder_jit
        let builder_jit = LLVMOrcCreateLLJITBuilder();
        if builder_jit.is_null() { Err(
            String::new() + 
            "LLVMOrcCreateLLJITBuilder: return null pointer"
        )? }
        LLVMOrcLLJITBuilderSetJITTargetMachineBuilder(
            builder_jit, builder_trg);

        // build and handle errors
        // SAFETY: builder_jit will disposed after lljit creation
        let mut lljit = null_mut();
        let err = LLVMOrcCreateLLJIT(&mut lljit, builder_jit);
        if lljit.is_null() { Err(
            String::new() + 
            "LLVMOrcCreateLLJIT: " + 
            &err_to_string(err)
        )? }
        Ok(Jit { lljit })
    }}
    pub fn compile(&self, x: &Expr) -> JVal<'_> {
        // compile expression to value
        todo!()
    }
}

impl Drop for Jit {
    fn drop(&mut self) {
        unsafe { LLVMOrcDisposeLLJIT(self.lljit); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jit_create() {
        let jit = Jit::new();
        println!("{jit:?}");
    }
}