# Web Browser Engineering in Rust

A Rust implementation of the [Web Browser Engineering](https://browser.engineering/) book.

## Overview

This project is a learning exercise to implement a web browser from scratch using Rust, following the concepts and techniques from the "Web Browser Engineering" book by Pavel Panchekha and Chris Harrelson.

## Features

Currently implemented:
- URL parsing (scheme, host, port, path)
- Basic TCP socket connections
- Strong type safety with newtype patterns

## Development

This project follows Test-Driven Development (TDD) with strict quality requirements:
- Minimum 50% test coverage (currently at 89%!)
- All tests must pass before committing
- Clippy linting with strict warnings

### Build

```bash
cargo build
```

### Test

```bash
cargo test
```

### Check Coverage

```bash
cargo tarpaulin --ignore-tests
```

## Project Structure

```
src/
├── main.rs   # Entry point
├── lib.rs    # Library root
├── url.rs    # URL parsing
├── scheme.rs # URL scheme handling
├── host.rs   # Host type
├── port.rs   # Port type
└── path.rs   # Path type
```

## License

This project is for educational purposes, implementing concepts from the Web Browser Engineering book.