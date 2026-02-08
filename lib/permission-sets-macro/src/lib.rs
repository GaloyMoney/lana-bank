// Re-export paste so the macro can use it
#[doc(hidden)]
pub use paste;

// Re-export inventory so the macro can use it
#[doc(hidden)]
pub use inventory;

/// An entry registered at program start for each permission set variant.
pub struct PermissionSetEntry {
    pub name: &'static str,
}

inventory::collect!(PermissionSetEntry);

/// Declarative macro for defining permission sets.
///
/// Just list your permission variant names - everything else is auto-derived!
///
/// # Usage
///
/// ```rust
/// use permission_sets_macro::permission_sets;
///
/// permission_sets! {
///     Viewer,
///     Writer,
/// }
/// ```
///
/// From a crate named `core-custody`, this generates:
/// - `pub const PERMISSION_SET_CUSTODY_VIEWER: &str = "custody_viewer";`
/// - `pub const PERMISSION_SET_CUSTODY_WRITER: &str = "custody_writer";`
///
/// Each variant is also registered via `inventory` for runtime discovery.
///
/// ## Naming Rules:
/// - Crate `core-custody` → module prefix `CUSTODY` → string prefix `custody_`
/// - Variant `Viewer` → enum variant `CustodyViewer` → string `custody_viewer`
#[macro_export]
macro_rules! permission_sets {
    ( $( $variant:ident ),* $(,)? ) => {
        $(
            $crate::paste::paste! {
                #[doc = concat!("Permission set: ", stringify!($variant))]
                pub const [<PERMISSION_SET_ $variant:snake:upper>]: &str = stringify!([<$variant:snake>]);

                $crate::inventory::submit! {
                    $crate::PermissionSetEntry {
                        name: [<PERMISSION_SET_ $variant:snake:upper>],
                    }
                }
            }
        )*
    };
}
