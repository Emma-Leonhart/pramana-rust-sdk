# Pramana SDK / Library Specification

**Target languages:** C#, Python, JavaScript, TypeScript, Java, Rust, Go

**Purpose:** Provide language-native libraries that allow developers to work with Pramana knowledge graph data using idiomatic constructs — similar to how an ORM maps database rows to objects, but adapted for Pramana's ontological model (items, propositions, edges, and Gaussian rationals).

**Reference implementation:** [PramanaLib (C#)](https://github.com/Emma-Leonhart/PramanaLib) — implements `GaussianRational` struct with exact arithmetic and deterministic UUID v5 generation.

---

## 1. Core Principles

### 1.1 Every Object Has a Pramana ID

Every Pramana-mapped object MUST carry its Pramana UUID and be constructable from one.

```
// Pseudocode — all languages must support this pattern
item = PramanaEntity.from_id("pra:00000007-0000-4000-8000-000000000007")
item.pramana_id  // → Guid/UUID "00000007-0000-4000-8000-000000000007"
item.pramana_uri // → "pra:00000007-0000-4000-8000-000000000007"
```

For struct pseudo-classes (e.g. `num:`, `date:`, `coord:`), the UUID is deterministic (v5) and computed from the canonical string representation. For regular items, the UUID is stored/assigned (v4).

### 1.2 Proposition-Oriented, Not Column-Oriented

Unlike traditional ORMs where an object's fields map to database columns, Pramana objects derive their properties from **Propositions** — first-class statement objects that link a subject to a value via a property. Each "field" on a mapped object corresponds to one or more propositions in the graph.

This means:
- A single field may have multiple values (multiple propositions with the same property)
- Each value carries provenance (the Proposition UUID, evidence links, temporal qualifiers)
- Values can be overridden, deprecated, or contested at the proposition level

### 1.3 Flattened Inheritance (Configurable)

Pramana's ontology has deep hierarchies (e.g. `Item > Entity > Particular > Continuant > Independent Continuant > Material Entity > Object`). Languages should **NOT** reproduce the full depth as class inheritance. Instead:

- The SDK flattens the hierarchy to a configurable depth (default: 2-3 levels)
- Deep ontological parents become interfaces/traits rather than base classes
- The flattening depth is configurable per project via a configuration object or file

```
// Flattening depth = 2 (default)
class WaterMolecule extends Entity { ... }

// Flattening depth = 4 (more specific)
class WaterMolecule extends MaterialEntity { ... }

// Regardless of depth, ontological ancestry is queryable at runtime:
water.is_instance_of("Continuant")  // → true
water.ancestry()                     // → [MaterialEntity, IndependentContinuant, Continuant, Particular, Entity, Item]
```

### 1.4 Interfaces for Diamond Patterns

Pramana supports multiple classification through separate edge types (`instanceOf`, `member of classification`, `role type`, `conceptual category`). In OO languages, these map to:

| Language | Mechanism |
|----------|-----------|
| C# | `interface` |
| Java | `interface` |
| TypeScript | `interface` |
| Python | Abstract base class (ABC) / Protocol |
| Rust | `trait` |
| Go | Interface (implicit) |
| JavaScript | Mixin pattern / Symbol-based protocols |

An entity that is both a `Chemical Compound` and a `Quantum Substance` would implement both interfaces while inheriting from a single flattened base class.

### 1.5 Per-Object Overrides

Individual Pramana items can override class-level behavior. This covers two scenarios:

**a) Computed properties on instances:**
A Property can have an associated computation rule (e.g. "calculate age from date of birth"). When a class defines such a rule, any instance can:
- Use the computed value (default)
- Override it with a manually asserted Proposition
- The override takes precedence, but the computed value remains available via `.computed_<property>()`

**b) Interface implementation on instances:**
An individual item may implement an interface that its class does not require. This is analogous to Pramana's `implements interface` edge on a specific entity rather than its class.

