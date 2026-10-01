# luac

Compilador Lua 5.1 para Rust, baseado em um subconjunto do código oficial do
Lua 5.1.5. A biblioteca expõe uma única função: recebe o caminho de um
arquivo-fonte e retorna o bytecode completo em um `Vec<u8>`.

O compilador inclui lexer, parser, geração de bytecode e suporte de memória,
strings e tabelas. As rotinas da VM, execução de funções, corrotinas, GC,
bibliotecas padrão e carregamento de bytecode foram removidos.

## Requisitos

- Rust estável com suporte à edição 2024 e Cargo.
- Compilador C11 e ferramentas de link: GCC/Clang no Linux, Xcode Command Line
  Tools no macOS ou Visual Studio Build Tools com suporte a C/C++ no Windows.

O Cargo compila e vincula os fontes C incluídos em `vendor/`. Não é necessário
instalar Lua nem baixar seus fontes durante o build. A única dependência Rust
externa direta é `cc`, usada durante a compilação.

## Usar como dependência Git

Depois de colocar este diretório na raiz do seu repositório GitHub, adicione
ao `Cargo.toml` do projeto consumidor, substituindo `SEU_USUARIO/luac` pelo
endereço do repositório criado:

```toml
[dependencies]
luac = { git = "https://github.com/SEU_USUARIO/luac.git" }
```

Para fixar uma revisão, acrescente `rev = "SHA_DO_COMMIT"` à dependência.
Durante o desenvolvimento local, também é possível usar:

```toml
[dependencies]
luac = { path = "../luac" }
```

```rust,no_run
fn main() -> std::io::Result<()> {
    let bytecode: Vec<u8> = luac::compile_file("main.lua")?;

    // Opcional: salvar o resultado em arquivo.
    std::fs::write("main.luac", &bytecode)?;
    Ok(())
}
```

O caminho de entrada é relativo ao diretório de execução do programa. Para
entregar o vetor a um leitor que aceite `std::io::Read`, use
`std::io::Cursor::new(bytecode)` ou `bytecode.as_slice()`.

## API

```rust,ignore
pub fn compile_file(path: impl AsRef<std::path::Path>)
    -> std::io::Result<Vec<u8>>;
```

A função apenas compila; nenhum código Lua é executado. Aceita shebang e
fontes com bytes fora de UTF-8. Cada chamada possui seu próprio contexto de
compilação e libera suas alocações ao terminar, inclusive em caso de erro.

Erros de leitura preservam o erro de I/O original. Erros de sintaxe retornam
`InvalidData` com a mensagem do parser; falta de memória detectada pelo
compilador ou pelo buffer de saída retorna `OutOfMemory`. Chunks já compilados
não são aceitos como entrada.

O resultado é um chunk Lua 5.1 completo, com cabeçalho, instruções, constantes,
protótipos e informações de depuração. Como no compilador original, usa os
tamanhos de tipos e a ordem dos bytes da arquitetura em que é executado;
não é um formato de bytecode independente de arquitetura.

## Linha de comando

Na raiz do repositório:

```sh
cargo run --release -- tests/fixtures/program.lua -o target/program.luac
```

Para instalar o comando a partir de uma cópia local:

```sh
cargo install --path . --locked
luac main.lua -o main.luac
```

Sem `-o`, a saída usa o caminho de entrada com a extensão `.luac`. O comando
salva o resultado de `compile_file()` e não executa o programa.

## Verificação

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo package --locked
```

O teste de compatibilidade é ignorado por padrão porque requer o interpretador
e o compilador oficiais Lua 5.1. Para executá-lo, informe seus caminhos:

```sh
LUA51=/caminho/para/lua5.1 LUAC51=/caminho/para/luac5.1 \
  cargo test --locked -- --include-ignored
```

Esse teste compara os bytes com o `luac` oficial e executa o resultado com o
Lua 5.1. O teste nativo de falhas de alocação e escrita tem instruções em
[tests/native/allocations.c](tests/native/allocations.c).

A configuração de CI verifica o crate em Linux, macOS e Windows. Um job Linux
também compila a distribuição oficial Lua 5.1.5 para a comparação, executa os
testes nativos com sanitizadores e verifica a ausência de símbolos da VM.
Esses downloads são exclusivos dos testes de compatibilidade.

## Repositório independente

Copie os arquivos deste diretório, incluindo `.github/`, `.gitignore`,
`Cargo.lock` e `vendor/`, para a raiz do novo repositório. Exclua `target/`.
Nenhum arquivo do projeto que contém esta pasta é necessário.

O campo `repository` do `Cargo.toml` pode ser preenchido com a URL definitiva
depois da criação do repositório. O uso como dependência Git não depende de
publicação no crates.io.

## Origem e licença

Licença [MIT](LICENSE). Os fontes C são derivados da
[distribuição oficial Lua 5.1.5](https://www.lua.org/ftp/lua-5.1.5.tar.gz),
com adaptações para remover a execução. O checksum da distribuição e a lista
de adaptações estão em [vendor/README](vendor/README). A licença original
está preservada em [vendor/lua-5.1.5/COPYRIGHT](vendor/lua-5.1.5/COPYRIGHT).
