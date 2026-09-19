# norma Quick Start

Get **norma** running in 5 minutes.

## 1️⃣ Prerequisites

- Rust 1.70+ — [Install rustup](https://rustup.rs/)
- Git
- SQLite 3.0+ (usually pre-installed)

## 2️⃣ Build

```bash
cd norma
cargo build --release
```

Takes ~2-3 minutes on first build.

## 3️⃣ Run the MCP Server

```bash
./target/release/norma
```

Server is ready when you see:
```
Pattern store initialized with SQLite
MCP transport established, waiting for connections...
```

## 4️⃣ Test with MCP Inspector

In a new terminal:

```bash
# Install MCP Inspector (global)
npm install -g @modelcontextprotocol/inspector

# Connect to norma
mcp-inspector stdio /path/to/norma/target/release/norma
```

Opens a browser interface. Try calling:

```
get_pattern_checklist(language: "java")
```

Expected response:
```json
[
  {
    "id": "java-static-import",
    "name": "Avoid Static Imports",
    "description": "Discourage wildcard static imports in Java",
    "severity": "warning",
    "enabled": true
  },
  ...
]
```

## 5️⃣ Use with Claude Code

1. Copy `.claude.example.json` to `.claude.json` in your project root
2. Update the path to your norma binary
3. Restart Claude Code
4. Use in prompts:

```
@tool validate_pattern_compliance
code: """
public class MyClass {
    public static void main(String[] args) {
        System.out.println("Hello");
    }
}
"""
language: "java"
```

## 🎯 Next Steps

See **DEVELOPMENT.md** for:
- AST-grep integration
- Custom pattern definition
- Pre-commit hook setup
- Test suite expansion

## 🆘 Troubleshooting

### Build fails with "mcpkit not found"

```bash
# Update dependencies
cargo update
cargo build --release
```

### Rust not found

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### SQLite error

```bash
# Ensure SQLite is installed
# macOS:
brew install sqlite3

# Linux (Ubuntu):
sudo apt-get install sqlite3 libsqlite3-dev

# Windows: Usually included, verify:
sqlite3 --version
```

---

**Ready to code?** Jump into `DEVELOPMENT.md` for the roadmap.
