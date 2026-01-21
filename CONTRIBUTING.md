# Contributing to Stylus Trace Studio

Thank you for your interest in contributing to Stylus Trace Studio! This document provides guidelines and instructions for contributing to the project.

---

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [How to Contribute](#how-to-contribute)
- [Coding Standards](#coding-standards)
- [Testing Guidelines](#testing-guidelines)
- [Commit Message Guidelines](#commit-message-guidelines)
- [Pull Request Process](#pull-request-process)
- [Issue Reporting](#issue-reporting)
- [Community](#community)

---

## Code of Conduct

We are committed to providing a welcoming and inclusive environment. All contributors are expected to:

- Be respectful and considerate
- Accept constructive criticism gracefully
- Focus on what's best for the community
- Show empathy towards other community members

**Report any unacceptable behavior** to the project maintainers.

---

## Getting Started

### Prerequisites

Before you begin, ensure you have:

- **Rust** 1.70 or higher: `rustup update`
- **Docker** for running the Nitro dev node
- **Git** for version control
- **Foundry** for testing: `curl -L https://foundry.paradigm.xyz | bash && foundryup`
- **Cargo Stylus**: `cargo install --force cargo-stylus`

### Quick Setup

```bash
# Fork the repository on GitHub
# Clone your fork
git clone https://github.com/Timi16/stylus-trace.git
cd stylus-trace-studio

# Add upstream remote
git remote add upstream https://github.com/Timi16/stylus-trace.git

# Install dependencies and build
cargo build

# Run tests
cargo test

# Start Nitro dev node
cd ../nitro-devnode
./run-dev-node.sh
```

---

## 🏗️ Development Setup

### Building the Project

```bash
# Debug build (faster compilation, slower execution)
cargo build

# Release build (slower compilation, optimized execution)
cargo build --release

# Run the CLI
./target/debug/stylus-trace --help
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_parse_trace

# Run with output
cargo test -- --nocapture

# Run tests with logging
RUST_LOG=debug cargo test
```

### Linting and Formatting

```bash
# Format code
cargo fmt

# Check formatting without modifying files
cargo fmt -- --check

# Run clippy (linter)
cargo clippy

# Run clippy with warnings as errors (CI standard)
cargo clippy -- -D warnings

# Fix clippy warnings automatically (where possible)
cargo clippy --fix
```

### Running Examples

```bash
# Run the end-to-end smoke test
cargo run --example smoke_test

# Run with a specific transaction
cargo run -- capture \
  --rpc http://localhost:8547 \
  --tx 0xabc123... \
  --output test-profile.json \
  --flamegraph test.svg
```

---

## 📁 Project Structure

```
stylus-trace-studio/
├── src/
│   ├── main.rs                 # CLI entry point
│   ├── lib.rs                  # Library root
│   ├── aggregator/             # Stack building and metrics
│   │   ├── mod.rs
│   │   ├── stack_builder.rs    # Collapsed stack generation
│   │   └── metrics.rs          # Hot path calculation
│   ├── commands/               # CLI commands
│   │   ├── mod.rs
│   │   └── capture.rs          # Capture command logic
│   ├── flamegraph/             # SVG generation
│   │   ├── mod.rs
│   │   └── generator.rs        # Flamegraph rendering
│   ├── output/                 # File I/O
│   │   ├── mod.rs
│   │   ├── json.rs             # JSON profile writer
│   │   └── svg.rs              # SVG file writer
│   ├── parser/                 # Trace parsing
│   │   ├── mod.rs
│   │   ├── stylus_trace.rs     # Main parser
│   │   ├── hostio.rs           # HostIO event extraction
│   │   └── schema.rs           # Output schema types
│   ├── rpc/                    # RPC client
│   │   ├── mod.rs
│   │   ├── client.rs           # HTTP client
│   │   └── types.rs            # RPC types
│   └── utils/                  # Utilities
│       ├── mod.rs
│       ├── config.rs           # Configuration constants
│       └── error.rs            # Error types
├── tests/                      # Integration tests
│   └── integration_test.rs
├── examples/                   # Example usage
│   └── smoke_test.rs
├── Cargo.toml                  # Rust dependencies
├── README.md                   # Project documentation
└── CONTRIBUTING.md             # This file
```

### Module Responsibilities

- **`aggregator`** - Converts parsed traces into collapsed stacks and calculates metrics
- **`commands`** - Implements CLI command logic (capture, validate, schema, version)
- **`flamegraph`** - Generates SVG flamegraphs from collapsed stacks
- **`output`** - Handles writing JSON profiles and SVG files
- **`parser`** - Parses raw trace JSON into structured data
- **`rpc`** - Communicates with Arbitrum Nitro RPC endpoints
- **`utils`** - Shared utilities, configuration, and error handling

---

## 🤝 How to Contribute

### Types of Contributions

We welcome various types of contributions:

1. **Bug Fixes** - Fix issues reported in GitHub Issues
2. **Feature Implementations** - Implement features from the roadmap
3. **Documentation** - Improve README, docs, or code comments
4. **Tests** - Add or improve test coverage
5. **Examples** - Create example workflows or use cases
6. **Performance** - Optimize existing code
7. **Tooling** - Improve CI/CD, build scripts, or dev tools

### Finding Issues to Work On

Look for issues labeled:
- `good-first-issue` - Good for newcomers
- `help-wanted` - We need community help
- `bug` - Bug reports needing fixes
- `enhancement` - New features or improvements
- `documentation` - Documentation improvements

### Before You Start

1. **Check existing issues** - Ensure the issue/feature isn't already being worked on
2. **Open a discussion** - For major changes, open an issue first to discuss the approach
3. **Comment on the issue** - Let others know you're working on it
4. **Keep it focused** - One pull request per feature/fix

---

## 💻 Coding Standards

### Rust Style Guide

We follow the official [Rust Style Guide](https://doc.rust-lang.org/style-guide/). Key points:

**Formatting:**
```rust
// Use rustfmt default settings
cargo fmt

// Lines should be ≤100 characters where practical
```

**Naming:**
```rust
// Types: PascalCase
struct ParsedTrace { ... }
enum FlamegraphPalette { ... }

// Functions and variables: snake_case
fn build_collapsed_stacks() { ... }
let hot_paths = vec![];

// Constants: SCREAMING_SNAKE_CASE
const MAX_TRACE_SIZE: usize = 10_000_000;
```

**Error Handling:**
```rust
// Use Result<T, E> for recoverable errors
pub fn parse_trace(tx_hash: &str, data: &Value) -> Result<ParsedTrace, ParseError> {
    // ...
}

// Document error conditions in function docs
/// # Errors
/// Returns `ParseError::InvalidFormat` if the trace structure is invalid.
```

**Documentation:**
```rust
// Public items MUST have doc comments
/// Parse raw trace JSON from stylusTracer
///
/// # Arguments
/// * `tx_hash` - Transaction hash being profiled
/// * `raw_trace` - Raw JSON from debug_traceTransaction
///
/// # Returns
/// Parsed trace data ready for aggregation
///
/// # Errors
/// * `ParseError::JsonError` - Invalid JSON structure
/// * `ParseError::InvalidFormat` - Missing required fields
pub fn parse_trace(tx_hash: &str, raw_trace: &Value) -> Result<ParsedTrace, ParseError> {
    // Implementation
}
```

**Testing:**
```rust
// Unit tests in the same file
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_trace() {
        // Arrange
        let raw_trace = json!({ "gas": 100, "structLogs": [] });
        
        // Act
        let result = parse_trace("0xabc", &raw_trace);
        
        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().total_gas_used, 100);
    }
}
```

### Code Quality Checklist

Before submitting, ensure:

-  Code compiles without warnings: `cargo build`
-  All tests pass: `cargo test`
-  Code is formatted: `cargo fmt`
-  No clippy warnings: `cargo clippy -- -D warnings`
-  Public items have documentation
-  New features have tests
-  Complex logic has comments

---

## Testing Guidelines

### Test Categories

**Unit Tests** - Test individual functions/modules
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collapsed_stack_to_line() {
        let stack = CollapsedStack::new("main;execute".to_string(), 1000);
        assert_eq!(stack.to_line(), "main;execute 1000");
    }
}
```

**Integration Tests** - Test component interactions
```rust
// tests/integration_test.rs
#[test]
fn test_full_capture_workflow() {
    // Start with raw trace
    // Parse it
    // Build stacks
    // Generate flamegraph
    // Verify output
}
```


### Writing Good Tests

```rust
#[test]
fn test_descriptive_name() {
    // Arrange - Set up test data
    let input = create_test_input();
    
    // Act - Execute the code being tested
    let result = function_under_test(input);
    
    // Assert - Verify the outcome
    assert_eq!(result, expected_value);
}
```

### Test Coverage

We aim for:
- **≥80% line coverage** for core modules
- **100% coverage** for public APIs
- **Edge case testing** for error conditions

```bash
# Install tarpaulin for coverage
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html
```

---

## Commit Message Guidelines

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification.

### Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation changes
- `style` - Code style changes (formatting, no logic change)
- `refactor` - Code refactoring
- `perf` - Performance improvements
- `test` - Adding or updating tests
- `chore` - Maintenance tasks
- `ci` - CI/CD changes

### Examples

```bash
# Feature
git commit -m "feat(flamegraph): add custom color palette support"

# Bug fix
git commit -m "fix(parser): handle missing gas field in trace"

# Documentation
git commit -m "docs(readme): add troubleshooting section"

# With body
git commit -m "refactor(rpc): simplify client connection logic

- Remove duplicate code
- Extract connection setup to helper
- Add connection timeout handling"

# Breaking change
git commit -m "feat(cli)!: change default output filename

BREAKING CHANGE: Default output is now 'profile.json' instead of 'output.json'"
```

### Best Practices

- Use present tense: "add feature" not "added feature"
- Use imperative mood: "move cursor to..." not "moves cursor to..."
- Keep subject line ≤50 characters
- Capitalize the subject line
- Don't end with a period
- Body should explain *what* and *why*, not *how*

---

## Pull Request Process

### Before Opening a PR

1. **Fork the repository**
2. **Create a feature branch** from `main`:
   ```bash
   git checkout -b feat/your-feature-name
   ```
3. **Make your changes**
4. **Add tests** for new functionality
5. **Run the quality checks**:
   ```bash
   cargo test
   cargo fmt -- --check
   cargo clippy -- -D warnings
   ```
6. **Commit your changes** using conventional commits
7. **Push to your fork**:
   ```bash
   git push origin feat/your-feature-name
   ```

### Opening a PR

1. **Go to the original repository** on GitHub
2. **Click "New Pull Request"**
3. **Choose your branch** from your fork
4. **Fill out the PR template**:

```markdown
## Description
Brief description of what this PR does.

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Related Issues
Closes #123

## Testing
Describe how you tested your changes.

## Checklist
- [ ] Tests pass locally
- [ ] Code is formatted
- [ ] No clippy warnings
- [ ] Documentation updated
- [ ] CHANGELOG.md updated (for user-facing changes)
```

5. **Request review** from maintainers

### PR Review Process

1. **Automated checks** must pass (CI/CD)
2. **At least one maintainer** must approve
3. **Address feedback** by pushing new commits
4. **Squash and merge** when approved

### After Your PR is Merged

1. **Delete your branch**:
   ```bash
   git branch -d feat/your-feature-name
   git push origin --delete feat/your-feature-name
   ```

2. **Update your fork**:
   ```bash
   git checkout main
   git pull upstream main
   git push origin main
   ```

---

## Issue Reporting

### Before Reporting

1. **Search existing issues** - Your issue might already be reported
2. **Update to latest version** - The bug might be fixed
3. **Test with minimal example** - Isolate the problem

### Bug Report Template

```markdown
**Describe the bug**
A clear description of what the bug is.

**To Reproduce**
Steps to reproduce:
1. Run command '...'
2. With input '...'
3. See error

**Expected behavior**
What you expected to happen.

**Actual behavior**
What actually happened.

**Environment:**
- OS: [e.g., macOS 13.0]
- Rust version: [e.g., 1.70.0]
- stylus-trace version: [e.g., 0.1.0]
- Nitro node version: [e.g., v3.0.1]

**Additional context**
- Transaction hash (if relevant)
- Error messages (full output)
- Logs (use `RUST_LOG=debug`)
```

### Feature Request Template

```markdown
**Is your feature request related to a problem?**
A clear description of the problem.

**Describe the solution you'd like**
What you want to happen.

**Describe alternatives you've considered**
Other solutions you've thought about.

**Additional context**
Examples, mockups, or references.
```

---

## Community

### Getting Help

- **GitHub Discussions** - Ask questions, share ideas
- **GitHub Issues** - Report bugs, request features
- **Discord** - Real-time chat (link in README)
- **Twitter** - Updates and announcements

### Communication Guidelines

- **Be clear and concise**
- **Provide context** (what you tried, what didn't work)
- **Share code** using code blocks or gists
- **Be patient** - Maintainers are volunteers
- **Be respectful** - Everyone is learning

### Recognition

Contributors are recognized in:
- `CONTRIBUTORS.md` file
- Release notes
- Project README

---

## 📚 Additional Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Arbitrum Stylus Docs](https://docs.arbitrum.io/stylus/stylus-gentle-introduction)
- [Flamegraph Visualization](https://www.brendangregg.com/flamegraphs.html)

---

## 🙏 Thank You!

Your contributions make this project better for everyone in the Arbitrum ecosystem. We appreciate your time and effort!

**Questions?** Open a discussion or reach out to the maintainers.

---

**Happy Contributing!**