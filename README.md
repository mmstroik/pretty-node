# pretty-node

A Node.js package tree explorer for LLMs (and humans), built with Rust for high performance.

> [!NOTE]
> This is the Node.js equivalent of [pretty-mod](https://github.com/zzstoatzz/pretty-mod) for Python packages.

```bash
# Explore package structure
» pnpx pretty-node tree express
📦 express@4.18.2
├── 📜 __all__: express, Router, static, json, urlencoded
├── ⚡ functions: express, static, json, urlencoded
├── 🔷 classes: Router, Application
└── 📦 lib
    ├── ⚡ functions: createApplication
    └── 🔷 classes: Application

# Inspect function signatures
» pnpx pretty-node sig express:Router
📎 Router
├── Parameters:
├── options?: RouterOptions
└── Returns: Router
```

## Installation

```bash
# Use ephemerally with pnpx (recommended)
pnpx pretty-node tree lodash

# Or install globally
npm install -g pretty-node
```

## CLI Usage

```bash
# Explore package structure
pretty-node tree express
pretty-node tree @types/node --depth 3

# Display function signatures  
pretty-node sig express:Router
pretty-node sig lodash:merge

# Get JSON output for programmatic use
pretty-node tree express -o json | jq '.exports'
pretty-node sig express:Router -o json

# Suppress download messages
pretty-node tree express --quiet

# Version specifiers
pretty-node tree express@4.18.0
pretty-node sig express@4.18.0:Router
```

## Features

- **🚀 High Performance**: Rust-based implementation for fast parsing
- **📦 Auto-download**: Automatically downloads packages from npm
- **🔍 AST Analysis**: Deep parsing of JavaScript/TypeScript files
- **📊 Multiple Formats**: Pretty-printed trees or JSON output
- **🎨 Customizable**: Environment variables for colors and icons
- **💾 Smart Caching**: Uses local node_modules when available

## Customization

### Display Characters

```bash
# ASCII mode for terminals without Unicode
PRETTY_NODE_ASCII=1 pretty-node tree express

# Customize icons
PRETTY_NODE_MODULE_ICON="[M]" pretty-node tree express
PRETTY_NODE_FUNCTION_ICON="fn" pretty-node tree express
```

### Colors

```bash
# Disable colors
PRETTY_NODE_NO_COLOR=1 pretty-node tree express
# or use standard
NO_COLOR=1 pretty-node tree express
```

## Development

```bash
# Clone and build
git clone <repo-url>
cd pretty-node
cargo build --release

# Run tests
cargo test

# Install for development
npm run install
```

## Architecture

- **Rust Core**: High-performance AST parsing and module analysis
- **npm Integration**: Auto-download and package resolution
- **Multi-format Support**: JavaScript, TypeScript, and declaration files
- **CLI Distribution**: Distributed via npm for easy global installation

## Comparison with pretty-mod

| Feature | pretty-node | pretty-mod |
|---------|-------------|------------|
| Language | Node.js/npm | Python/pip |
| Distribution | `pnpx pretty-node` | `uvx pretty-mod` |
| Core Engine | Rust | Rust |
| Package Source | npm registry | PyPI |
| File Types | JS/TS/d.ts | .py |

## License

MIT