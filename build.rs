fn main() {
    let source = "vendor/lua-5.1.5/src";
    // Compiler-only subset of Lua 5.1.5, with a per-compilation allocator.
    let files = [
        "lcode.c",
        "lcompiler.c",
        "ldump.c",
        "llex.c",
        "lmem.c",
        "lobject.c",
        "lopcodes.c",
        "lparser.c",
        "lstring.c",
        "ltable.c",
        "lzio.c",
    ];

    println!("cargo:rerun-if-changed={source}");
    cc::Build::new()
        .include(source)
        .files(files.map(|file| format!("{source}/{file}")))
        .std("c11")
        .compile("luac51");

    if std::env::var("CARGO_CFG_TARGET_FAMILY").as_deref() == Ok("unix") {
        println!("cargo:rustc-link-lib=m");
    }
}
