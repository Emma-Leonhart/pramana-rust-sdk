# pramana-rust-sdk

Rust SDK for the [Pramana](https://pramana-data.ca) knowledge graph. Provides exact-arithmetic value types, item model mapping, and data source connectors for working with Pramana data in Rust.

## Status

**Pre-implementation** - Project structure and implementation plan documented. See [IMPLEMENTATION.md](IMPLEMENTATION.md) for the full design.

## Key Features (Planned)

- **GaussianRational** (standard short name: **Gauss**; Gaussian integers: **Gint**) - Exact complex rational arithmetic (`a/b + (c/d)i`) with `num::BigInt`
- **Deterministic Pramana IDs** - UUID v5 generation matching the canonical Pramana web app
- **Full operator overloading** - `+`, `-`, `*`, `/`, `%` via `std::ops` traits
- **Correct partial ordering** - `PartialOrd` without `Ord` for complex numbers
- **Derive macros** - `#[derive(PramanaEntity)]` for ORM-style mapping
- **Feature-gated dependencies** - SPARQL, REST API, SQLite are opt-in
- **Serde integration** - Serialize/deserialize Pramana types to/from JSON

## Installation (Future)

```toml
[dependencies]
pramana-sdk = "0.1"
```

## Quick Example (Planned API)

```rust
use pramana_sdk::GaussianRational;

let half = GaussianRational::from_i64(1, 2, 0, 1)?;   // 1/2
let third = GaussianRational::from_i64(1, 3, 0, 1)?;  // 1/3
let result = &half + &third;                             // 5/6

println!("{}", result.pramana_id());  // deterministic UUID v5
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