```
// Pseudocode
class Person extends Entity {
    // Class-level computed property
    age() => today() - this.date_of_birth
}

// Per-object override
alice.age = 42  // Manual override takes precedence
alice.computed_age()  // Still returns the computed value
```

---

## 2. GaussianRational — Required in All Languages

The `GaussianRational` type is the single most important type that every Pramana SDK must implement. It represents the `num:` pseudo-class: a complex number `a/b + (c/d)i` with exact arbitrary-precision rational arithmetic.

### 2.1 Storage Format

Four integer components: `(A, B, C, D)` representing `A/B + (C/D)i`

- `A`: real numerator (any integer)
- `B`: real denominator (positive integer, never zero)
- `C`: imaginary numerator (any integer)
- `D`: imaginary denominator (positive integer, never zero)

**Canonical form:** Fractions reduced to lowest terms, denominators always positive.
`num:2,4,0,1` canonicalizes to `num:1,2,0,1`.

### 2.2 Required Operations

All Pramana SDKs MUST implement the following for `GaussianRational`:

#### Constructors
| Constructor | Meaning |
|-------------|---------|
| `GaussianRational(a, b, c, d)` | Full form: `a/b + (c/d)i` |
| `GaussianRational(real, imag)` | Integer parts: `real + imag*i` |
| `GaussianRational(value)` | Real integer (imaginary = 0) |
| `GaussianRational.parse("a,b,c,d")` | From canonical string |
| `GaussianRational.from_pramana_id(uuid)` | Reverse lookup if value is known |

#### Arithmetic (operator overloading where supported)
| Operation | Symbol | Notes |
|-----------|--------|-------|
| Addition | `+` | Component-wise rational addition |
| Subtraction | `-` | Component-wise rational subtraction |
| Negation | `-` (unary) | Negate both components |
| Multiplication | `*` | Full complex multiplication `(a+bi)(c+di)` |
| Division | `/` | Multiply by conjugate, divide |
| Modulo | `%` | Real-only; throw for complex |
| Exponentiation | `pow(base, int_exp)` | Integer exponents only (static method) |

#### Comparison (operator overloading where supported)
| Operation | Notes |
|-----------|-------|
| Equality (`==`, `!=`) | Component-wise comparison after normalization |
| Ordering (`<`, `>`, `<=`, `>=`) | Real values only; throw for complex |

#### Properties / Accessors
| Property | Type | Description |
|----------|------|-------------|
| `a` / `real_numerator` | BigInt | Real part numerator |
| `b` / `real_denominator` | BigInt | Real part denominator |
| `c` / `imag_numerator` | BigInt | Imaginary part numerator |
| `d` / `imag_denominator` | BigInt | Imaginary part denominator |
| `is_real` | bool | `C == 0` |
| `is_integer` | bool | Real and `B == 1` |
| `is_gaussian_integer` | bool | `B == 1 && D == 1` |
| `is_zero` | bool | `A == 0 && C == 0` |
| `is_positive` | bool | Real and `A > 0` |
| `is_negative` | bool | Real and `A < 0` |
| `conjugate` | GaussianRational | `A/B - (C/D)i` |
| `magnitude_squared` | GaussianRational | `(A/B)^2 + (C/D)^2` (exact) |
| `real_part` | GaussianRational | `A/B + 0i` |
| `imaginary_part` | GaussianRational | `C/D` as real value |
| `reciprocal` | GaussianRational | `1/z` via conjugate method |
| `pramana_id` | UUID/Guid | Deterministic UUID v5 |

#### Intentionally Unsupported
These MUST throw/error rather than silently losing precision:

| Method | Reason |
|--------|--------|
| `magnitude` / `abs` (complex) | Square root produces irrationals |
| `phase` / `arg` | Arctangent produces irrationals |
| `to_polar` | Both magnitude and phase are irrational |
| `sqrt` | Produces irrationals for most inputs |

