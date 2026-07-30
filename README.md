# Rust Trading Systems Lab

[![Rust CI](https://github.com/AndriiSolomon/rust-trading-systems-lab/actions/workflows/ci.yml/badge.svg)](https://github.com/AndriiSolomon/rust-trading-systems-lab/actions/workflows/ci.yml)

A small, self-contained Rust repository demonstrating defensive engineering patterns for trading-system infrastructure.

## What this repository demonstrates

- deterministic order-state transitions;
- pre-trade risk validation with explicit failure modes;
- local-versus-exchange state reconciliation;
- protection of order invariants through private state;
- unit and integration testing;
- automated formatting, linting, and tests with GitHub Actions.

## Important boundary

This repository contains **demonstration code only**.

It does **not** contain production AUREXIS source code, proprietary strategy logic, API credentials, live account data, exchange keys, or production risk parameters.

## Repository layout

```text
rust-trading-systems-lab/
├── .github/
│   └── workflows/
│       └── ci.yml
├── src/
│   ├── lib.rs
│   ├── order_state.rs
│   ├── reconciliation.rs
│   └── risk.rs
├── tests/
│   └── trading_workflow.rs
├── .editorconfig
├── .gitignore
├── Cargo.toml
├── README.md
└── rust-toolchain.toml
```

## Run locally

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

## Design principles

- **Explicit transitions:** order status changes only through validated events.
- **Protected invariants:** callers cannot directly mutate internal order state.
- **Typed failures:** invalid inputs and rejected operations return specific errors.
- **Defensive validation:** non-finite and structurally invalid numeric inputs are rejected.
- **Deterministic reconciliation:** state differences produce explicit reconciliation actions.
- **No runtime dependencies:** the crate uses only the Rust standard library.

## Author

**Andrii Soloviov**

Rust Developer | Trading Systems Builder | Fintech Infrastructure

[LinkedIn](https://www.linkedin.com/in/andrii-aurexis/) · [AUREXIS](https://whiterock-aurexis.com)
