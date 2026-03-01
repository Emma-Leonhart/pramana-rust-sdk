use uuid::Uuid;

/// Trait for objects that can be linked to entities in the Pramana knowledge graph.
/// Provides identity and URL properties for graph integration.
///
/// This is the Rust equivalent of C#'s `IPramanaLinkable` interface.
pub trait PramanaLinkable {
    /// Returns the UUID (v4 or v5) identifying this entity in the Pramana graph.
    fn pramana_guid(&self) -> Uuid;

    /// Returns the Pramana identifier string (e.g. `"pra:num:3,1,2,1"`).
    /// Returns `None` for objects that are not pseudo-class instances.
    fn pramana_id(&self) -> Option<String> {
        None
    }

    /// Returns the Pramana entity URL using the hashed UUID,
    /// e.g. `"https://pramana.dev/entity/{guid}"`.
    fn pramana_hash_url(&self) -> String {
        format!("https://pramana.dev/entity/{}", self.pramana_guid())
    }

    /// Returns the Pramana entity URL. For pseudo-class instances this uses the
    /// `pramana_id` string; otherwise it falls back to `pramana_hash_url`.
    fn pramana_url(&self) -> String {
        self.pramana_hash_url()
    }
}
