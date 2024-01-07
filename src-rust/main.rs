// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod jit;
mod ctx;

fn main() {
    unsafe {
        // use machine code jit
        use jit::llvm::target::*;
        use jit::llvm::execution_engine::*;
        LLVMLinkInMCJIT();
        LLVM_InitializeNativeAsmParser(); 
        LLVM_InitializeNativeAsmPrinter();
        LLVM_InitializeNativeDisassembler();
        LLVM_InitializeNativeTarget();
    }
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}