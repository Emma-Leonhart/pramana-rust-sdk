use crate::pramana_role::PramanaRole;

/// Trait that all Pramana-mapped objects implement, providing
/// access to the ontology roles (interfaces) the object participates in.
///
/// This is the Rust equivalent of C#'s `PramanaInterface`.
pub trait PramanaInterfaceTrait {
    /// Returns the `PramanaRole` instances that this object fulfils
    /// within the Pramana ontology.
    fn get_roles(&self) -> Vec<&PramanaRole>;
}