### 2.3 Pramana ID Generation

Every `GaussianRational` value has a deterministic UUID v5 identity:

1. Compute the canonical `num:` string: `num:{A},{B},{C},{D}` (after normalization)
2. Generate UUID v5 using namespace `a6613321-e9f6-4348-8f8b-29d2a3c86349` and the canonical string as the name
3. The resulting UUID matches what the Pramana web app generates

**This UUID generation algorithm must be identical across all language implementations** — this is what makes cross-language interop possible.

### 2.4 Number Hierarchy Classification

Each `GaussianRational` can be classified into the Pramana number hierarchy:

```
Number (complex)
├── Gaussian Rational (non-zero imaginary part)
└── Real Number (imaginary part is zero)
    ├── Rational Number (non-integer)
    │   └── (e.g. 3/4)
    └── Integer (denominator is 1)
        └── Whole Number (non-negative)
            └── Natural Number (positive)
```

The SDK should provide a `classify()` or `number_type` method that returns the most specific classification.

### 2.5 Formatting

All implementations must support at least these string representations:

| Format | Example for `7/2 + 3/4i` | Description |
|--------|---------------------------|-------------|
| Mixed | `3 & 1/2 + 3/4 i` | Human-readable with mixed fractions |
| Improper | `7/2 + 3/4 i` | Improper fractions |
| Raw | `<7,2,3,4>` | Component tuple |
| Canonical | `num:7,2,3,4` | Pramana `num:` namespace form |

### 2.6 Language-Specific Implementation Notes

| Language | BigInt Type | Struct/Class | Operator Overloading |
|----------|------------|--------------|---------------------|
| C# | `System.Numerics.BigInteger` | `readonly struct` | Full support via operator keywords |
| Python | `int` (arbitrary precision natively) | Frozen dataclass or `__slots__` class | `__add__`, `__mul__`, etc. |
| TypeScript | `bigint` (native) | `class` (immutable via readonly) | No operator overloading; use methods `.add()`, `.mul()` etc. |
| JavaScript | `BigInt` (native) | `class` (freeze in constructor) | No operator overloading; use methods |
| Java | `java.math.BigInteger` | `final class` (immutable) | No operator overloading; use methods `.add()`, `.multiply()` etc. |
| Rust | `num::BigInt` (from `num` crate) | `struct` with `#[derive(Clone, Copy)]` if small enough, else owned | Full support via `std::ops` traits |
| Go | `math/big.Int` | Struct with value receivers | No operator overloading; use methods |

---

## 3. Pramana Item Model Mapping

### 3.1 Base Item Type

Every Pramana item maps to a base type with these core fields:

```
PramanaItem {
    uuid: UUID                    // The Pramana UUID (always present)
    type: ItemType                // Entity | Property | Proposition | Sense | Evidence | StanceLink
    properties: Map<string, any>  // Literal properties (labels, objectValue, etc.)
    edges: Map<string, UUID>      // Edges to other items (instanceOf, subclassOf, subject, object, etc.)
}
```

### 3.2 Typed Subclasses

The SDK should provide typed wrappers for each item type:

```
PramanaEntity extends PramanaItem {
    label: string                  // from properties.EntityLabel
    instance_of: PramanaEntity?    // resolved from edges.instanceOf
    subclass_of: PramanaEntity?    // resolved from edges.subclassOf
    propositions: List<Proposition> // all propositions where this is the subject
    senses: List<Sense>            // all senses that refer to this entity
}

PramanaProperty extends PramanaItem {
    label: string                  // from properties.PropertyLabel
    datatype: string               // "external-id" | "pramana-item" | "struct" | legacy types
    formatter_url: string?         // URL template for external-id display
    description: string?           // Human-readable description
}

PramanaProposition extends PramanaItem {
    subject: PramanaItem           // resolved from edges.subject
    property: PramanaProperty      // resolved from edges.instanceOf
    object: PramanaItem?           // resolved from edges.object (for pramana-item values)
    object_value: string?          // from properties.objectValue (for literal values)
}

PramanaSense extends PramanaItem {
    label: string                  // from properties.SenseLabel
    language: string?              // from properties.language
    refers_to: PramanaEntity?      // resolved from edges.refersTo
    sense_type: PramanaEntity?     // resolved from edges.instanceOf
}
```

