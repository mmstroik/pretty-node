# pretty-node

A Rust-based CLI tool for exploring Node.js package trees and extracting function signatures, built as a Node.js equivalent to pretty-mod.

## Getting Oriented

- Read `README.md`, `Cargo.toml`, and test files in `tests/`
- This project mirrors the architecture and functionality of `../pretty-mod`
- **Reference `../pretty-mod` files regularly when stuck or need implementation guidance**

## Build and Test

```bash
# Build the project
cargo build --release

# Run tests
cargo test

# Run CLI locally
cargo run -- tree express
cargo run -- signature express:Router
```

## Current Status

### ✅ Completed Features
- Basic CLI with tree and signature commands
- Package auto-download from npm registry
- AST parsing with swc for JavaScript/TypeScript
- Import chain resolution for better symbol tracking
- Smart signatures for decorator patterns (@flow, @task)
- Recursive signature search across module tree
- Factory pattern for output formatters (Pretty/JSON)
- KeyboardInterrupt handling with proper exit codes
- Comprehensive test suite (30 tests passing)

### 🔄 Current Issues

1. **Semantic analyzer compilation errors** (in progress)
   - Incorrect Visit trait method names in `src/parser/semantic_analyzer.rs:144`
   - Need to fix: `visit_function_decl` → correct swc method name
   - Need to fix: `visit_export_named_decl` → correct swc method name
   - Missing fields in ClassInfo initialization
   - Parameter type mismatches for constructor params

2. **TypeScript declaration file parsing** (pending)
   - Need enhanced parsing of .d.ts files
   - Better type extraction from TypeScript annotations

### 📋 Next Steps

1. **Fix semantic analyzer** (high priority)
   - Research correct swc_ecma_visit::Visit trait method names
   - Reference `../pretty-mod/src/semantic.rs` for patterns
   - Fix compilation errors and re-enable in `src/parser/mod.rs`

2. **Enhance TypeScript support**
   - Improve .d.ts file parsing
   - Better type annotation extraction
   - Reference `../pretty-mod/src/parser/typescript.rs`

3. **Advanced signature extraction**
   - Class method vs function distinction
   - Better parameter type resolution
   - JSDoc comment extraction
   - Reference `../pretty-mod/src/parser/signature.rs`

## Architecture Notes

- **CLI**: `src/main.rs` - main entry point with clap
- **Explorer**: `src/explorer.rs` - core module tree exploration
- **Parsers**: `src/parser/` - AST parsing, signature extraction
- **Output**: `src/output_format.rs` - factory pattern for formatters
- **Tests**: `tests/` - comprehensive CLI and functionality tests

## Reference Files in pretty-mod

When implementing features or fixing issues, regularly check:
- `../pretty-mod/src/semantic.rs` - for semantic analysis patterns
- `../pretty-mod/src/parser/` - for parsing strategies  
- `../pretty-mod/tests/` - for test patterns and coverage
- `../pretty-mod/src/explorer.rs` - for exploration logic
- `../pretty-mod/CLAUDE.md` - for general project guidance

## Debug Mode

Set `PRETTY_NODE_DEBUG=1` to enable verbose logging during development.