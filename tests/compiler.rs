use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "luac-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).unwrap();
        Self(dir)
    }

    fn write(&self, name: &str, source: impl AsRef<[u8]>) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, source).unwrap();
        path
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn returns_a_native_lua51_chunk_without_executing_source() {
    let fixture = Fixture::new();
    let source = fixture.write("compile.lua", "error('this must never execute')");
    let bytes = luac::compile_file(source).unwrap();
    assert_eq!(&bytes[..6], b"\x1bLua\x51\x00");
    assert_eq!(bytes[6], u8::from(cfg!(target_endian = "little")));
    assert_eq!(bytes[7], size_of::<std::ffi::c_int>() as u8);
    assert_eq!(bytes[8], size_of::<usize>() as u8);
    assert_eq!(&bytes[9..12], &[4, 8, 0]);
}

#[test]
fn preserves_io_errors_and_reports_syntax_location() {
    let fixture = Fixture::new();
    assert_eq!(
        luac::compile_file(fixture.0.join("missing.lua"))
            .unwrap_err()
            .kind(),
        ErrorKind::NotFound
    );
    let source = fixture.write("syntax.lua", "#!/usr/bin/env lua5.1\nlocal = 1\n");
    let error = luac::compile_file(source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert!(error.to_string().contains("syntax.lua:2:"), "{error}");
}

#[test]
fn rejects_binary_input_and_newer_lua_syntax() {
    let fixture = Fixture::new();
    for source in [
        &b"\x1bLua\x51\x00"[..],
        b"#!/usr/bin/env lua5.1\n\x1bLua\x51\x00",
        b"return 7 // 2",
        b"::label:: goto label",
    ] {
        let path = fixture.write("invalid.lua", source);
        assert_eq!(
            luac::compile_file(path).unwrap_err().kind(),
            ErrorKind::InvalidData
        );
    }
}

#[test]
fn accepts_empty_sources_shebangs_and_non_utf8_bytes() {
    let fixture = Fixture::new();
    for source in [&b""[..], b"#!lua5.1", b"#!lua5.1\r\nreturn '\xff\x00'"] {
        let path = fixture.write("source.lua", source);
        assert!(
            luac::compile_file(path)
                .unwrap()
                .starts_with(b"\x1bLua\x51")
        );
    }
}

#[test]
fn independent_calls_can_compile_in_parallel() {
    let fixture = Fixture::new();
    let source = fixture.write("parallel.lua", include_bytes!("fixtures/program.lua"));
    let expected = luac::compile_file(&source).unwrap();
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..8)
            .map(|_| scope.spawn(|| luac::compile_file(&source).unwrap()))
            .collect();
        for handle in handles {
            assert_eq!(handle.join().unwrap(), expected);
        }
    });
}

#[test]
fn parser_errors_do_not_affect_later_compilations() {
    let fixture = Fixture::new();
    let deep = format!("return {}1{}", "(".repeat(500), ")".repeat(500));
    for source in [
        "local function broken(a, b) local t = {1, 2, 3}",
        "return [=[unfinished long string",
        "return 12invalid",
        "local = 1",
        &deep,
    ] {
        let input = fixture.write("error.lua", source);
        assert_eq!(
            luac::compile_file(&input).unwrap_err().kind(),
            ErrorKind::InvalidData
        );
        fs::write(&input, "return 42").unwrap();
        assert!(luac::compile_file(&input).unwrap().starts_with(b"\x1bLua"));
    }
}

#[test]
fn cli_writes_default_and_explicit_output() {
    let fixture = Fixture::new();
    let source = fixture.write("source.lua", "return 42");
    let expected = luac::compile_file(&source).unwrap();
    let output = fixture.0.join("custom.luac");
    success(Command::new(env!("CARGO_BIN_EXE_luac")).arg(&source));
    assert_eq!(fs::read(source.with_extension("luac")).unwrap(), expected);
    success(
        Command::new(env!("CARGO_BIN_EXE_luac"))
            .arg(&source)
            .arg("-o")
            .arg(&output),
    );
    assert_eq!(fs::read(output).unwrap(), expected);
}