### 3.3 Lazy vs. Eager Resolution

Edge references (UUIDs) can be resolved in two modes:

- **Lazy (default):** Edges store UUIDs; resolved on first access
- **Eager:** All edges resolved at load time (useful for small graphs or cached data)

The mode should be configurable per-session or per-query.

---

## 4. Ontology-to-Class Mapping ("Pramana ORM")

### 4.1 The Mapping Concept

Developers define language-native classes that map to Pramana entity types. The SDK maps propositions to class fields, handling type conversion, multiplicity, and provenance tracking.

```python
# Python example
@pramana_entity(instance_of="uuid-of-Shinto-shrine-class")
class ShintoShrine:
    name: str                           # maps to EntityLabel
    coordinates: Coordinate             # maps to "coordinates" property
    wikidata_id: Optional[str]          # maps to "Wikidata ID" property
    part_of: Optional["ShintoShrine"]   # maps to "part of" property (edge)
```

```csharp
// C# example
[PramanaEntity(InstanceOf = "uuid-of-Shinto-shrine-class")]
public class ShintoShrine : PramanaEntity
{
    [PramanaProperty("coordinates")]
    public Coordinate Coordinates { get; set; }

    [PramanaProperty("Wikidata ID")]
    public string? WikidataId { get; set; }
}
```

```rust
// Rust example
#[pramana_entity(instance_of = "uuid-of-Shinto-shrine-class")]
struct ShintoShrine {
    #[pramana_prop("coordinates")]
    coordinates: Option<Coordinate>,

    #[pramana_prop("Wikidata ID")]
    wikidata_id: Option<String>,
}
```

### 4.2 Property Mapping Annotations

Each mapped field needs to specify:

| Annotation | Purpose | Default |
|------------|---------|---------|
| Property name/UUID | Which Pramana property this field represents | Field name (snake_case → title case) |
| Multiplicity | Single value or list | Single |
| Required | Whether the field must have a value | false |
| Computed | Whether this is a computed property with a rule | false |
| Override allowed | Whether per-object overrides are supported | true |

### 4.3 Flattening Configuration

```python
# Python example
pramana_config = PramanaConfig(
    flatten_depth=3,           # How many inheritance levels to preserve as classes
    lazy_resolve=True,         # Lazy edge resolution
    include_provenance=False,  # Don't track proposition UUIDs on simple reads
)
```

```typescript
// TypeScript example
const config: PramanaConfig = {
    flattenDepth: 3,
    lazyResolve: true,
    includeProvenance: false,
};
```

### 4.4 Query Interface

The SDK should support querying items from a Pramana data source:

```python
# Python
shrines = pramana.query(ShintoShrine).filter(coordinates__not_null=True).all()
water = pramana.get_by_id("00000007-0000-4000-8000-000000000007", as_type=ChemicalCompound)
```

```csharp
// C#
var shrines = pramana.Query<ShintoShrine>().Where(s => s.Coordinates != null).ToList();
var water = pramana.GetById<ChemicalCompound>("00000007-0000-4000-8000-000000000007");
```

---

## 5. Data Sources

The SDK must support loading Pramana data from multiple sources:

| Source | Priority | Notes |
|--------|----------|-------|
| `.pra` file (JSON) | HIGH | Direct file read; the canonical format |
| GraphDB SPARQL endpoint | HIGH | HTTP queries to `localhost:7200` or remote |
| Pramana REST API | MEDIUM | FastAPI endpoints on `pramana-data.ca` |
| SQLite export | LOW | Read from `pramana_export.db` |
| MongoDB export | LOW | Read from MongoDB collection |

