//! Compilador Lua 5.1 baseado no código oficial do Lua 5.1.5.
//!
//! ```no_run
//! let bytecode = luac::compile_file("main.lua")?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! O resultado é um chunk binário completo, com informações de depuração,
//! equivalente ao `luac` oficial para um arquivo. O formato usa a arquitetura
//! nativa (endianness e tamanhos de tipos), como o Lua 5.1 original.
//! O código é apenas compilado, nunca executado.

use std::ffi::{CString, c_char, c_int, c_void};
use std::io;
use std::path::Path;
use std::ptr;
use std::slice;

/// Compila um arquivo-fonte Lua 5.1 e retorna seu bytecode completo.
///
/// Aceita arquivos com shebang e fontes com bytes fora de UTF-8.
/// Erros de sintaxe retornam `InvalidData`, com arquivo e linha informados
/// pelo compilador. Erros de leitura preservam o erro de I/O original.
/// A entrada deve ser código-fonte; chunks já compilados são rejeitados.
pub fn compile_file(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    let path = path.as_ref();
    let contents = std::fs::read(path)?;
    // Like luaL_loadfile, retain a newline in place of the shebang so that
    // line numbers remain identical to the original file.
    let source = if contents.first() == Some(&b'#') {
        contents
            .iter()
            .position(|&byte| byte == b'\n')
            .map_or(&b"\n"[..], |newline| &contents[newline..])
    } else {
        &contents
    };
    if source.first() == Some(&0x1b) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected Lua source, received a binary chunk",
        ));
    }

    let mut name = vec![b'@'];
    name.extend_from_slice(path.as_os_str().as_encoded_bytes());
    let name =
        CString::new(name).map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    let mut bytecode = Vec::<u8>::new();
    // SAFETY: Buffers and the exclusive output pointer live for this call.
    // The C compiler owns and frees all parser allocations before returning.
    // Its error recovery never crosses a Rust frame.
    let status = unsafe {
        luac_compile(
            source.as_ptr().cast(),
            source.len(),
            name.as_ptr(),
            write_bytecode,
            ptr::from_mut(&mut bytecode).cast(),
        )
    };
    match status {
        0 => Ok(bytecode),
        4 => Err(io::Error::new(
            io::ErrorKind::OutOfMemory,
            "not enough memory to compile Lua source",
        )),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            String::from_utf8_lossy(&bytecode).into_owned(),
        )),
    }
}

unsafe extern "C" fn write_bytecode(
    _state: *mut c_void,
    data: *const c_void,
    len: usize,
    output: *mut c_void,
) -> c_int {
    // Lua can pass a null data pointer for an empty block.
    if len == 0 {
        return 0;
    }
    // SAFETY: compile_file supplies an exclusive Vec pointer; luac_compile supplies
    // len readable bytes. Neither pointer escapes this callback.
    let output = unsafe { &mut *output.cast::<Vec<u8>>() };
    // Do not panic/unwind across the C boundary on allocation failure.
    if output.try_reserve(len).is_err() {
        return 1;
    }
    output.extend_from_slice(unsafe { slice::from_raw_parts(data.cast(), len) });
    0
}

unsafe extern "C" {
    fn luac_compile(
        source: *const c_char,
        len: usize,
        name: *const c_char,
        writer: unsafe extern "C" fn(*mut c_void, *const c_void, usize, *mut c_void) -> c_int,
        output: *mut c_void,
    ) -> c_int;
}
