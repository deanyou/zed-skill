# SKILL LSP Server

A Language Server Protocol implementation for the Cadence SKILL programming language, built with Rust.

## Features

- **Code Completion**: Built-in and user-defined function completion with documentation
- **Hover Information**: Function documentation, parameter info, and return types
- **Go to Definition**: Jump to function definitions
- **Find References**: Find all references to symbols
- **Document Symbols**: Browse file structure
- **Diagnostics**: Syntax error detection and warnings
- **Signature Help**: Parameter hints while typing
- **Code Formatting**: Basic SKILL code formatting
- **Code Actions**: Add documentation comments

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)

### Building

```bash
cargo build --release
```

The binary will be at `target/release/skill-lsp`.

### Installing

```bash
# Install to system PATH
sudo cp target/release/skill-lsp /usr/local/bin/

# Or install locally
cp target/release/skill-lsp ~/.local/bin/
```

## Usage

The LSP server communicates via stdio and follows the Language Server Protocol specification. It's designed to be used by Zed or any other LSP-compatible editor.

### Command Line Options

```
skill-lsp [OPTIONS]

Options:
      --stdio    Use stdio for communication (default)
  -h, --help     Print help
  -V, --version  Print version
```

## Built-in Function Support

### Standard SKILL Functions
All standard SKILL functions including:
- List operations: `car`, `cdr`, `cons`, `list`, `append`, `reverse`, etc.
- Control flow: `if`, `cond`, `case`, `when`, `unless`, `progn`, etc.
- Iteration: `foreach`, `for`, `while`, `mapcar`, etc.
- I/O: `printf`, `sprintf`, `open`, `close`, `read`, `print`, etc.
- Math: `+`, `-`, `*`, `/`, `sin`, `cos`, `sqrt`, etc.
- Type checking: `stringp`, `fixp`, `floatp`, `listp`, etc.

### Allegro SKILL Functions
Cadence Allegro-specific API functions:
- Database: `dbOpenCellViewByType`, `dbCreateRect`, `dbSave`, etc.
- Geometry: `geGetWindowCellView`, `geGetEditCellView`, etc.
- User Interface: `hiCreateForm`, `axlFormCreate`, etc.
- Schematic: `schCreateInst`, `schCreateNet`, etc.

## Architecture

The LSP server is organized into modules:

- `main.rs`: LSP protocol handling and server lifecycle
- `completion.rs`: Code completion and signature help
- `hover.rs`: Hover information and documentation
- `symbols.rs`: Symbol extraction and navigation
- `diagnostics.rs`: Syntax checking and error reporting

## Dependencies

- `tower-lsp`: LSP server framework
- `lsp-types`: LSP protocol types
- `tokio`: Async runtime
- `tree-sitter`: Syntax parsing
- `ropey`: Efficient text editing
- `regex`: Pattern matching

## License

MIT License
