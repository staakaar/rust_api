## Rust REST API


## Lint
[Clippy](https://github.com/rust-lang/rust-clippy)が有名

CIで静的解析を組み込み、失敗してしまう場合
```zsh
cargo clippy -- -D warnings
```

コードブロックでの対応の場合
```rust
#[allow(clippy::lint_name)]
```

## Format
```zsh
rustup component add rustfmt
cargo fmt

# CI pipline
cargo fmt -- --check
```

## Security Vulnerabilities
```zsh
cargo install cargo-audit

# scan
cargo audit
```

## Rust Test

### Code Coverage

Rustのプロジェクトにおいて簡単にコードカバレッジが測定できる方法
```zsh
cargo install cargo-tarpaulin
```

[Codecov](https://about.codecov.io/)
[Coveralls](https://coveralls.io/)
