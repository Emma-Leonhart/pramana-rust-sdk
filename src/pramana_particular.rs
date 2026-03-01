use uuid::Uuid;

use crate::error::PramanaResult;
use crate::pramana_interface::PramanaInterfaceTrait;
use crate::pramana_linkable::PramanaLinkable;
use crate::pramana_object::PramanaObject;
use crate::pramana_role::PramanaRole;

/// The well-known class ID for PramanaParticular in the ontology.
pub const PRAMANA_PARTICULAR_CLASS_ID: Uuid = Uuid::from_bytes([
    0x13, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04,
]);

/// A minimal subclass of `PramanaObject` used for the Pramana OGM class hierarchy.
///
/// In the C# SDK this is `PramanaParticular : PramanaObject`. In Rust we
/// compose a `PramanaObject` and delegate identity methods to it.
#[derive(Debug, Clone)]
pub struct PramanaParticular {
    inner: PramanaObject,
}

impl PramanaParticular {
    /// Creates a new `PramanaParticular` with no ID.
    pub fn new() -> Self {
        Self {
            inner: PramanaObject::new(),
        }
    }

    /// Creates a new `PramanaParticular` with the given ID.
    pub fn with_id(id: Uuid) -> Self {
        Self {
            inner: PramanaObject::with_id(id),
        }
    }

    /// The well-known class ID for PramanaParticular.
    pub fn class_id() -> Uuid {
        PRAMANA_PARTICULAR_CLASS_ID
    }

    /// The class-level URL in the Pramana graph.
    pub fn class_url() -> String {
        format!("https://pramana.dev/entity/{}", Self::class_id())
    }

    /// Assigns a UUID v4 to this object (delegates to inner object).
    pub fn generate_id(&mut self) -> PramanaResult<()> {
        self.inner.generate_id()
    }

    /// Returns a reference to the inner `PramanaObject`.
    pub fn inner(&self) -> &PramanaObject {
        &self.inner
    }
}

impl Default for PramanaParticular {
    fn default() -> Self {
        Self::new()
    }
}

impl PramanaLinkable for PramanaParticular {
    fn pramana_guid(&self) -> Uuid {
        self.inner.pramana_guid()
    }

    fn pramana_id(&self) -> Option<String> {
        None
    }
}

impl PramanaInterfaceTrait for PramanaParticular {
    fn get_roles(&self) -> Vec<&PramanaRole> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_id_has_expected_value() {
        assert_eq!(
            PramanaParticular::class_id(),
            Uuid::parse_str("13000000-0000-4000-8000-000000000004").unwrap()
        );
    }

    #[test]
    fn class_url_uses_class_id() {
        assert_eq!(
            PramanaParticular::class_url(),
            format!(
                "https://pramana.dev/entity/{}",
                PramanaParticular::class_id()
            )
        );
    }

    #[test]
    fn default_has_nil_guid() {
        let obj = PramanaParticular::new();
        assert_eq!(obj.pramana_guid(), Uuid::nil());
    }

    #[test]
    fn generate_id_works() {
        let mut obj = PramanaParticular::new();
        obj.generate_id().unwrap();
        assert_ne!(obj.pramana_guid(), Uuid::nil());
    }

    #[test]
    fn generate_id_errors_on_second_call() {
        let mut obj = PramanaParticular::new();
        obj.generate_id().unwrap();
        assert!(obj.generate_id().is_err());
    }
}
