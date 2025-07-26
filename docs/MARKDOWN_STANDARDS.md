# Spiral Core Markdown Standards

This document outlines the markdown formatting standards for the Spiral Core project.

## Configuration

The project uses markdownlint with a custom configuration in `.markdownlint.json` and `.markdownlintrc`.

## Disabled Rules

We've intentionally disabled certain markdownlint rules that conflict with our documentation style:

### MD022 - Blanks around headings

**Disabled** - We allow flexible spacing around headings for better visual organization.

### MD032 - Blanks around lists  

**Disabled** - We allow flexible spacing around lists for better readability.

### MD040 - Fenced code language

**Disabled** - We allow code blocks without language specification for simple examples and directory structures.

### MD047 - Single trailing newline

**Disabled** - We don't enforce trailing newlines as it can be inconsistent across editors.

### MD041 - First line in file should be a top-level heading

**Disabled** - Our documentation may start with introductory text before the main heading.

## Enabled Rules with Custom Settings

### MD013 - Line length

- **Max length**: 120 characters
- **Excludes**: Code blocks and tables
- **Reason**: Allows for readable documentation while preventing overly long lines

### MD033 - Inline HTML

- **Allowed elements**: `<br>`, `<details>`, `<summary>`
- **Reason**: These HTML elements enhance documentation readability

## Documentation Style Guidelines

### File Naming

- Use `CLAUDE-{category}-{topic}.md` pattern for project documentation
- Use descriptive kebab-case for general documentation
- Examples: `CLAUDE-colocation-patterns.md`, `API_REFERENCE.md`

### Structure

```
# Main Title

Brief introduction paragraph.

## Section Heading

Content with proper spacing and formatting.

### Subsection

- List items with consistent formatting
- Code examples with appropriate language tags when helpful
- Tables for structured information

## Code Examples

Use language-specific code blocks when the language is relevant:

```rust
fn example() {
    println!("Language specified for syntax highlighting");
}
```

Use plain code blocks for general examples, configs, or directory structures:

```
src/
├── api/
│   ├── mod.rs
│   └── tests/
```

### Lists and Spacing

- Use consistent list formatting
- Allow flexible spacing for visual organization
- Prefer bullet points (-) over asterisks (*) for consistency

### Emphasis

- Use **bold** for important concepts and file names
- Use *italics* for emphasis and variable names
- Use `inline code` for code elements, commands, and file paths

### Links

- Use descriptive link text
- Prefer relative links for internal documentation
- Include link titles when helpful: `[Link Text](url "Title")`

## VS Code Integration

If using VS Code with the markdownlint extension, the `.markdownlint.json` file will automatically configure the linting rules.

### Recommended VS Code Settings

Add to your workspace `.vscode/settings.json`:

```json
{
  "markdownlint.config": {
    "extends": ".markdownlint.json"
  },
  "files.trimTrailingWhitespace": true,
  "files.insertFinalNewline": false
}
```

## Validation

To validate markdown files manually:

```bash
# Install markdownlint-cli globally
npm install -g markdownlint-cli

# Lint all markdown files
markdownlint docs/*.md *.md

# Lint with config
markdownlint -c .markdownlint.json docs/*.md
```

## Documentation Quality Checklist

When creating or updating documentation:

- [ ] File follows naming conventions
- [ ] Structure is clear with appropriate headings
- [ ] Code examples are properly formatted
- [ ] Links are working and descriptive
- [ ] Content is concise but complete
- [ ] No markdownlint violations (with our config)
- [ ] Consistent with project style and tone
