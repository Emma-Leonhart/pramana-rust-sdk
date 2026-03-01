use uuid::Uuid;

use crate::error::{PramanaError, PramanaResult};
use crate::pramana_interface::PramanaInterfaceTrait;
use crate::pramana_linkable::PramanaLinkable;
use crate::pramana_role::PramanaRole;

/// The well-known root ID for the PramanaObject class itself in the ontology.
pub const PRAMANA_OBJECT_ROOT_ID: Uuid = Uuid::from_bytes([
    0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
]);

/// Base type for all objects mapped into the Pramana knowledge graph.
/// Implements [`PramanaLinkable`] for graph identity and
/// [`PramanaInterfaceTrait`] for ontology role participation.
///
/// **Friction by design:** IDs are never auto-generated. A new `PramanaObject`
/// starts with `Uuid::nil()` and only receives a real UUID v4 when
/// [`generate_id()`](PramanaObject::generate_id) is explicitly called. This prevents
/// disposable or transient objects from polluting the graph with throw-away
/// identifiers. Once assigned, the ID is immutable — calling `generate_id()` a
/// second time returns an error.
#[derive(Debug, Clone)]
pub struct PramanaObject {
    guid: Uuid,
}

impl PramanaObject {
    /// Creates a new `PramanaObject` with no ID (starts as `Uuid::nil()`).
    pub fn new() -> Self {
        Self { guid: Uuid::nil() }
    }

    /// Creates a new `PramanaObject` with the given ID.
    pub fn with_id(id: Uuid) -> Self {
        Self { guid: id }
    }

    /// The well-known root ID for the PramanaObject class.
    pub fn root_id() -> Uuid {
        PRAMANA_OBJECT_ROOT_ID
    }

    /// The class-level ID (same as `root_id()` for `PramanaObject`).
    pub fn class_id() -> Uuid {
        PRAMANA_OBJECT_ROOT_ID
    }

    /// The class-level URL in the Pramana graph.
    pub fn class_url() -> String {
        format!("https://pramana.dev/entity/{}", Self::class_id())
    }

    /// Assigns a new UUID v4 to this object.
    ///
    /// Returns an error if the object already has a non-nil ID — IDs are
    /// write-once by design.
    pub fn generate_id(&mut self) -> PramanaResult<()> {
        if self.guid == Uuid::nil() {
            self.guid = Uuid::new_v4();
            Ok(())
        } else {
            Err(PramanaError::IdAlreadyAssigned)
        }
    }

    /// Returns a reference to the internal GUID.
    pub fn guid(&self) -> Uuid {
        self.guid
    }
}

impl Default for PramanaObject {
    fn default() -> Self {
        Self::new()
    }
}

impl PramanaLinkable for PramanaObject {
    fn pramana_guid(&self) -> Uuid {
        self.guid
    }

    fn pramana_id(&self) -> Option<String> {
        None
    }
}

impl PramanaInterfaceTrait for PramanaObject {
    fn get_roles(&self) -> Vec<&PramanaRole> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_nil_guid() {
        let obj = PramanaObject::new();
        assert_eq!(obj.pramana_guid(), Uuid::nil());
    }

    #[test]
    fn with_id_sets_guid() {
        let id = Uuid::new_v4();
        let obj = PramanaObject::with_id(id);
        assert_eq!(obj.pramana_guid(), id);
    }

    #[test]
    fn generate_id_assigns_non_nil() {
        let mut obj = PramanaObject::new();
        obj.generate_id().unwrap();
        assert_ne!(obj.pramana_guid(), Uuid::nil());
    }

    #[test]
    fn generate_id_errors_on_second_call() {
        let mut obj = PramanaObject::new();
        obj.generate_id().unwrap();
        assert_eq!(obj.generate_id(), Err(PramanaError::IdAlreadyAssigned));
    }

    #[test]
    fn generate_id_errors_when_constructed_with_id() {
        let mut obj = PramanaObject::with_id(Uuid::new_v4());
        assert_eq!(obj.generate_id(), Err(PramanaError::IdAlreadyAssigned));
    }

    #[test]
    fn pramana_id_is_none() {
        let obj = PramanaObject::new();
        assert!(obj.pramana_id().is_none());
    }

    #[test]
    fn hash_url_contains_guid() {
        let id = Uuid::new_v4();
        let obj = PramanaObject::with_id(id);
        assert_eq!(
            obj.pramana_hash_url(),
            format!("https://pramana.dev/entity/{id}")
        );
    }

    #[test]
    fn pramana_url_equals_hash_url() {
        let obj = PramanaObject::with_id(Uuid::new_v4());
        assert_eq!(obj.pramana_url(), obj.pramana_hash_url());
    }

    #[test]
    fn class_id_equals_root_id() {
        assert_eq!(PramanaObject::root_id(), PramanaObject::class_id());
    }

    #[test]
    fn root_id_has_expected_value() {
        assert_eq!(
            PramanaObject::root_id(),
            Uuid::parse_str("10000000-0000-4000-8000-000000000001").unwrap()
        );
    }

    #[test]
    fn class_url_uses_class_id() {
        assert_eq!(
            PramanaObject::class_url(),
            format!("https://pramana.dev/entity/{}", PramanaObject::class_id())
        );
    }

    #[test]
    fn get_roles_returns_empty() {
        let obj = PramanaObject::new();
        assert!(obj.get_roles().is_empty());
    }
}
