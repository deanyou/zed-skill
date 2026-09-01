# Zed Cadence SKILL

[English](README.md) | [简体中文](README.zh-CN.md)

Zed 编辑器的 **Cadence SKILL** 语言扩展 —— SKILL 是 Cadence Virtuoso EDA 工具的 Lisp 系脚本语言。

| | |
|---|---|
| 文件类型 | `.il` `.ils` `.skill` |
| 扩展 | [zed-skill/](zed-skill/) |
| 语言服务器 | [skill-lsp/](skill-lsp/)（Rust / tower-lsp） |
| 语法（Grammar） | [tree-sitter-skill/](tree-sitter-skill/) |

## 功能特性

- 语法高亮：特殊形式（`defun` `let` `foreach` …）、内建函数（`car` `mapcar` `printf`、`db*`/`ge*`/`hi*` API）、引号语法（`'` `` ` `` `,` `,@`）、块注释、`?keywords`、点对（dotted pairs）
- 彩虹括号与配对括号高亮，支持 `()` `[]` `{}`
- 代码补全：**9,600+ 官方 API 函数**（从 Cadence IC23.1 官方参考文档提取）、100+ 核心内建函数、用户自定义函数、代码片段
- 悬停文档：含官方 API 签名与描述
- 跳转定义、查找引用、文档符号、工作区符号
- 重命名符号、文档高亮、签名帮助、格式化
- 括号内自动缩进、代码折叠、大纲（outline）、扩选
- 诊断：括号不配对、引号不匹配

## 快速安装

**方式 A —— 预编译二进制**（推荐）：从 [GitHub Releases](https://github.com/deanyou/zed-skill/releases) 下载对应平台的 `skill-lsp-*.tar.gz` / `.zip`，解压后放入 `PATH`。

**方式 B —— 源码编译**（需先安装 [Rust](https://rustup.rs)）：

```bash
# 1. 克隆仓库
git clone https://github.com/deanyou/zed-skill.git
cd zed-skill

# 2. 编译并安装语言服务器
cargo install --path skill-lsp
# macOS / Linux → ~/.cargo/bin/skill-lsp
# Windows       → %USERPROFILE%\.cargo\bin\skill-lsp.exe
```

**3. 在 Zed 中安装扩展**
Zed → Extensions（macOS: `cmd-shift-x`，其他平台: `ctrl-shift-x`）→ **Install Dev Extension** → 选择本仓库的 `zed-skill/` 目录。

首次会自动编译扩展，完成后打开任意 `.il` 文件即可。

> 正式发布中：[zed-industries/extensions PR](https://github.com/zed-industries/extensions/pull/7377)。合并后可直接在 Zed 的 Extensions 面板搜索 "skill" 安装。

## 配置

settings.json 文件位置：

| 系统 | 路径 |
|---|---|
| macOS / Linux | `~/.config/zed/settings.json` |
| Windows | `%APPDATA%\Zed\settings.json` |

若 `skill-lsp` 不在 `PATH` 中，需指定二进制路径：

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
// Windows —— JSON 中路径分隔符用 \\ 或 /
{
  "lsp": {
    "skill-lsp": {
      "binary": { "path": "C:\\Users\\you\\.cargo\\bin\\skill-lsp.exe" }
    }
  }
}
```

可选 —— 彩虹括号（层级颜色跟随主题 `accents`）：

```json
{
  "colorize_brackets": true,
  "theme_overrides": {
    "你的主题名": {
      "editor.document_highlight.bracket_background": "#fabd2f99"
    }
  }
}
```

> `theme_overrides` 下的键必须与当前激活的主题名完全一致。此配置块三平台通用。

## 常见问题

- **`Failed to compile grammar 'skill'`**：Zed 需下载 WASI SDK 编译语法。若下载失败（网络原因），可在终端设置已缓存的 SDK 后启动 Zed：
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
- **无补全/悬停**：确认 `skill-lsp` 在 `PATH` 中（`which skill-lsp` / `Get-Command skill-lsp`），或按上文设置 `lsp.skill-lsp.binary.path`。安装二进制后需重启 Zed。

## 许可证

[MIT](LICENSE)
