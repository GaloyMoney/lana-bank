use std::fmt::Display;

/// Trait for action enums to provide their permission set
pub trait ActionPermission {
    fn permission_set(&self) -> &'static str;
}

/// Simple action mapping - just the essentials!
#[derive(Clone, Debug)]
pub struct ActionMapping {
    pub full_action_name: String,     // "access:user:create"
    pub object_name: String,          // "access/user/*"
    pub permission_set: &'static str, // "access_writer"
}

impl ActionMapping {
    /// Create a complete action mapping with all context
    pub fn new<M: Display, E: Display, A: Display>(
        module: M,
        entity: E,
        action: A,
        permission_set: &'static str,
    ) -> Self {
        let module_str = module.to_string();
        let entity_str = entity.to_string();
        let action_str = action.to_string();

        Self {
            full_action_name: format!("{module_str}:{entity_str}:{action_str}"),
            object_name: format!("{module_str}/{entity_str}/*"),
            permission_set,
        }
    }

    /// Returns the permission set for this action
    pub fn permission_set(&self) -> &'static str {
        self.permission_set
    }

    /// Returns full action name: "module:entity:action"
    pub fn action_name(&self) -> &str {
        &self.full_action_name
    }

    /// Returns object name: "module/entity/*"
    pub fn all_objects_name(&self) -> &str {
        &self.object_name
    }
}

/// Helper to generate action mappings from enum variants  
pub fn generate_action_mappings<T, M: Display, E: Display>(
    module: M,
    entity: E,
    variants: &[T],
) -> Vec<ActionMapping>
where
    T: ActionPermission + Display + Clone,
{
    variants
        .iter()
        .map(|variant| ActionMapping::new(&module, &entity, variant, variant.permission_set()))
        .collect()
}

/// Auto-derive module and entity names from discriminant
pub fn auto_generate_action_mappings<T, D>(
    module_name: &str,
    entity_discriminant: D,
    variants: &[T],
) -> Vec<ActionMapping>
where
    T: ActionPermission + Display + Clone,
    D: Display,
{
    let entity_name = entity_discriminant.to_string();
    generate_action_mappings(module_name, entity_name, variants)
}

/// Trait for modules to provide their name
pub trait ModuleName {
    const MODULE_NAME: &'static str;
}

/// Fully auto-derived action mappings - no hardcoding needed!
pub fn fully_auto_generate_action_mappings<T, D, M>(
    entity_discriminant: D,
    variants: &[T],
) -> Vec<ActionMapping>
where
    T: ActionPermission + Display + Clone,
    D: Display,
    M: ModuleName,
{
    auto_generate_action_mappings(M::MODULE_NAME, entity_discriminant, variants)
}

/// Simplified version - just takes module name directly
pub fn simple_auto_mappings<T>(
    module: &str,
    entity: impl Display,
    variants: &[T],
) -> Vec<ActionMapping>
where
    T: ActionPermission + Display + Clone,
{
    generate_action_mappings(module, entity, variants)
}

/// Ultra-clean macro for generating action mappings
/// Uses the ModuleName trait from Self to automatically derive the module name
#[macro_export]
macro_rules! auto_mappings {
    ($entity:expr => $action_type:ty) => {
        $crate::action_description::simple_auto_mappings(
            Self::MODULE_NAME,
            $entity,
            <$action_type as strum::VariantArray>::VARIANTS,
        )
    };
}

// Type alias for consistency across codebase
pub type ActionDescription = ActionMapping;
