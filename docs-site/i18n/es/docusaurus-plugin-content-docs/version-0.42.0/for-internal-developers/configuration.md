---
id: configuration
title: Configuración de Dominio
sidebar_position: 5
---

# Configuración de Dominio

La Configuración de Dominio proporciona almacenamiento de configuración persistente y con tipado seguro, con dos niveles de visibilidad.

## Tipos Soportados

Tipos simples: `bool`, `i64`, `u64`, `String`, `Decimal`.

Estructuras complejas (solo configuraciones internas): Cualquier estructura que implemente `Serialize` y `Deserialize`.

## Niveles de Visibilidad

### Configuraciones Internas

Las configuraciones internas deben ser propiedad completa de otro crate del núcleo. Esto significa que el crate propietario debe ser el único en leer y actualizar la configuración, y define sus propias reglas de autorización específicas para esa configuración. Sin embargo, el crate domain-config sigue siendo el propietario de la persistencia. El punto es que el crate "propietario" debe ser el único código que interactúa directamente con esta configuración interna.

Los temas relacionados con la interfaz de usuario para las configuraciones internas deben ser gestionados directamente por el crate que las posee, ya que las configuraciones internas no aparecen en la página genérica de "Configuraciones".

Las configuraciones internas soportan tanto tipos simples como estructuras complejas.

### Configuraciones Expuestas

Las configuraciones expuestas aparecen automáticamente en la página de Configuraciones de la aplicación de administración para usuarios autorizados. Los roles requeridos para leer y escribir estas configuraciones son:

- `PERMISSION_SET_EXPOSED_CONFIG_VIEWER`
- `PERMISSION_SET_EXPOSED_CONFIG_WRITER`

Use configuraciones expuestas para ajustes generales que no requieren lógica de autorización personalizada.

Las configuraciones expuestas solo soportan tipos simples.

## Ciclo de Vida de la Configuración

### Registro

Las configuraciones se definen usando los macros `define_internal_config!` o `define_exposed_config!`. Cada configuración especifica una clave única y opcionalmente un valor predeterminado y una función de validación.

### Siembra

Usar los macros `define_internal_config!` o `define_exposed_config!` registra automáticamente su configuración para la siembra. Al iniciar la aplicación, todas las configuraciones registradas se siembran en la base de datos. Los desarrolladores que definen nuevas configuraciones no necesitan llamar manualmente a ninguna función de siembra - solo use el macro y la configuración estará disponible. Debido a esta siembra automática, `get` siempre tiene éxito para todas las configuraciones.

### Lectura de Valores

Para leer una configuración, llame a `get::<SuConfiguracion>()` en el servicio apropiado:

```rust
// Configuración interna (aplique su propia autorización antes de esta llamada)
let typed_config: TypedDomainConfig<MyConfig> = internal_configs.get::<MyConfig>().await?;

// Configuración expuesta (requiere sujeto de autenticación)
let typed_config: TypedDomainConfig<MyConfig> = exposed_configs.get::<MyConfig>(&subject).await?;
```

El método `get()` devuelve un envoltorio `TypedDomainConfig<C>`. Llame a `.value()` para obtener el valor resuelto como un `Option<T>` estándar:

- `Some(value)` - el valor resuelto (ya sea de la base de datos o el predeterminado)
- `None` - no existe ningún valor y no se definió un valor predeterminado

El llamador no necesita saber si el valor provino de una entrada explícita en la base de datos o del valor predeterminado definido en el registro.
