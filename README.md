# pretty-node

a node.js package tree explorer for LLMs (and humans)

> [!NOTE]
> this is the node.js equivalent of [pretty-mod](https://github.com/zzstoatzz/pretty-mod) for python packages.

```bash
# explore package structure
» pnpx pretty-node tree express
📦 express@4.18.2
├── 📜 __all__: express, Router, static, json, urlencoded
├── ⚡ functions: express, static, json, urlencoded
├── 🔷 classes: Router, Application
└── 📦 lib
    ├── ⚡ functions: createApplication
    └── 🔷 classes: Application

# inspect function signatures
» pnpx pretty-node sig express:Router
📎 Router
├── Parameters:
├── options?: RouterOptions
└── Returns: Router
```

## installation

```bash
# use ephemerally with pnpx (recommended)
pnpx pretty-node tree lodash

# or install globally
npm install -g pretty-node
```

## cli

pretty-node includes a command-line interface for shell-based exploration:

> [!IMPORTANT]
> all commands below can be run ephemerally with `pnpx`, e.g. `pnpx pretty-node tree express`

```bash
# explore package structure
pretty-node tree express

# go deeper into the tree with --depth
pretty-node tree express --depth 3

# display function signatures  
pretty-node sig express:Router

# get JSON output for programmatic use
pretty-node tree express -o json | jq '.exports'
pretty-node sig express:Router -o json

# explore packages even without having them installed
pretty-node tree lodash
pretty-node tree @types/node --depth 1

# use --quiet to suppress download messages
pretty-node tree express --quiet

# version specifiers - explore specific versions
pretty-node tree express@4.18.0
pretty-node sig express@4.18.0:Router

# scoped packages
pretty-node tree @types/node
pretty-node sig @types/node:Buffer
```

## customization

pretty-node supports extensive customization through environment variables:

### display characters

```bash
# use ASCII-only mode for terminals without Unicode support
PRETTY_NODE_ASCII=1 pretty-node tree express

# customize individual icons
PRETTY_NODE_MODULE_ICON="[M]" pretty-node tree express
PRETTY_NODE_FUNCTION_ICON="fn" pretty-node tree express
PRETTY_NODE_CLASS_ICON="cls" pretty-node tree express
```

### colors

```bash
# disable colors entirely
PRETTY_NODE_NO_COLOR=1 pretty-node tree express
# or use the standard NO_COLOR environment variable
NO_COLOR=1 pretty-node tree express
```

## development

```bash
gh repo clone <your-org>/pretty-node && cd pretty-node
just --list # see https://github.com/casey/just
```
