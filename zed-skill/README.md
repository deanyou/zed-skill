# Zed SKILL Extension

A Zed editor extension that provides comprehensive language support for Cadence SKILL programming language.

## Features

Based on the [herbertagosto.skillherb](https://marketplace.visualstudio.com/items?itemName=herbertagosto.skillherb) VS Code extension, this Zed extension provides:

### Syntax Highlighting
- Full syntax highlighting for SKILL language constructs
- Support for comments, strings, numbers, constants, keywords, operators
- Highlighting for built-in functions and special forms

### Code Navigation
- **Go to Definition** (F12): Jump to function definitions
- **Find References**: Find all references to a symbol
- **Document Symbols**: Browse file structure in the outline panel

### Code Completion
- **Built-in Functions**: All standard SKILL functions with documentation
- **Allegro SKILL Functions**: Support for Cadence Allegro-specific API
- **User-defined Functions**: Auto-completion for your own functions
- **Signature Help**: Parameter hints while typing

### Hover Information
- Documentation on built-in functions
- User-defined function documentation from doc comments
- Parameter and return type information

### Diagnostics
- Syntax error detection
- Unbalanced parenthesis checking
- Common misspelling detection
- Deprecated function warnings

### Code Snippets
Quick templates for common constructs:
- `region` - Foldable code regions
- `defun` - Function definitions
- `procedure` - Procedure definitions
- `let` - Local variable bindings
- `when`/`unless` - Conditional execution
- `case` - Case dispatch
- `if` - Conditional expressions
- `foreach`/`for` - Loops
- `mapcar` - List mapping
- `lambda` - Anonymous functions
- `doc` - Documentation comments

### Code Folding
- Region-based folding with `;region` and `;endregion` markers
- Automatic folding for code blocks

## Installation

### Prerequisites

1. Install [Rust](https://www.rust-lang.org/tools/install) via rustup
2. Install the WASI SDK target (Zed will do this automatically)

### Building from Source

1. Clone this repository:
```bash
git clone <repository-url>
cd zed-skill
```

2. Build the LSP server:
```bash
cd skill-lsp
cargo build --release
```

3. Install the extension as a dev extension:
   - Open Zed
   - Go to Extensions (`cmd-shift-x` or `ctrl-shift-x`)
   - Click "Install Dev Extension"
   - Select the `zed-skill` directory

### Installing the LSP Server

The LSP server binary needs to be in your PATH or configured in settings:

```bash
# Copy to a directory in your PATH
cp skill-lsp/target/release/skill-lsp /usr/local/bin/
```

Or configure the path in Zed settings:

```json
{
  "lsp": {
    "skill-lsp": {
      "binary": {
        "path": "/path/to/skill-lsp"
      }
    }
  }
}
```

## Configuration

The extension can be configured in Zed's settings:

```json
{
  "languages": {
    "Skill": {
      "tab_size": 4,
      "hard_tabs": false
    }
  },
  "lsp": {
    "skill-lsp": {
      "settings": {
        "enableAllegroFunctions": true,
        "diagnosticEnabled": true
      }
    }
  }
}
```

## Language Configuration

File extensions associated with SKILL:
- `.il` - SKILL source files
- `.ils` - SKILL source files (alternative)
- `.skill` - SKILL source files

## SKILL Language Overview

SKILL is a Lisp-based programming language used in Cadence Design Systems EDA tools. It's primarily used for:

- Customizing Cadence tools
- Automating design tasks
- Creating PCells (parameterized cells)
- Design rule checking
- Layout and schematic automation

### Example SKILL Code

```skill
;;;
;;; @function createRectangle
;;; @description Creates a rectangle in the current cell view
;;; @param layer Layer name
;;; @param purpose Purpose name
;;; @param bBox Bounding box list
;;; @return Database ID of the created rectangle
;;;
(defun createRectangle (layer purpose bBox)
  (dbCreateRect geGetEditCellView() layer purpose bBox))

;;;
;;; @function createCellView
;;; @description Opens a cell view for editing
;;; @param lib Library name
;;; @param cell Cell name
;;; @param view View name
;;; @return Cell view ID
;;;
(defun createCellView (lib cell view)
  (dbOpenCellViewByType lib cell view "maskLayout" "a"))
```

## Building the LSP Server Separately

The LSP server is a separate Rust project that can be built and distributed independently:

```bash
cd skill-lsp
cargo build --release
```

The resulting binary at `target/release/skill-lsp` can be installed system-wide or configured in Zed settings.

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

MIT License

## Credits

This extension is inspired by the [Skill+](https://marketplace.visualstudio.com/items?itemName=herbertagosto.skillherb) VS Code extension by Herbert Agosto.
