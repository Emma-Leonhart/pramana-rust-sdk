# pramana-rust-sdk

Rust SDK for the [Pramana](https://pramana.dev) knowledge graph. Provides exact-arithmetic value types, item model mapping, and data source connectors for working with Pramana data in Rust.

> **Note:** Unlike the other Pramana SDKs, this one is being developed as a learning project — Rust is a language the maintainer is exploring for the first time through vibecoding. Contributions, corrections, and idiomatic Rust suggestions are very welcome!

## Status

**Phase 1 implemented** — Core Gaussian rational and Gaussian integer arithmetic with deterministic Pramana UUID v5 identity.

## Key Features

- **Gauss** — Exact Gaussian rational arithmetic (`A/B + (C/D)i`) with arbitrary-precision `BigInt` components
- **Gint** — Gaussian integers (`Z[i]`), a subset of Gauss with integer-only components
- **Deterministic Pramana IDs** — UUID v5 generation matching the canonical Pramana web app
- **Full operator overloading** — `+`, `-`, `*`, `/`, `%` via `std::ops` traits
- **Number theory** — GCD, extended GCD, Gaussian primality testing, modular congruence
- **Serde integration** — Serialize/deserialize Pramana types to/from JSON
- **Parsing** — Parse from canonical `"A,B,C,D"` and `"pra:num:A,B,C,D"` formats
- **Multiple display formats** — Human-readable, raw vector, improper fraction, mixed number, decimal

## Installation

```toml
[dependencies]
pramana-sdk = "0.1"
```

## Quick Example

```rust
use pramana_sdk::{Gauss, Gint};

// Gaussian integer: 3 + 4i
let z = Gint::new(3, 4);
println!("{}", z);                  // "3 + 4i"
println!("{}", z.pramana_id());     // deterministic UUID v5

// Gaussian rational: 1/2 + 3/4 i
let w = Gauss::new(1, 2, 3, 4);
println!("{}", w);                  // "1/2 + 3/4i"
println!("{}", w.to_raw_string());  // "<1,2,3,4>"

// Arithmetic
let sum = Gauss::from_ints(1, 1) + Gauss::from_ints(2, 3);
println!("{}", sum);                // "3 + 4i"

// Division is exact
let a = Gauss::from_ints(2, 2);
let b = Gauss::from_ints(1, 1);
println!("{}", a / b);              // "2"
```

## Documentation

- [General SDK Specification](08_SDK_LIBRARY_SPECIFICATION.md) - Cross-language design spec
- [Rust Implementation Guide](IMPLEMENTATION.md) - Rust-specific implementation details

## Acknowledgments

The Gauss and Gint implementations across all Pramana SDKs were heavily inspired by [gaussian_integers](https://github.com/alreich/gaussian_integers) by **Alfred J. Reich, Ph.D.**, which provides exact arithmetic for Gaussian integers and Gaussian rationals in Python.

## Pramana SDK Family

| Language | Repository | Package |
|----------|-----------|---------|
| C# / .NET | [pramana-dotnet-sdk](https://github.com/Emma-Leonhart/pramana-dotnet-sdk) | `Pramana.SDK` (NuGet) |
| Python | [pramana-python-sdk](https://github.com/Emma-Leonhart/pramana-python-sdk) | `pramana-sdk` (PyPI) |
| TypeScript | [pramana-ts-sdk](https://github.com/Emma-Leonhart/pramana-ts-sdk) | `@pramana/sdk` (npm) |
| JavaScript | [pramana-js-sdk](https://github.com/Emma-Leonhart/pramana-js-sdk) | `@pramana/sdk` (npm) |
| Java | [pramana-java-sdk](https://github.com/Emma-Leonhart/pramana-java-sdk) | `org.pramana:pramana-sdk` (Maven) |
| Rust | **pramana-rust-sdk** (this repo) | `pramana-sdk` (crates.io) |
| Go | [pramana-go-sdk](https://github.com/Emma-Leonhart/pramana-go-sdk) | `github.com/Emma-Leonhart/pramana-go-sdk` |

All SDKs implement the same core specification and must produce identical results for UUID v5 generation, canonical string normalization, and arithmetic operations.
