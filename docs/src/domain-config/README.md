# Domain Config

Domain Config provides type-safe, persistent configuration storage with two visibility levels.

## Visibility Levels

### Internal Configs

Internal configs are fully owned by another core crate. The owning crate exposes methods to read and update the config, and defines its own authorization rules specific to that config.

UI related topics for internal configs needs to be manage by the crate that owns it directly, as internal config do not appear in the generic "Configurations" page.

Internal configs support both simple types and complex structs.

### Exposed Configs

Exposed configs automatically appear in the admin app's Configurations page for authorized users. They use standard domain-config authorization (viewer/writer permission sets).

Use exposed configs for general settings that don't require custom authorization logic.

Exposed configs only support simple types.

## Supported Types

Simple types: `bool`, `i64`, `u64`, `String`, `Decimal`.

Complex structs (internal configs only): Any struct implementing `Serialize` and `Deserialize`.

## Config Lifecycle

### Registration

Configs are defined using the `define_internal_config!` or `define_exposed_config!` macros. Each config specifies a unique key and optionally a default value and validation function.

### Seeding

All registered configs are seeded at application startup via `seed_registered()`. This creates database entries for any configs that don't yet exist. Because of seeding, fetching a config always succeeds.

### Reading Values

When reading a config, the `value()` method returns an `Option`:

- `Some(value)` if the config has been explicitly set
- `Some(default)` if no value is set but a default was specified
- `None` if no value is set and no default exists
