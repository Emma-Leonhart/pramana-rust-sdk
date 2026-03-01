# Pramana Rust SDK - Implementation Guide

**Package name:** `pramana-sdk` (crates.io)
**Minimum Rust:** 1.70+
**Reference implementation:** [PramanaLib (C#)](https://github.com/Emma-Leonhart/PramanaLib)

---

## 1. Project Structure

```
pramana-rust-sdk/
├── Cargo.toml
├── src/
│   ├── lib.rs                      # Public API re-exports
│   ├── gaussian_rational.rs        # GaussianRational implementation
│   ├── pramana_id.rs               # UUID v5 generation utilities
│   ├── number_type.rs              # NumberType enum
│   ├── item.rs                     # PramanaItem enum/trait
│   ├── entity.rs                   # PramanaEntity struct
│   ├── property.rs                 # PramanaProperty struct
│   ├── proposition.rs              # PramanaProposition struct
│   ├── sense.rs                    # PramanaSense struct
│   ├── graph.rs                    # PramanaGraph (loading, serialization)
│   ├── orm/
│   │   ├── mod.rs
│   │   ├── config.rs               # PramanaConfig
│   │   ├── mapping.rs              # Derive macro support types
│   │   └── query.rs                # Query builder
│   ├── datasources/
│   │   ├── mod.rs
│   │   ├── pra_file.rs             # .pra JSON file reader
│   │   ├── sparql.rs               # GraphDB SPARQL connector
│   │   ├── rest_api.rs             # Pramana REST API connector
│   │   └── sqlite.rs               # SQLite export reader
│   └── structs/
│       ├── mod.rs
│       ├── date.rs                 # date: pseudo-class
│       ├── time.rs                 # time: pseudo-class
│       ├── interval.rs             # interval: pseudo-class
│       ├── coordinate.rs           # coord: pseudo-class
│       └── chemical.rs             # chem: / element: pseudo-classes
├── pramana-derive/                 # Proc macro crate (separate)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                  # #[derive(PramanaEntity)] proc macro
├── tests/
│   ├── gaussian_rational_tests.rs
│   ├── pramana_id_tests.rs
│   ├── item_model_tests.rs
│   ├── orm_tests.rs
│   ├── serialization_tests.rs
│   └── test_vectors.json           # Cross-language test vectors
├── benches/
│   └── gaussian_rational_bench.rs  # Criterion benchmarks
└── docs/
    └── api.md
```

## 2. Build & Packaging

### Cargo.toml

```toml
[package]
name = "pramana-sdk"
version = "0.1.0"
edition = "2021"
rust-version = "1.70"
description = "Rust SDK for the Pramana knowledge graph"
license = "MIT"
repository = "https://github.com/Emma-Leonhart/pramana-rust-sdk"

[dependencies]
num = "0.4"                    # BigInt, BigRational
uuid = { version = "1.0", features = ["v5", "serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sha1 = "0.10"                  # For UUID v5 (or use uuid crate's built-in)
thiserror = "1.0"              # Error types

[dev-dependencies]
criterion = "0.5"
proptest = "1.0"               # Property-based testing

[features]
default = []
sparql = ["dep:reqwest"]
api = ["dep:reqwest"]
sqlite = ["dep:rusqlite"]
derive = ["dep:pramana-derive"]
full = ["sparql", "api", "sqlite", "derive"]

[dependencies.reqwest]
version = "0.12"
optional = true
features = ["json"]

[dependencies.rusqlite]
version = "0.31"
optional = true

[dependencies.pramana-derive]
path = "./pramana-derive"
optional = true

[[bench]]
name = "gaussian_rational_bench"
harness = false
```

### Key decisions:
- **`num` crate** for `BigInt` and `BigRational` (de facto standard)
- **`uuid` crate** with `v5` feature for deterministic UUIDs
- **Feature flags** for optional dependencies (SPARQL, REST API, SQLite)
- **Separate proc-macro crate** (`pramana-derive`) for `#[derive(PramanaEntity)]`
- **Serde** for serialization (Rust ecosystem standard)
- **Zero required network dependencies** — SPARQL/REST are opt-in features

## 3. GaussianRational (Gauss) Implementation

> **Naming convention:** The standard short name for a Gaussian rational is **`Gauss`**. When referring specifically to a Gaussian integer (both denominators are 1), the standard short name is **`Gint`**.

### 3.1 Struct Design

Rust uses `num::BigInt` from the `num` crate for arbitrary precision.

```rust
use num::BigInt;
use num::integer::gcd;
use std::fmt;
use uuid::Uuid;

/// Exact complex rational number: a/b + (c/d)i.
/// Immutable — all fields are private, all operations return new values.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct GaussianRational {
    a: BigInt, // real numerator
    b: BigInt, // real denominator (positive, nonzero)
    c: BigInt, // imaginary numerator
    d: BigInt, // imaginary denominator (positive, nonzero)
}

impl GaussianRational {
    pub fn new(a: BigInt, b: BigInt, c: BigInt, d: BigInt) -> Result<Self, PramanaError> {
        if b <= BigInt::ZERO || d <= BigInt::ZERO {
            return Err(PramanaError::InvalidDenominator);
        }
        // Normalize to canonical form
        let g_real = gcd(a.abs(), b.clone());
        let g_imag = gcd(c.abs(), d.clone());
        Ok(Self {
            a: &a / &g_real,
            b: &b / &g_real,
            c: &c / &g_imag,
            d: &d / &g_imag,
        })
    }

    /// Convenience constructor from i64 values.
    pub fn from_i64(a: i64, b: i64, c: i64, d: i64) -> Result<Self, PramanaError> {
        Self::new(
            BigInt::from(a), BigInt::from(b),
            BigInt::from(c), BigInt::from(d),
        )
    }
}
```

### 3.2 Constructors

```rust
    pub fn from_int(value: i64) -> Self {
        Self::from_i64(value, 1, 0, 1).unwrap()
    }

    pub fn from_bigint(value: BigInt) -> Self {
        Self::new(value, BigInt::from(1), BigInt::from(0), BigInt::from(1)).unwrap()
    }

    pub fn from_complex(real: i64, imag: i64) -> Self {
        Self::from_i64(real, 1, imag, 1).unwrap()
    }

    pub fn parse(s: &str) -> Result<Self, PramanaError> {
        let normalized = s.strip_prefix("num:").unwrap_or(s);
        let parts: Vec<&str> = normalized.split(',').collect();
        if parts.len() != 4 {
            return Err(PramanaError::ParseError(format!("Expected 4 components: {}", s)));
        }
        Self::new(
            parts[0].trim().parse()?,
            parts[1].trim().parse()?,
            parts[2].trim().parse()?,
            parts[3].trim().parse()?,
        )
    }
```

### 3.3 Operator Overloading via `std::ops` Traits

Rust supports full operator overloading through trait implementations:

```rust
use std::ops::{Add, Sub, Mul, Div, Rem, Neg};

impl Add for &GaussianRational {
    type Output = GaussianRational;

    fn add(self, other: &GaussianRational) -> GaussianRational {
        let real_num = &self.a * &other.b + &other.a * &self.b;
        let real_den = &self.b * &other.b;
        let imag_num = &self.c * &other.d + &other.c * &self.d;
        let imag_den = &self.d * &other.d;
        GaussianRational::new(real_num, real_den, imag_num, imag_den).unwrap()
    }
}

impl Sub for &GaussianRational { ... }
impl Neg for &GaussianRational { ... }

impl Mul for &GaussianRational {
    type Output = GaussianRational;

    fn mul(self, other: &GaussianRational) -> GaussianRational {
        // (a+bi)(c+di) = (ac-bd) + (ad+bc)i
        ...
    }
}

impl Div for &GaussianRational { ... }

impl Rem for &GaussianRational {
    type Output = GaussianRational;

    fn rem(self, other: &GaussianRational) -> GaussianRational {
        if !self.is_real() || !other.is_real() {
            panic!("Modulo only defined for real values");
        }
        ...
    }
}
```

Also implement for owned values (`impl Add for GaussianRational`) and mixed (`impl Add<&GaussianRational> for GaussianRational`) for ergonomics.

### 3.4 Comparison via `PartialOrd`

```rust
impl PartialOrd for GaussianRational {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if !self.is_real() || !other.is_real() {
            return None; // Complex values are not ordered
        }
        // Compare a1/b1 vs a2/b2 via cross-multiplication
        let lhs = &self.a * &other.b;
        let rhs = &other.a * &self.b;
        lhs.partial_cmp(&rhs)
    }
}

// Note: NOT implementing Ord, because complex values have no total order.
// PartialOrd returns None for complex values, which is correct.
```

### 3.5 Properties

```rust
    pub fn is_real(&self) -> bool { self.c == BigInt::ZERO }
    pub fn is_integer(&self) -> bool { self.is_real() && self.b == BigInt::from(1) }
    pub fn is_gaussian_integer(&self) -> bool {
        self.b == BigInt::from(1) && self.d == BigInt::from(1)
    }
    pub fn is_zero(&self) -> bool { self.a == BigInt::ZERO && self.c == BigInt::ZERO }
    pub fn is_positive(&self) -> bool { self.is_real() && self.a > BigInt::ZERO }
    pub fn is_negative(&self) -> bool { self.is_real() && self.a < BigInt::ZERO }

    pub fn conjugate(&self) -> GaussianRational {
        GaussianRational::new(self.a.clone(), self.b.clone(), -&self.c, self.d.clone()).unwrap()
    }

    pub fn magnitude_squared(&self) -> GaussianRational { ... }
    pub fn real_part(&self) -> GaussianRational { ... }
    pub fn imaginary_part(&self) -> GaussianRational { ... }
    pub fn reciprocal(&self) -> GaussianRational { ... }

    pub fn classify(&self) -> NumberType { ... }

    pub fn real_numerator(&self) -> &BigInt { &self.a }
    pub fn real_denominator(&self) -> &BigInt { &self.b }
    pub fn imag_numerator(&self) -> &BigInt { &self.c }
    pub fn imag_denominator(&self) -> &BigInt { &self.d }

    pub fn pow(&self, exp: i32) -> GaussianRational { ... }
```

### 3.6 Pramana ID (UUID v5)

```rust
use uuid::Uuid;

const NUM_NAMESPACE: Uuid = Uuid::from_bytes([
    0xa6, 0x61, 0x33, 0x21, 0xe9, 0xf6, 0x43, 0x48,
    0x8f, 0x8b, 0x29, 0xd2, 0xa3, 0xc8, 0x63, 0x49,
]);

impl GaussianRational {
    pub fn canonical(&self) -> String {
        format!("num:{},{},{},{}", self.a, self.b, self.c, self.d)
    }

    pub fn pramana_id(&self) -> Uuid {
        Uuid::new_v5(&NUM_NAMESPACE, self.canonical().as_bytes())
    }

    pub fn pramana_uri(&self) -> String {
        format!("pra:{}", self.pramana_id())
    }
}
```

The `uuid` crate with the `v5` feature handles UUID v5 generation natively.

### 3.7 Display and Formatting

```rust
impl fmt::Display for GaussianRational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.canonical())
    }
}

impl GaussianRational {
    pub fn to_mixed(&self) -> String { ... }
    pub fn to_improper(&self) -> String { ... }
    pub fn to_raw(&self) -> String {
        format!("<{},{},{},{}>", self.a, self.b, self.c, self.d)
    }
}
```

### 3.8 Serde Serialization

```rust
use serde::{Serialize, Deserialize, Serializer, Deserializer};

impl Serialize for GaussianRational {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.canonical())
    }
}

impl<'de> Deserialize<'de> for GaussianRational {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::parse(&s).map_err(serde::de::Error::custom)
    }
}
```

### 3.9 Intentionally Unsupported

```rust
    /// Panics with an explanation that complex magnitude is irrational.
    pub fn magnitude(&self) -> ! {
        panic!("Complex magnitude produces irrationals. Use magnitude_squared() for exact result.")
    }
    // phase(), to_polar(), sqrt() — same treatment
```

## 4. Error Handling

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PramanaError {
    #[error("Denominators must be positive integers")]
    InvalidDenominator,

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Operation not supported for complex values: {0}")]
    ComplexNotSupported(String),

    #[error("Item not found: {0}")]
    NotFound(Uuid),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
```

## 5. Item Model

### 5.1 Enum-Based Type Dispatch

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PramanaItem {
    Entity(PramanaEntity),
    Property(PramanaProperty),
    Proposition(PramanaProposition),
    Sense(PramanaSense),
}

impl PramanaItem {
    pub fn uuid(&self) -> &Uuid {
        match self {
            Self::Entity(e) => &e.uuid,
            Self::Property(p) => &p.uuid,
            Self::Proposition(p) => &p.uuid,
            Self::Sense(s) => &s.uuid,
        }
    }
}
```

### 5.2 Typed Structs

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PramanaEntity {
    pub uuid: Uuid,
    pub label: String,
    pub instance_of: Option<Uuid>,
    pub subclass_of: Option<Uuid>,
    pub properties: HashMap<String, serde_json::Value>,
    pub edges: HashMap<String, Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PramanaProperty {
    pub uuid: Uuid,
    pub label: String,
    pub datatype: String,
    pub formatter_url: Option<String>,
    pub description: Option<String>,
}
```

## 6. ORM-Style Mapping

### 6.1 Derive Macro (Proc Macro Crate)

```rust
// In pramana-derive/src/lib.rs
use proc_macro::TokenStream;

#[proc_macro_derive(PramanaEntity, attributes(pramana))]
pub fn derive_pramana_entity(input: TokenStream) -> TokenStream {
    // Generate impl block with field-to-proposition mapping
    ...
}
```

### 6.2 Usage

```rust
use pramana_derive::PramanaEntity;

#[derive(PramanaEntity)]
#[pramana(instance_of = "uuid-of-shinto-shrine-class")]
struct ShintoShrine {
    #[pramana(prop = "coordinates")]
    coordinates: Option<Coordinate>,

    #[pramana(prop = "Wikidata ID")]
    wikidata_id: Option<String>,

    #[pramana(prop = "part of")]
    part_of: Option<Box<ShintoShrine>>,
}
```

### 6.3 Traits for Multiple Classification

Rust traits handle the diamond pattern naturally:

```rust
pub trait ChemicalCompound {
    fn molecular_formula(&self) -> &str;
}

pub trait QuantumSubstance {
    fn quantum_state(&self) -> &str;
}

struct Water {
    // ...
}

impl ChemicalCompound for Water { ... }
impl QuantumSubstance for Water { ... }
```

### 6.4 Query Interface

```rust
let shrines: Vec<ShintoShrine> = pramana.query::<ShintoShrine>()
    .filter(|s| s.coordinates.is_some())
    .limit(100)
    .collect()?;

let water: ChemicalCompound = pramana.get_by_id(
    Uuid::parse_str("00000007-0000-4000-8000-000000000007")?
)?;
```

## 7. Rust-Specific Advantages

- **Full operator overloading** via `std::ops` traits
- **`PartialOrd` without `Ord`** — correctly models that complex numbers lack total ordering
- **Zero-cost abstractions** — trait dispatch, generics
- **Ownership system** — prevents data races in concurrent graph operations
- **Proc macros** — compile-time code generation for ORM mapping
- **`serde`** — ecosystem-standard serialization
- **`uuid` crate** — native UUID v5 support
- **No garbage collector** — predictable performance for large graphs
- **`thiserror`** — ergonomic error type definitions

## 8. Testing Strategy

```bash
cargo test                    # Run all tests
cargo test --features full    # Test with all features
cargo bench                   # Run benchmarks
cargo clippy                  # Lint
cargo doc --open              # Generate and view docs
```

- **Unit tests** in each module (`#[cfg(test)]` blocks)
- **Integration tests** in `tests/` directory
- **Property-based tests** with `proptest` for arithmetic invariants
- **Benchmarks** with Criterion for GaussianRational performance
- **Cross-language test vectors** from `test_vectors.json`

## 9. Implementation Priority

### Phase 1 - GaussianRational (core)
1. Implement `GaussianRational` struct with `num::BigInt` components
2. Implement `std::ops` traits (`Add`, `Sub`, `Mul`, `Div`, `Rem`, `Neg`)
3. Implement `PartialEq`, `Eq`, `Hash`, `PartialOrd`
4. Implement UUID v5 via `uuid` crate
5. Implement `FromStr`, `Display`, `Serialize`, `Deserialize`
6. Write test suite + property-based tests

### Phase 2 - Base Item Model
1. Implement `PramanaItem` enum and typed structs
2. Implement `PramanaGraph` with serde JSON serialization
3. Implement `.pra` file reader

### Phase 3 - ORM Mapping
1. Create `pramana-derive` proc macro crate
2. Implement `#[derive(PramanaEntity)]` with field attribute mapping
3. Implement query builder

### Phase 4 - Data Sources & Provenance
1. SPARQL connector (`reqwest`, feature-gated)
2. REST API connector (`reqwest`, feature-gated)
3. SQLite connector (`rusqlite`, feature-gated)
4. Provenance metadata

### Phase 5 - Pseudo-Classes
1. `PramanaDate`, `PramanaTime`, `PramanaInterval` (using `chrono` crate)
2. `Coordinate` struct
3. `ChemicalIdentifier` / `ChemicalElement`
