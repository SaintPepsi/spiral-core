# 🔧 Pre-Commit Hooks Documentation

## Overview

This project uses Git pre-commit hooks to automatically fix formatting and linting issues before commits, ensuring consistent code quality.

## What Gets Fixed Automatically

### 1. **Rust Formatting** ✅

- Runs `cargo fmt` on all Rust files
- Automatically fixes formatting issues
- Ensures consistent code style

### 2. **Markdown Linting** ✅

- Fixes common Markdown issues (if markdownlint is installed)
- Ensures consistent documentation formatting
- Fixes issues like:
  - Trailing spaces
  - Inconsistent heading styles
  - Missing blank lines
  - Line length issues

### 3. **Compilation Check** ✅

- Verifies code compiles before allowing commit
- Prevents broken code from entering repository

## What Gets Checked (Warnings Only)

### 1. **Clippy Linting** ⚠️

- Runs `cargo clippy` for code quality suggestions
- Non-blocking (won't prevent commit)
- Shows warnings for potential improvements

### 2. **TODO Comments** ⚠️

- Counts TODO comments in staged files
- Non-blocking reminder to address TODOs

### 3. **Debug Prints** ⚠️

- Checks for `dbg!` and `println!` in Rust files
- Non-blocking warning about debug code

## Setup Instructions (One-Time)

### Quick Setup

```bash
# After cloning the repo, run:
./scripts/setup-git-hooks.sh

# Or use Make:
make setup
```

That's it! The hooks are now active and will run on every commit.

**Note**: This is a one-time setup per machine. The hooks will work until you explicitly change them.

## How It Works

When you run `git commit`, the pre-commit hook:

1. **Formats Rust code** - Runs `cargo fmt`
2. **Fixes Markdown** - Runs `markdownlint --fix` on staged .md files
3. **Checks compilation** - Runs `cargo check`
4. **Re-stages files** - Adds formatted files back to commit
5. **Shows summary** - Displays what was fixed/checked

## Bypassing Hooks

If you need to commit without running hooks (not recommended):

```bash
git commit --no-verify -m "Emergency commit"
```

## Example Output

```
🔧 Running pre-commit checks...

📐 Checking Rust formatting...
  Fixing Rust formatting...
  ✅ Rust formatting fixed

📝 Checking Markdown files...
  Fixing: docs/README.md
  ✅ Markdown files checked

🔍 Running Clippy linting...
  ✅ No Clippy warnings

🔎 Checking for common issues...
  ⚠️  Found 3 TODO comments in staged files
  ✅ No debug prints found

🔨 Running quick compilation check...
  ✅ Code compiles successfully

📦 Adding formatted files to commit...
✅ Files formatted and added to commit

✅ Pre-commit checks passed!

📋 Files being committed:
M       src/main.rs
M       docs/README.md
```

## Troubleshooting

### Hook Not Running

```bash
# Verify hooks are configured
git config core.hooksPath
# Should output: .githooks

# If not, run setup again
./scripts/setup-git-hooks.sh
```

### Markdownlint Not Working

```bash
# Check if markdownlint is available
which markdownlint || which npx

# Install if needed
npm install -g markdownlint-cli
```

### Hook Permissions Error

```bash
# Fix permissions
chmod +x .githooks/pre-commit
```

## Benefits

1. **Consistent Formatting** - All code follows same style
2. **Fewer CI Failures** - Catches issues before push
3. **Cleaner History** - No "fix formatting" commits
4. **Early Error Detection** - Compilation checked before commit
5. **Documentation Quality** - Markdown stays clean and readable

## Customization

To modify the pre-commit behavior, edit `.githooks/pre-commit`.

Common customizations:

- Add more file type checks
- Include additional linters
- Add project-specific validations
- Modify warning/error thresholds

## Integration with CI/CD

The pre-commit hooks complement but don't replace CI/CD checks:

- **Pre-commit**: Fast, local, automatic fixes
- **CI/CD**: Comprehensive, all platforms, final validation

Both work together to maintain code quality.
