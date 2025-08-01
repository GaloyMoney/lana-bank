use std::fmt::{Debug, Display};

/// Marker for actions with no path segment provided.
#[derive(Clone)]
pub struct NoPath;

/// Marker for actions with both module and entity name provided.
#[derive(Clone)]
pub struct FullPath(String, String);

/// Description of a defined action. Each description consists
/// of a portion of its path that was known during its construction,
/// name of the action and assigned permission sets.
///
/// To obtain full name of the action and its related object, both
/// segments of the path – module and entity – need to be present (i. e. `P = FullPath`).
#[derive(Clone)]
pub struct ActionDescription<P: Clone> {
    path: P,
    name: String,
    permission_sets: Vec<&'static str>,
}

impl<P: Clone> ActionDescription<P> {
    pub fn permission_sets(&self) -> &[&'static str] {
        &self.permission_sets
    }
}

impl ActionDescription<NoPath> {
    pub fn new<D: core::fmt::Display, const N: usize>(
        name: D,
        permission_sets: &'static [&'static str; N],
    ) -> Self {
        Self {
            path: NoPath,
            name: name.to_string(),
            permission_sets: permission_sets.to_vec(),
        }
    }

    /// Simplified constructor that takes a single permission instead of an array.
    /// This is the new approach where hierarchy is handled at role assignment time.
    pub fn new2<D: core::fmt::Display>(name: D, permission_set: &'static str) -> Self {
        Self {
            path: NoPath,
            name: name.to_string(),
            permission_sets: vec![permission_set],
        }
    }
}

impl ActionDescription<NoPath> {
    /// Returns new action derived from this action with `module_name` and `entity_name`
    /// added to its path.
    pub fn inject_path<M: Display, E: Display>(
        self,
        module_name: M,
        entity_name: E,
    ) -> ActionDescription<FullPath> {
        ActionDescription {
            path: FullPath(module_name.to_string(), entity_name.to_string()),
            name: self.name,
            permission_sets: self.permission_sets,
        }
    }
}

impl ActionDescription<FullPath> {
    /// Returns full name of this action, including module and entity
    /// to which the action belongs. The format is `module:entity:action`.
    pub fn action_name(&self) -> String {
        format!("{}:{}:{}", self.path.0, self.path.1, self.name)
    }

    /// Returns full name of this action's object with catch-all reference `*`.
    /// The format is `module/entity/*`.
    pub fn all_objects_name(&self) -> String {
        format!("{}/{}/*", self.path.0, self.path.1)
    }
}

impl Debug for ActionDescription<FullPath> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} -> {} : {:?}",
            self.all_objects_name(),
            self.action_name(),
            self.permission_sets
        )
    }
}
