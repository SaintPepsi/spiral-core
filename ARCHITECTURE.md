# Ultimate Simplicity: VS Code Chat Agent

## The Discovery: `code chat --mode='agent'`

We can literally just call VS Code's chat agent directly from the terminal!

## Super Simple Architecture

```
vscode-agent dev "Create a REST API"
         ↓
    code chat --mode='agent' "Create a REST API in Rust"
         ↓
    Parse response → Write to files
         ↓
    cargo check & test
```