```python
# Python — multiple data sources
graph = PramanaGraph.from_file("foundation.pra")
graph = PramanaGraph.from_sparql("http://localhost:7200/repositories/pramana")
graph = PramanaGraph.from_api("https://pramana-data.ca")
```

---

## 6. Struct Pseudo-Classes

Beyond `GaussianRational` (`num:`), the following pseudo-classes exist. These are lower priority for SDK implementation but should be planned for:

| Pseudo-class | Namespace UUID | Constructor Example | SDK Priority |
|--------------|---------------|---------------------|-------------|
| `num:` | `a6613321-e9f6-4348-8f8b-29d2a3c86349` | `num:3,4,0,1` | **REQUIRED** |
| `date:` | `924b23ca-50d5-46d5-b974-ea01d33f4e13` | `date:2026-02-28` | Medium |
| `time:` | `0802c070-38cb-482e-9aac-81578f3c8463` | `time:14:30:00` | Medium |
| `interval:` | `ff206534-16f0-4e90-8286-4c730e63f86a` | `interval:2025-01-01/2026-01-01` | Medium |
| `coord:` | (uses `struct` namespace) | `coord:35.06028,135.7528` | Medium |
| `chem:` / `inchi:` | `8b526203-d299-4288-9760-ecaca9d4e1c0` | `chem:InChI=1S/H2O/h1H2` | Low |
| `element:` | `9a850d17-368c-4613-a34a-83f6b2159581` | `element:8` | Low |

For `date:`, `time:`, and `interval:`, the SDK should map to the language's native date/time types while preserving the Pramana ID. For `coord:`, a lightweight lat/lon pair struct suffices.

Each of these must generate deterministic UUID v5 identifiers using the namespace UUID listed above, the same way `num:` does.

---

## 7. Operator Overloading Strategy

Operator overloading support varies significantly by language. Here is the strategy for each:

### 7.1 Languages with Full Operator Overloading

**C#, Python, Rust** — Implement all arithmetic and comparison operators directly.

```csharp
// C# — already implemented in PramanaLib
public static GaussianRational operator +(GaussianRational a, GaussianRational b) => ...
public static bool operator ==(GaussianRational a, GaussianRational b) => ...
```

```python
# Python
def __add__(self, other): ...
def __eq__(self, other): ...
def __lt__(self, other): ...
def __hash__(self): ...
```

```rust
// Rust
impl std::ops::Add for GaussianRational { ... }
impl std::cmp::PartialEq for GaussianRational { ... }
impl std::cmp::PartialOrd for GaussianRational { ... }
```

### 7.2 Languages with Limited Operator Overloading

**Kotlin (JVM)** — Has operator functions. If a JVM variant is added, use Kotlin for the primary API.

### 7.3 Languages without Operator Overloading

**Java, JavaScript, TypeScript, Go** — Use named methods instead.

```java
// Java
GaussianRational result = a.add(b);
GaussianRational product = a.multiply(b);
boolean equal = a.equals(b);
int cmp = a.compareTo(b);
```

```typescript
// TypeScript
const result = a.add(b);
const product = a.mul(b);
const equal = a.eq(b);
const less = a.lt(b);
```

```go
// Go
result := a.Add(b)
product := a.Mul(b)
equal := a.Equal(b)
```

**Naming convention for method-based arithmetic:**

| Operation | Method Name |
|-----------|-------------|
| Addition | `add` |
| Subtraction | `sub` |
| Multiplication | `mul` |
| Division | `div` |
| Modulo | `mod` |
| Negation | `neg` |
| Power | `pow` |
| Equality | `eq` / `equals` |
| Less than | `lt` |
| Greater than | `gt` |
| Less or equal | `lte` |
| Greater or equal | `gte` |
| Compare | `compareTo` / `cmp` |

---

