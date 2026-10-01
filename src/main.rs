use std::io;
use std::path::PathBuf;
use std::process::ExitCode;

fn run() -> io::Result<()> {
    let mut args = std::env::args_os().skip(1);
    let usage = || {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "uso: luac <arquivo.lua> [-o arquivo.luac]",
        )
    };
    let input = PathBuf::from(args.next().ok_or_else(usage)?);
    let output = match args.next() {
        None => input.with_extension("luac"),
        Some(flag) if flag == "-o" => PathBuf::from(args.next().ok_or_else(usage)?),
        Some(_) => return Err(usage()),
    };
    if args.next().is_some() {
        return Err(usage());
    }
    let bytecode = luac::compile_file(&input)?;
    if input == output
        || std::fs::canonicalize(&output)
            .is_ok_and(|output| std::fs::canonicalize(&input).is_ok_and(|input| input == output))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "o arquivo de saída deve ser diferente do código-fonte",
        ));
    }
    std::fs::write(output, bytecode)
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("luac: {error}");
            ExitCode::FAILURE
        }
    }
}
