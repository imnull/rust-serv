# Contributing to rust-serv

Thank you for your interest in contributing to rust-serv! This document provides guidelines and instructions for contributing.

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment for all contributors.

## How to Contribute

### Reporting Bugs

If you find a bug, please open an issue with:
- Clear description of the problem
- Steps to reproduce
- Expected vs actual behavior
- Your environment (OS, Rust version)

### Suggesting Features

Feature suggestions are welcome! Please open an issue describing:
- The feature you'd like to see
- Why it would be useful
- Possible implementation approach (if you have ideas)

### Pull Requests

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test`)
5. Run clippy (`cargo clippy`)
6. Format code (`cargo fmt`)
7. Commit your changes (`git commit -m 'Add amazing feature'`)
8. Push to the branch (`git push origin feature/amazing-feature`)
9. Open a Pull Request

## Development Setup

### Prerequisites

- Rust 1.70+ (recommended: latest stable)
- Git

### Building

```bash
git clone https://github.com/imnull/rust-serv.git
cd rust-serv
cargo build
```

### Testing

```bash
# Run all tests
cargo test

# Run with coverage
cargo tarpaulin --out Html
```

### Running

```bash
cargo run
```

## Code Style

- Use `cargo fmt` before committing
- Follow Rust naming conventions
- Write doc comments for public APIs
- Add tests for new functionality

## Project Structure

```
rust-serv/
├── src/
│   ├── main.rs           # Entry point
│   ├── lib.rs            # Library root
│   ├── server/           # HTTP server implementation
│   ├── handler/          # Request handlers
│   ├── middleware/       # Middleware layers
│   ├── config/           # Configuration management
│   └── ...               # Other modules
├── tests/                # Integration tests
├── benches/              # Benchmarks
└── docs/                 # Documentation
```

## Testing Guidelines

- Write unit tests for new functionality
- Maintain 95%+ test coverage
- Use realistic test data
- Test edge cases and error conditions

## Documentation

- Update README.md for user-facing changes
- Add/update doc comments for public APIs
- Update docs/ for significant features
- Include examples in documentation

## Performance

rust-serv prioritizes performance. When contributing:
- Benchmark significant changes
- Avoid unnecessary allocations
- Use appropriate data structures
- Consider memory usage

## Questions?

Feel free to open an issue for questions or join discussions in existing issues.

## License

By contributing, you agree that your contributions will be dual-licensed under:
- MIT License ([LICENSE-MIT](./LICENSE-MIT))
- Apache License 2.0 ([LICENSE-APACHE](./LICENSE-APACHE))

---

Thank you for contributing to rust-serv! 🚀