## 8. Serialization and Wire Format

### 8.1 JSON Serialization

All SDK types must serialize to/from JSON in a format compatible with `.pra` files:

```json
{
    "uuid": "00000007-0000-4000-8000-000000000007",
    "type": "Entity",
    "properties": {
        "EntityLabel": "Water"
    },
    "edges": {
        "instanceOf": "uuid-of-chemical-compound-class",
        "subclassOf": null
    }
}
```

For `GaussianRational` values in proposition `objectValue` fields, serialize as the canonical `num:` string.

### 8.2 Pramana URI Format

All items use the `pra:` URI prefix: `pra:00000007-0000-4000-8000-000000000007`

Struct pseudo-class values use their namespace prefix: `num:3,4,0,1`, `date:2026-02-28`

---

## 9. Evidence and Provenance

When `includeProvenance` is enabled in the configuration, every mapped field carries metadata about its source:

```python
# Python
shrine.coordinates           # → Coordinate(35.06, 135.75)
shrine.coordinates.pra_id    # → UUID of the Proposition asserting this value
shrine.coordinates.evidence  # → List of Evidence items supporting this proposition
shrine.coordinates.modality  # → "asserted" | "alleged" | "reported" | etc.
shrine.coordinates.valid_from  # → date or None
shrine.coordinates.valid_until # → date or None
```

This is optional and defaults to off for performance. When off, fields return plain values.

---

## 10. Computed Properties and Logic Rules

### 10.1 Concept

Some Pramana properties can have associated computation rules. For example:
- `age` = `today - date_of_birth`
- `molecular_weight` = sum of atomic weights of constituent atoms
- `is_prime` = primality test on integer value

### 10.2 Rule Definition

Rules are defined either:
- **In Pramana data** — as a formula/expression attached to the Property definition
- **In SDK code** — as a method on the mapped class (more common initially)

```python
@pramana_entity(instance_of="...")
class Person:
    date_of_birth: date

    @computed_property("age")
    def age(self) -> int:
        return (date.today() - self.date_of_birth).days // 365
```

### 10.3 Override Precedence

When both a computed value and a manually asserted proposition exist:
1. **Manual assertion wins** (returned by default accessor)
2. **Computed value available** via `computed_<property>()` method
3. **Conflict flagging** — SDK can optionally warn when computed and asserted values differ

### 10.4 Future: Logic from Pramana Data

Currently, computation rules live in SDK code. A future goal is to express rules in Pramana's data model itself (as formula-type propositions on Property definitions), and have the SDK evaluate them at runtime. This is analogous to stored procedures in databases. The SDK should reserve extension points for this.

---

## 11. Package Names and Distribution

| Language | Package Name | Registry | Min Version |
|----------|-------------|----------|-------------|
| C# | `PramanaLib` | NuGet | .NET 8.0+ |
| Python | `pramana` | PyPI | Python 3.11+ |
| TypeScript | `@pramana/core` | npm | Node 18+ / ES2020 |
| JavaScript | `@pramana/core` | npm | (same package as TS, ships JS + declarations) |
| Java | `org.pramana:pramana-core` | Maven Central | Java 17+ |
| Rust | `pramana` | crates.io | Rust 1.70+ |
| Go | `github.com/pramana-kg/pramana-go` | Go modules | Go 1.21+ |

---

## 12. Implementation Priority

### Phase 1 — GaussianRational Only (per language)
- [ ] Implement `GaussianRational` with exact arithmetic
- [ ] UUID v5 generation matching Pramana's `num:` namespace
- [ ] All operators / methods
- [ ] Parsing and formatting
- [ ] Number hierarchy classification
- [ ] Full test suite verifying cross-language consistency

### Phase 2 — Base Item Model
- [ ] `PramanaItem` base type with uuid, type, properties, edges
- [ ] `PramanaEntity`, `PramanaProperty`, `PramanaProposition`, `PramanaSense` typed wrappers
- [ ] JSON serialization to/from `.pra` format
- [ ] `.pra` file loading

