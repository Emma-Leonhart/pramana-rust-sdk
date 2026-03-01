//! # Pramana SDK
//!
//! Exact Gaussian rational arithmetic and deterministic UUID v5 identity
//! for the [Pramana knowledge graph](https://pramana-data.ca).
//!
//! ## Core types
//!
//! - [`Gauss`] — Gaussian rational (`Q[i]`): `A/B + (C/D)i` with
//!   arbitrary-precision components.
//! - [`Gint`]  — Gaussian integer (`Z[i]`): a subset of `Gauss` where
//!   both denominators are 1.
//!
//! ## Quick start
//!
//! ```rust
//! use pramana_sdk::{Gauss, Gint};
//!
//! let z = Gint::new(3, 4);           // 3 + 4i
//! let w = Gauss::new(1, 2, 3, 4);    // 1/2 + 3/4 i
//!
//! println!("{}", z);                  // "3 + 4i"
//! println!("{}", w);                  // "1/2 + 3/4i"
//! println!("{}", z.pramana_id());     // deterministic UUID v5
//! ```

pub mod error;
pub mod gauss;
pub mod gint;
pub mod number_theory;
pub mod pramana_id;

// Re-export main types at crate root.
pub use error::{PramanaError, PramanaResult};
pub use gauss::Gauss;
pub use gint::Gint;
pub use number_theory::is_prime;
pub use pramana_id::PRAMANA_NAMESPACE;

/// Mathematical alias for [`Gint`] (the ring **Z**\[i\]).
pub type Zi = Gint;

/// Mathematical alias for [`Gauss`] (the field **Q**\[i\]).
pub type Qi = Gauss;
