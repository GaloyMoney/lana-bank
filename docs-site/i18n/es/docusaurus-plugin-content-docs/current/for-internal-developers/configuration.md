---
id: configuration
title: Configuración de dominio
sidebar_position: 5
---

# Configuración de Dominio

La Configuración de Dominio proporciona almacenamiento de configuración persistente y con seguridad de tipos con dos niveles de visibilidad.

## Tipos Compatibles

Tipos simples: `bool`, `i64`, `u64`, `String`, `Decimal`, `Timezone`, `Time`.

Structs complejos (solo configuraciones internas): Cualquier struct que implemente `Serialize` y `Deserialize`.

## Niveles de Visibilidad

### Configuraciones Internas

Las configuraciones internas deben ser completamente propiedad de otro crate principal. Esto significa que el crate propietario debe ser el único que lea y actualice la configuración, y define sus propias reglas de autorización específicas para esa configuración. Sin embargo, el crate domain-config sigue siendo propietario de la persistencia. El punto es simplemente que el crate "propietario" debe ser el único código que interactúe directamente con esta configuración interna.

Los temas relacionados con la interfaz de usuario para las configuraciones internas deben ser gestionados directamente por el crate que las posee, ya que las configuraciones internas no aparecen en la página genérica de "Configuraciones".

Las configuraciones internas admiten tanto tipos simples como structs complejos.

### Configuraciones Expuestas

Las configuraciones expuestas aparecen automáticamente en la página de Configuraciones de la aplicación de administración para usuarios autorizados. Los roles requeridos para leer y escribir estas configuraciones son:

- `PERMISSION_SET_EXPOSED_CONFIG_VIEWER`
- `PERMISSION_SET_EXPOSED_CONFIG_WRITER`

Utilice configuraciones expuestas para ajustes generales que no requieran lógica de autorización personalizada.

Las configuraciones expuestas solo admiten tipos simples.

## Ciclo de Vida de la Configuración

### Registro

Las configuraciones se definen utilizando las macros `define_internal_config!` o `define_exposed_config!`. Cada configuración especifica una clave única y, opcionalmente, un valor predeterminado y una función de validación.

### Inicialización

El uso de las macros `define_internal_config!` o `define_exposed_config!` registra automáticamente su configuración para su inicialización. Al iniciar la aplicación, todas las configuraciones registradas se inicializan en la base de datos. Los desarrolladores que definen nuevas configuraciones no necesitan llamar manualmente a ninguna función de inicialización; simplemente use la macro y la configuración estará disponible. Debido a esta inicialización automática, `get` siempre tiene éxito para todas las configuraciones.

### Lectura de valores

Para leer una configuración, llama a `get::<YourConfig>()` en el servicio correspondiente:

```rust
// Internal config (enforce your own authorization before this call)
let typed_config: TypedDomainConfig<MyConfig> = internal_configs.get::<MyConfig>().await?;

// Exposed config (requires auth subject)
let typed_config: TypedDomainConfig<MyConfig> = exposed_configs.get::<MyConfig>(&subject).await?;
```

El método `get()` devuelve un wrapper `TypedDomainConfig<C>`. La forma de acceder al valor depende de si la configuración tiene un valor predeterminado:

**Para configuraciones CON valores predeterminados** (definidas con una cláusula `default:`), usa `.value()`:

```rust
// Returns T directly - always succeeds because the default guarantees a value
let value: bool = typed_config.value();
```

**Para configuraciones SIN valores predeterminados**, usa `.maybe_value()`:

```rust
// Returns Option<T>
let value: Option<String> = typed_config.maybe_value();
```

- `Some(value)` - la configuración se ha establecido explícitamente mediante `update`
- `None` - no existe ningún valor

El llamador no necesita saber si el valor provino de una entrada explícita en la base de datos o del valor predeterminado definido durante el registro.

### Acceso de solo lectura para consumidores internos

Para trabajos en segundo plano y procesos internos que necesitan leer configuraciones expuestas sin contexto de usuario, utiliza `ExposedDomainConfigsReadOnly`:

```rust
let readonly_configs = ExposedDomainConfigsReadOnly::new(&pool);
let typed_config = readonly_configs.get_without_audit::<MyConfig>().await?;
```

Este servicio proporciona acceso de solo lectura a las configuraciones expuestas sin requerir un sujeto de autorización. Utiliza este patrón cuando:

- Los trabajos en segundo plano necesiten valores de configuración durante la ejecución
- Los procesos internos operen sin contexto de usuario
- Necesites evitar la sobrecarga de autorización para acceso interno de solo lectura

El servicio de solo lectura únicamente admite lectura; las actualizaciones de configuración aún requieren el servicio estándar `ExposedDomainConfigs` con la autorización adecuada.