### Phase 3 — ORM-style Mapping
- [ ] Annotation/decorator-based class mapping
- [ ] Configurable flattening depth
- [ ] Interface/trait mapping for multiple classification
- [ ] Lazy edge resolution
- [ ] Query interface

### Phase 4 — Data Sources and Provenance
- [ ] GraphDB SPARQL connector
- [ ] REST API connector
- [ ] Evidence/provenance tracking on fields
- [ ] Per-object overrides and computed properties

### Phase 5 — Additional Pseudo-Classes
- [ ] `date:` / `time:` / `interval:` pseudo-classes
- [ ] `coord:` struct
- [ ] `chem:` / `element:` pseudo-classes

---

## 13. Cross-Language Consistency Requirements

The following MUST produce identical results across all implementations:

1. **UUID v5 generation** — `GaussianRational(3, 4, 0, 1).pramana_id` must return the same UUID in every language
2. **Canonical string** — `num:3,4,0,1` is the same everywhere; normalization rules are language-independent
3. **Arithmetic results** — `GaussianRational(1,2,0,1) + GaussianRational(1,3,0,1)` must equal `GaussianRational(5,6,0,1)` everywhere
4. **JSON serialization** — The same `.pra` JSON must deserialize identically in all languages

A cross-language test suite (probably maintained as a JSON file of test vectors) should be used to verify conformance.

---

## 14. Test Vectors (Reference)

These test cases must pass in every implementation:

### GaussianRational Arithmetic
```
num:1,2,0,1 + num:1,3,0,1 = num:5,6,0,1        // 1/2 + 1/3 = 5/6
num:1,1,1,1 * num:0,1,1,1 = num:-1,1,0,1        // (1+i) * i = -1+i ... wait
num:0,1,1,1 * num:0,1,1,1 = num:-1,1,0,1        // i * i = -1
num:2,4,0,1 canonical = num:1,2,0,1              // 2/4 normalizes to 1/2
num:3,1,0,1 classify = "Natural Number"
num:0,1,0,1 classify = "Whole Number"
num:-5,1,0,1 classify = "Integer"
num:3,4,0,1 classify = "Rational Number"
num:1,1,1,1 classify = "Gaussian Rational"
```

### UUID v5 Consistency
```
num:5,1,0,1 → PramanaId must match across C#, Python, JS, Java, Rust, Go
num:0,1,0,1 → PramanaId must match across all languages
```

(Exact expected UUIDs should be computed from the C# reference implementation and documented.)

---

## Appendix A: Language-Specific BigInteger Availability

| Language | Native BigInt | Notes |
|----------|--------------|-------|
| C# | `System.Numerics.BigInteger` | Built-in, excellent |
| Python | `int` | Native arbitrary precision |
| JavaScript | `BigInt` | Native since ES2020 |
| TypeScript | `bigint` | Native type |
| Java | `java.math.BigInteger` | Built-in, immutable |
| Rust | `num::BigInt` | Via `num` crate (de facto standard) |
| Go | `math/big.Int` | Built-in, mutable (handle with care) |

## Appendix B: UUID v5 Algorithm

UUID v5 uses SHA-1 hashing with a namespace UUID:

1. Convert namespace UUID to 16 bytes (big-endian)
2. Concatenate with name bytes (UTF-8 encoded canonical string)
3. SHA-1 hash the result (20 bytes)
4. Set version to 5 (byte 6: `(hash[6] & 0x0F) | 0x50`)
5. Set variant to RFC 4122 (byte 8: `(hash[8] & 0x3F) | 0x80`)
6. Take first 16 bytes as the UUID

**Pramana `num:` namespace:** `a6613321-e9f6-4348-8f8b-29d2a3c86349`

The canonical name for `num:3,4,0,1` is the string `"num:3,4,0,1"` (after normalization).
