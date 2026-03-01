use uuid::Uuid;

/// The fixed Pramana namespace UUID used for all UUID v5 generation.
pub const PRAMANA_NAMESPACE: Uuid = Uuid::from_bytes([
    0xa6, 0x61, 0x33, 0x21, 0xe9, 0xf6, 0x43, 0x48, 0x8f, 0x8b, 0x29, 0xd2, 0xa3, 0xc8, 0x63, 0x49,
]);

/// Generate a deterministic Pramana UUID v5 from a canonical key string.
pub fn pramana_uuid(key: &str) -> Uuid {
    Uuid::new_v5(&PRAMANA_NAMESPACE, key.as_bytes())
}

/// Build the canonical Pramana label string: `"pra:num:A,B,C,D"`.
pub fn pramana_label(key: &str) -> String {
    format!("pra:num:{key}")
}

/// Build the Pramana entity URL from a UUID.
pub fn pramana_url(id: &Uuid) -> String {
    format!("https://pramana-data.ca/entity/{id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace_uuid() {
        assert_eq!(
            PRAMANA_NAMESPACE.to_string(),
            "a6613321-e9f6-4348-8f8b-29d2a3c86349"
        );
    }

    #[test]
    fn test_deterministic_uuid() {
        let id1 = pramana_uuid("1,1,0,1");
        let id2 = pramana_uuid("1,1,0,1");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_different_keys_different_uuids() {
        let id1 = pramana_uuid("1,1,0,1");
        let id2 = pramana_uuid("2,1,0,1");
        assert_ne!(id1, id2);
    }
}
