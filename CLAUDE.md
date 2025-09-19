# CLAUDE.md

## 🚨 CRITICAL: Implementation Rules

**ONLY implement what the user explicitly requests. NOTHING MORE.**
- If you want to add extra features, ASK THE USER FIRST
- Do NOT add port handling, error cases, or optimizations unless requested
- Stick to the exact requirements given

## 🚨 CRITICAL: Type Safety

**Use newtype pattern for meaningful primitive types**
- Wrap primitive types that have domain meaning (e.g., `struct Host(String)`)
- This prevents mixing incompatible values and adds semantic clarity
- Implement necessary traits (Deref, AsRef, Display) for ergonomic usage
- Examples: Host, Port, Path, UserId, Email, etc.

## 🚨 CRITICAL: Test-Driven Development (TDD)

**THIS PROJECT STRICTLY FOLLOWS TDD. NO EXCEPTIONS.**

1. **ALWAYS write tests FIRST** before ANY implementation
2. **Follow TDD cycle**: Red → Green → Refactor
3. **NEVER let test coverage drop below 50%** (mandatory)
4. **RECOMMENDED: Maintain test coverage above 80%** for production quality
5. **Check coverage after EVERY change**: `cargo tarpaulin --ignore-tests`

## Project Overview

Rust-based web browser implementation with strict TDD practices.

## 🔥 MANDATORY Quality Checklist

**Run ALL after EVERY implementation:**

```bash
cargo build
cargo test --all-features
cargo clippy --all-features --tests -- -W clippy::all -D warnings
cargo tarpaulin --ignore-tests  # MUST stay > 50%
```

**If ANY fail, FIX immediately!**

## Essential Commands

### Testing
```bash
cargo test                       # Run all tests frequently
cargo test -- --nocapture        # With output
cargo tarpaulin --ignore-tests   # Coverage check
```

### Build
```bash
cargo build           # Debug build
cargo run            # Build and run
```

### Quality
```bash
cargo fmt                                    # Format
cargo clippy -- -W clippy::all -D warnings  # Lint with errors
```

## 📝 Documentation Rules

**Write natural, purposeful docs:**

- Document public functions with `///` when purpose isn't obvious
- Skip trivial getters/setters
- Focus on "why" not "what"
- Include doc tests for complex APIs

Good:
```rust
/// Parses URLs according to RFC 3986.
/// Uses "://" to distinguish scheme from host.
```

## TDD Workflow

1. Write failing tests defining expected behavior
2. Run `cargo test` to confirm failure
3. Implement minimal code to pass
4. Run quality checklist
5. **REFACTOR**: Question names, logic, structure, performance
6. Add edge case tests
7. Run checklist again

## Project Structure

Standard Rust/Cargo with TDD:
- `src/main.rs` - Entry point
- `src/lib.rs` - Library root
- Tests in `#[cfg(test)]` modules or alongside code
- `Cargo.toml` - Dependencies

## Dependencies

```toml
[dev-dependencies]
cargo-tarpaulin = "*"  # Test coverage

[dependencies]
# Add as needed with tests
```

Preferred crates:
- HTTP: `reqwest`, `hyper`
- HTML: `html5ever`, `scraper`
- GUI: `egui`, `gtk-rs`