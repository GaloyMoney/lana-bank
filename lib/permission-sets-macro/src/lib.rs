// Re-export paste so the macro can use it
#[doc(hidden)]
pub use paste;

// Re-export linkme so the macro can use it
#[doc(hidden)]
pub use linkme;

/// An entry registered at program start for each permission set variant.
pub struct PermissionSetEntry {
    pub name: &'static str,
    pub description: &'static str,
}

#[linkme::distributed_slice]
pub static PERMISSION_SET_ENTRIES: [PermissionSetEntry];

/// Look up a permission set entry by name.
pub fn find_by_name(name: &str) -> Option<&'static PermissionSetEntry> {
    PERMISSION_SET_ENTRIES.iter().find(|e| e.name == name)
}

/// Iterate over all registered permission set entries.
pub fn all_entries() -> impl Iterator<Item = &'static PermissionSetEntry> {
    PERMISSION_SET_ENTRIES.iter()
}

/// Declarative macro for defining permission sets.
///
/// List your permission variant names with descriptions - everything else is auto-derived!
///
/// # Usage
///
/// ```rust
/// use permission_sets_macro::permission_sets;
///
/// permission_sets! {
///     Viewer("Can view resources"),
///     Writer("Can create and manage resources"),
/// }
/// ```
///
/// From a crate named `core-custody`, this generates:
/// - `pub const PERMISSION_SET_CUSTODY_VIEWER: &str = "custody_viewer";`
/// - `pub const PERMISSION_SET_CUSTODY_WRITER: &str = "custody_writer";`
///
/// Each variant is also registered via `linkme` for runtime discovery.
///
/// ## Naming Rules:
/// - Crate `core-custody` → module prefix `CUSTODY` → string prefix `custody_`
/// - Variant `Viewer` → enum variant `CustodyViewer` → string `custody_viewer`
#[macro_export]
macro_rules! permission_sets {
    ( $( $variant:ident ($description:literal) ),* $(,)? ) => {
        $(
            $crate::paste::paste! {
                #[doc = concat!("Permission set: ", stringify!($variant))]
                pub const [<PERMISSION_SET_ $variant:snake:upper>]: &str = stringify!([<$variant:snake>]);

                #[$crate::linkme::distributed_slice($crate::PERMISSION_SET_ENTRIES)]
                #[linkme(crate = $crate::linkme)]
                static [<__PERM_ENTRY_ $variant:snake:upper>]: $crate::PermissionSetEntry = $crate::PermissionSetEntry {
                    name: [<PERMISSION_SET_ $variant:snake:upper>],
                    description: $description,
                };
            }
        )*
    };
}
