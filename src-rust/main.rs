// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(lazy_cell)]

pub mod jit;
pub mod ctx;

#[cfg(feature = "llvm-8")]
pub use llvm_sys_80 as llvm;
#[cfg(feature = "llvm-9")]
pub use llvm_sys_90 as llvm;
#[cfg(feature = "llvm-10")]
pub use llvm_sys_100 as llvm;
#[cfg(feature = "llvm-11")]
pub use llvm_sys_110 as llvm;
#[cfg(feature = "llvm-12")]
pub use llvm_sys_120 as llvm;
#[cfg(feature = "llvm-13")]
pub use llvm_sys_130 as llvm;
#[cfg(feature = "llvm-14")]
pub use llvm_sys_140 as llvm;
#[cfg(feature = "llvm-15")]
pub use llvm_sys_150 as llvm;
#[cfg(feature = "llvm-16")]
pub use llvm_sys_160 as llvm;

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}