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

/// Type-safe action mapping generator that ensures module names are valid
/// This macro provides compile-time validation of module names and discriminant/action type matching
#[macro_export]
macro_rules! map_action {
    ($module:ident, $discriminant:expr, $action_type:ty) => {{
        // Normalize identifier-style module names to kebab-case for action strings.
        const MODULE_NAME: &'static str = stringify!($module);
        let module_str = MODULE_NAME.replace('_', "-");

        let entity_str = $discriminant.to_string();
        <$action_type as strum::VariantArray>::VARIANTS
            .iter()
            .map(|variant| {
                ActionMapping::new(&module_str, &entity_str, variant, variant.permission_set())
            })
            .collect::<Vec<ActionMapping>>()
    }};
}
