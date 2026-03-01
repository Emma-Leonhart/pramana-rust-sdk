use uuid::Uuid;

use crate::error::PramanaResult;
use crate::pramana_linkable::PramanaLinkable;
use crate::pramana_object::PramanaObject;

/// Represents a role (interface) in the Pramana ontology.
/// Roles form a hierarchy via `subclass_of` and `instance_of`,
/// and track their position in the role graph through
/// `parent_roles` and `child_roles`.
///
/// In Rust, since we don't have class inheritance, `PramanaRole` composes
/// a `PramanaObject` internally and delegates identity methods to it.
#[derive(Debug, Clone)]
pub struct PramanaRole {
    inner: PramanaObject,
    label: String,
    instance_of: Option<Box<PramanaRole>>,
    subclass_of: Option<Box<PramanaRole>>,
    parent_roles: Vec<PramanaRole>,
    child_roles: Vec<PramanaRole>,
}

impl PramanaRole {
    /// Creates a new `PramanaRole` with the given label and no ID.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            inner: PramanaObject::new(),
            label: label.into(),
            instance_of: None,
            subclass_of: None,
            parent_roles: Vec::new(),
            child_roles: Vec::new(),
        }
    }

    /// Creates a new `PramanaRole` with the given label and ID.
    pub fn with_id(label: impl Into<String>, id: Uuid) -> Self {
        Self {
            inner: PramanaObject::with_id(id),
            label: label.into(),
            instance_of: None,
            subclass_of: None,
            parent_roles: Vec::new(),
            child_roles: Vec::new(),
        }
    }

    /// Returns the human-readable label for this role.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Sets the label.
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    /// Returns a reference to the inner `PramanaObject`.
    pub fn inner(&self) -> &PramanaObject {
        &self.inner
    }

    /// Assigns a UUID v4 to this role (delegates to inner object).
    pub fn generate_id(&mut self) -> PramanaResult<()> {
        self.inner.generate_id()
    }

    // ── Hierarchy ──────────────────────────────────────────────

    /// Returns the role that this role is an instance of, if any.
    pub fn instance_of(&self) -> Option<&PramanaRole> {
        self.instance_of.as_deref()
    }

    /// Sets the role that this role is an instance of.
    pub fn set_instance_of(&mut self, role: PramanaRole) {
        self.instance_of = Some(Box::new(role));
    }

    /// Returns the role that this role is a subclass of, if any.
    pub fn subclass_of(&self) -> Option<&PramanaRole> {
        self.subclass_of.as_deref()
    }

    /// Sets the role that this role is a subclass of.
    pub fn set_subclass_of(&mut self, role: PramanaRole) {
        self.subclass_of = Some(Box::new(role));
    }

    /// Returns the parent roles of this role in the hierarchy.
    pub fn parent_roles(&self) -> &[PramanaRole] {
        &self.parent_roles
    }

    /// Adds a parent role.
    pub fn add_parent_role(&mut self, role: PramanaRole) {
        self.parent_roles.push(role);
    }

    /// Returns the child roles of this role in the hierarchy.
    pub fn child_roles(&self) -> &[PramanaRole] {
        &self.child_roles
    }

    /// Adds a child role.
    pub fn add_child_role(&mut self, role: PramanaRole) {
        self.child_roles.push(role);
    }
}

impl PramanaLinkable for PramanaRole {
    fn pramana_guid(&self) -> Uuid {
        self.inner.pramana_guid()
    }

    fn pramana_id(&self) -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_sets_label() {
        let role = PramanaRole::new("Entity");
        assert_eq!(role.label(), "Entity");
    }

    #[test]
    fn with_id_sets_guid() {
        let id = Uuid::new_v4();
        let role = PramanaRole::with_id("Entity", id);
        assert_eq!(role.pramana_guid(), id);
    }

    #[test]
    fn without_id_has_nil_guid() {
        let role = PramanaRole::new("Entity");
        assert_eq!(role.pramana_guid(), Uuid::nil());
    }

    #[test]
    fn instance_of_defaults_to_none() {
        let role = PramanaRole::new("Entity");
        assert!(role.instance_of().is_none());
    }

    #[test]
    fn subclass_of_defaults_to_none() {
        let role = PramanaRole::new("Entity");
        assert!(role.subclass_of().is_none());
    }

    #[test]
    fn parent_roles_initially_empty() {
        let role = PramanaRole::new("Entity");
        assert!(role.parent_roles().is_empty());
    }

    #[test]
    fn child_roles_initially_empty() {
        let role = PramanaRole::new("Entity");
        assert!(role.child_roles().is_empty());
    }

    #[test]
    fn can_build_role_hierarchy() {
        let parent = PramanaRole::new("Thing");
        let child_label = "Person";

        let mut child = PramanaRole::new(child_label);
        child.set_subclass_of(parent.clone());
        child.add_parent_role(parent.clone());

        assert_eq!(child.subclass_of().unwrap().label(), "Thing");
        assert_eq!(child.parent_roles().len(), 1);
        assert_eq!(child.parent_roles()[0].label(), "Thing");
    }

    #[test]
    fn instance_of_can_be_set() {
        let class_role = PramanaRole::new("Class");
        let mut instance = PramanaRole::new("MyClass");
        instance.set_instance_of(class_role);
        assert_eq!(instance.instance_of().unwrap().label(), "Class");
    }

    #[test]
    fn generate_id_works() {
        let mut role = PramanaRole::new("Entity");
        role.generate_id().unwrap();
        assert_ne!(role.pramana_guid(), Uuid::nil());
    }

    #[test]
    fn generate_id_errors_on_second_call() {
        let mut role = PramanaRole::new("Entity");
        role.generate_id().unwrap();
        assert!(role.generate_id().is_err());
    }
}
