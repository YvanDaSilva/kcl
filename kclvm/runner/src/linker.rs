use crate::command::Command;

use once_cell::sync::Lazy;
use std::ffi::CString;
use std::sync::Mutex;

static LINKER_MUTEX: Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(0i32));

/// KclvmLinker is mainly responsible for linking the libs generated by KclvmAssembler.
pub struct KclvmLinker;
impl KclvmLinker {
    /// Link the libs generated by method "gen_bc_or_ll_file".
    pub fn link_all_libs(lib_paths: Vec<String>, lib_path: String) -> String {
        let mut cmd = Command::new();
        // In the final stage of link, we can't ignore any undefined symbols and do
        // not allow external mounting of the implementation.
        cmd.link_libs_with_cc(&lib_paths, &lib_path)
    }
}

#[allow(dead_code)]
extern "C" {
    fn LldMachOMain(args: *const *const libc::c_char, size: libc::size_t) -> libc::c_int;
    fn LldELFMain(args: *const *const libc::c_char, size: libc::size_t) -> libc::c_int;
    fn LldMinGWMain(args: *const *const libc::c_char, size: libc::size_t) -> libc::c_int;
    fn LldWasmMain(args: *const *const libc::c_char, size: libc::size_t) -> libc::c_int;
}

/// LLD Linker main function.
/// Take an object file and turn it into a final linked binary ready for deployment.
/// The lld linker is totally not thread-safe.
/// Ref: https://github.com/llvm/llvm-project/blob/main/lld/tools/lld/lld.cpp
/// TODO: WASM target.
pub fn lld_main(args: &[CString]) -> bool {
    let mut command_line: Vec<*const libc::c_char> = Vec::with_capacity(args.len() + 1);

    let executable_name = CString::new("lld").unwrap();

    command_line.push(executable_name.as_ptr());

    for arg in args {
        command_line.push(arg.as_ptr());
    }

    let _lock = LINKER_MUTEX.lock().unwrap();

    #[cfg(target_os = "macos")]
    unsafe {
        LldMachOMain(command_line.as_ptr(), command_line.len()) == 0
    }
    #[cfg(target_os = "linux")]
    unsafe {
        LldELFMain(command_line.as_ptr(), command_line.len()) == 0
    }

    #[cfg(target_os = "windows")]
    true
}
