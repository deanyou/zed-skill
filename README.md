# Zed Cadence SKILL

[English](README.md) | [简体中文](README.zh-CN.md)

Zed editor extension for **Cadence SKILL** — the Lisp-based scripting language of Cadence Virtuoso EDA tools.

| | |
|---|---|
| File types | `.il` `.ils` `.skill` |
| Extension | [zed-skill/](zed-skill/) |
| Language server | [skill-lsp/](skill-lsp/) (Rust / tower-lsp) |
| Grammar | [tree-sitter-skill/](tree-sitter-skill/) |

## Features

- Syntax highlighting: special forms (`defun` `let` `foreach` …), built-ins (`car` `mapcar` `printf`, `db*`/`ge*`/`hi*` APIs), quoting (`'` `` ` `` `,` `,@`), block comments, `?keywords`, dotted pairs
- Rainbow brackets & matched-bracket highlight for `()` `[]` `{}`
- Completion: **9,600+ official API functions** (extracted from Cadence IC23.1 reference docs), 100+ core built-ins, user-defined functions, snippets
- Hover docs with official API signatures & descriptions
- Go to definition, find references, document symbols, workspace symbols
- Rename symbol, document highlight, signature help, formatting
- Auto-indent inside brackets, code folding, outline, expand selection
- Diagnostics: unbalanced parens, unmatched quotes

## Quick Install

**Option A — prebuilt binary** (recommended): download `skill-lsp-*.tar.gz` / `.zip` for your platform from [GitHub Releases](https://github.com/deanyou/zed-skill/releases), extract, and put it on `PATH`.

**Option B — build from source** (requires [Rust](https://rustup.rs)):

```bash
# 1. Clone
git clone https://github.com/deanyou/zed-skill.git
cd zed-skill

# 2. Build & install the LSP server
cargo install --path skill-lsp
# macOS / Linux → ~/.cargo/bin/skill-lsp
# Windows      → %USERPROFILE%\.cargo\bin\skill-lsp.exe
```

**3. Install the extension in Zed**
Zed → Extensions (`cmd-shift-x` on macOS, `ctrl-shift-x` elsewhere) → **Install Dev Extension** → select the `zed-skill/` directory of this repo.

After Zed compiles the extension (first time only), open any `.il` file.

> Pending official publishing: [zed-industries/extensions PR](https://github.com/zed-industries/extensions/pull/7377). Once merged, install directly from Zed's Extensions panel by searching "skill".

## Setup

Settings file location:

| OS | Path |
|---|---|
| macOS / Linux | `~/.config/zed/settings.json` |
| Windows | `%APPDATA%\Zed\settings.json` |

If `skill-lsp` is not in `PATH`, point Zed to it:

```jsonc
// macOS / Linux
{
  "lsp": {
    "skill-lsp": {
      "binary": { "path": "/usr/local/bin/skill-lsp" }
    }
  }
}
```

```jsonc
// Windows — use \\ or / as path separator in JSON
{
  "lsp": {
    "skill-lsp": {
      "binary": { "path": "C:\\Users\\you\\.cargo\\bin\\skill-lsp.exe" }
    }
  }
}
```

Optional — rainbow brackets (colors follow the theme `accents`):

```json
{
  "colorize_brackets": true,
  "theme_overrides": {
    "Your Theme Name": {
      "editor.document_highlight.bracket_background": "#fabd2f99"
    }
  }
}
```

> The key under `theme_overrides` must exactly match your active theme name. This block is identical on all platforms.

## Troubleshooting

- **`Failed to compile grammar 'skill'`**: Zed downloads the WASI SDK to compile the grammar. If the download fails (network), launch Zed from a terminal with the bundled SDK:
  ```bash
  # macOS
  export WASI_SDK_PATH="$HOME/Library/Application Support/Zed/extensions/build/wasi-sdk"
  open -a Zed
  ```
  ```bash
  # Linux
  export WASI_SDK_PATH="$HOME/.local/share/zed/extensions/build/wasi-sdk"
  zed &
  ```
  ```powershell
  # Windows (PowerShell)
  $env:WASI_SDK_PATH = "$env:LOCALAPPDATA\Zed\extensions\build\wasi-sdk"
  zed
  ```
- **No completion/hover**: check that `skill-lsp` resolves in `PATH` (`which skill-lsp` / `Get-Command skill-lsp`), or set `lsp.skill-lsp.binary.path` as above. Restart Zed after installing the binary.

## Changelog

### 0.2.1

- Fixed hover/signature lookup for mixed-case API functions (e.g. `getShellEnvVar`)
- UTF-8 position encoding — correct offsets on lines with CJK comments
- Signature help: innermost call detection, space-separated argument index
- Diagnostics: no false positives from `/* */` comments, escaped quotes, or cross-line string state
- Auto-indent & code folding now actually active (fixed Zed `indents.scm` captures)
- Grammar revision pinned to a full commit SHA

### 0.2.0

- **9,600+ official API functions** in completion & hover (generated from Cadence IC23.1 `.fnd` reference docs)
- Rename symbol, workspace symbols, document highlight, signature help
- Auto-indent, code folding, outline, expand selection; extension icon
- CI + prebuilt binaries on GitHub Releases

## License

[MIT](LICENSE)
