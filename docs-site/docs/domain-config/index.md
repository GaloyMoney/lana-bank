---
id: index
title: Domain Config
sidebar_position: 1
---

# Domain Config

Domain Config provides type-safe, persistent configuration storage with two visibility levels.

## Supported Types

Simple types: `bool`, `i64`, `u64`, `String`, `Decimal`.

Complex structs (internal configs only): Any struct implementing `Serialize` and `Deserialize`.

## Visibility Levels

### Internal Configs

Internal configs should be fully owned by another core crate. That means that the owning crate should be the only one to read and update the config, and defines its own authorization rules specific to that config. However, the domain-config crate still owns the persistence. The point is just that the "owner" crate should be the only code which interacts with this internal config directly.

UI-related topics for internal configs need to be managed by the crate that owns them directly, as internal configs do not appear in the generic "Configurations" page.

Internal configs support both simple types and complex structs.

### Exposed Configs

Exposed configs automatically appear in the admin app's Configurations page for authorized users. The roles required for reading and writing these configs are:

- `PERMISSION_SET_EXPOSED_CONFIG_VIEWER`
- `PERMISSION_SET_EXPOSED_CONFIG_WRITER`

Use exposed configs for general settings that don't require custom authorization logic.

Exposed configs only support simple types.

## Config Lifecycle

### Registration

Configs are defined using the `define_internal_config!` or `define_exposed_config!` macros. Each config specifies a unique key and optionally a default value and validation function.

### Seeding

Using the `define_internal_config!` or `define_exposed_config!` macros automatically registers your config for seeding. At application startup, all registered configs are seeded to the database. Developers defining new configs do not need to call any seeding functions manually - just use the macro and the config will be available. Because of this automatic seeding, `get` always succeeds for all configs.

### Reading Values

To read a config, call `get::<YourConfig>()` on the appropriate service:

```rust
// Internal config (enforce your own authorization before this call)
let typed_config: TypedDomainConfig<MyConfig> = internal_configs.get::<MyConfig>().await?;

// Exposed config (requires auth subject)
let typed_config: TypedDomainConfig<MyConfig> = exposed_configs.get::<MyConfig>(&subject).await?;
```

The `get()` method returns a `TypedDomainConfig<C>` wrapper. Call `.value()` on it to get the resolved value as a standard `Option<T>`:

- `Some(value)` - the resolved value (either from the database or the default)
- `None` - no value exists and no default was defined

The caller doesn't need to know whether the value came from an explicit database entry or from the default defined at registration.