#[test]
fn cli_preserves_source_and_existing_output_on_error() {
    let fixture = Fixture::new();
    let source = fixture.write("source.lua", "return 42");
    let result = Command::new(env!("CARGO_BIN_EXE_luac"))
        .arg(&source)
        .arg("-o")
        .arg(&source)
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert_eq!(fs::read(&source).unwrap(), b"return 42");

    let invalid = fixture.write("invalid.lua", "local =");
    let result = Command::new(env!("CARGO_BIN_EXE_luac"))
        .arg(invalid)
        .arg("-o")
        .arg(&source)
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert_eq!(fs::read(source).unwrap(), b"return 42");
}

fn success(command: &mut Command) -> Output {
    let output = command.output().expect("could not start command");
    assert!(
        output.status.success(),
        "{command:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

// Run with LUA51=/path/to/lua5.1 LUAC51=/path/to/luac5.1
// cargo test -- --include-ignored
#[test]
#[ignore = "requires official Lua 5.1 interpreter and compiler (LUA51 and LUAC51)"]
fn matches_official_compiler_and_executes_with_lua51() {
    let lua = std::env::var_os("LUA51").unwrap_or_else(|| "lua5.1".into());
    let luac = std::env::var_os("LUAC51").unwrap_or_else(|| "luac5.1".into());
    let version = success(Command::new(&lua).arg("-v"));
    assert!(
        String::from_utf8_lossy(&version.stdout).contains("Lua 5.1")
            || String::from_utf8_lossy(&version.stderr).contains("Lua 5.1")
    );

    let fixture = Fixture::new();
    let large_source = format!("local s = [=[{}]=]; print(#s)", "x".repeat(128 * 1024));
    let constants = (0..1500)
        .map(|i| format!("{i}, 'constant_{i}', "))
        .collect::<String>();
    let many_constants = format!("local t = {{{constants}}}; print(#t, t[1], t[#t])");
    for (name, source) in [
        ("empty.lua", &b""[..]),
        (
            "shebang.lua",
            &b"#!/usr/bin/env lua5.1\r\nprint(42)\r\n"[..],
        ),
        ("shebang-only.lua", &b"#!lua5.1"[..]),
        ("raw.lua", &b"print(#'\xff\x00')"[..]),
        ("large.lua", large_source.as_bytes()),
        ("constants.lua", many_constants.as_bytes()),
        (
            "numbers.lua",
            &b"local t = {1e300, -1e300, 0, -0, 0xFF, 1/0, 0/0}; print(#t, t[5])"[..],
        ),
        ("program.lua", &include_bytes!("fixtures/program.lua")[..]),
    ] {
        let input = fixture.write(name, source);
        let ours = fixture.0.join("ours.luac");
        let reference = fixture.0.join("reference.luac");
        success(
            Command::new(env!("CARGO_BIN_EXE_luac"))
                .arg(&input)
                .arg("-o")
                .arg(&ours),
        );
        success(Command::new(&luac).arg("-o").arg(&reference).arg(&input));
        // Compare as slices to avoid printing a huge Vec if a regression occurs.
        assert!(
            fs::read(&ours).unwrap() == fs::read(reference).unwrap(),
            "{name}"
        );
        assert_same_execution(Path::new(&lua), &input, &ours);
    }

    for source in [
        "local = 1",
        "return [=[unfinished",
        "local function f() return 1",
        "return 1e+",
        "break",
    ] {
        let input = fixture.write("syntax.lua", source);
        let error = luac::compile_file(&input).unwrap_err();
        let reference = Command::new(&luac).arg("-p").arg(&input).output().unwrap();
        assert!(!reference.status.success());
        let message = String::from_utf8_lossy(&reference.stderr);
        assert!(message.contains(&error.to_string()), "{message} != {error}");
    }
}

fn assert_same_execution(lua: &Path, source: &Path, compiled: &Path) {
    let source_output = success(Command::new(lua).arg(source));
    let compiled_output = success(Command::new(lua).arg(compiled));
    assert_eq!(compiled_output.stdout, source_output.stdout);
    assert_eq!(compiled_output.stderr, source_output.stderr);
}
