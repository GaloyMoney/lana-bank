---
id: self-custody-signet
title: Probando la autocustodia en Signet
---

# Probar la autocustodia en Signet

Esta guía explica el flujo local de Signet para el proveedor de autocustodia.

En los ejemplos siguientes:

- `default` es la billetera emisora y representa a la parte externa que financia la billetera del préstamo
- `mykeys_receive_test` es una billetera local de recepción que puedes usar para inspeccionar descriptores
- Lana almacena sólo la `xpub` de la cuenta; la `xpriv` correspondiente permanece fuera del backend

## Requisitos previos

Inicia Lana con una configuración que incluya soporte de esplora para Signet bajo `app.custody.custody_providers.self_custody_directory`:

```yaml
app:
  custody:
    custody_providers:
      self_custody_directory:
        mainnet_url: https://blockstream.info/api/
        testnet3_url: https://blockstream.info/testnet/api/
        testnet4_url: https://mempool.space/testnet4/api/
        signet_url: https://blockstream.info/signet/api/
```

También necesitas un nodo Bitcoin Core en funcionamiento con Signet habilitado para que `bitcoin-cli -signet` pueda comunicarse con él.

## Generación de claves preferida

El flujo soportado de Lana es generar la clave de cuenta de autocustodia localmente con `lana-cli` y pegar sólo el `account_xpub` en el panel de administración:

```bash
cargo run -p lana-cli -- genxpriv --network signet
```

El comando muestra:

- `network`
- `account_path`
- `account_xpriv`
- `account_xpub`
- `receive_path_template`

Sólo `account_xpub` debe ingresar en Lana. Mantén `account_xpriv` fuera del backend.

## Opcional: Inspeccionar una billetera de recepción de Signet en Bitcoin Core

Si deseas una billetera local de Bitcoin Core para inspeccionar descriptores de Signet, crea una billetera de descriptores:

```bash
bitcoin-cli -signet createwallet "mykeys_receive_test" false false "" false true
```

Si hay varias billeteras cargadas, pasa siempre `-rpcwallet=<walletname>`:

```bash
bitcoin-cli -signet -rpcwallet=mykeys_receive_test listdescriptors
```

Para extraer la `xpub` de la cuenta externa BIP84 del descriptor público:

```bash
bitcoin-cli -signet -rpcwallet=mykeys_receive_test listdescriptors \
  | jq -r '.descriptors[]
    | select(.internal == false)
    | select(.desc | startswith("wpkh("))
    | .desc
    | capture("wpkh\\(\\[[^]]+\\](?<xpub>[^/]+)")
    | .xpub'
```

No utilices `listdescriptors true` para este paso. Ese comando devuelve descriptores privados que contienen `tprv`, los cuales no deben pegarse en Lana.

## Crear una billetera de envío

Si aún no tienes una billetera de Signet cargada para financiar transacciones, crea una primero:

```bash
bitcoin-cli -signet createwallet "default"
bitcoin-cli -signet -rpcwallet=default getnewaddress
bitcoin-cli -signet -rpcwallet=default getbalance
```

Si `bitcoin-cli -signet getnewaddress` falla con `No wallet is loaded`, crea o carga una billetera antes de volver a intentarlo.

## Crear el custodio de auto-custodia en Lana

En el panel de administración:

1. Abre el diálogo para crear un custodio.
2. Elige `Self-Custody`.
3. Establece `Network` en `Signet`.
4. Pega el `account_xpub` de `lana-cli genxpriv --network signet` o desde el paso de extracción del descriptor anterior.

No necesitas ingresar una URL de esplora en la interfaz. Lana selecciona el backend de esplora de Signet según la configuración de inicio.

## Financiar una facilidad pendiente

Después de aprobar una propuesta de línea de crédito, Lana crea una facilidad pendiente con una dirección de recepción de Signet derivada.

Abre la página de la facilidad pendiente, copia la dirección de la billetera y luego fondea desde la billetera de envío:

```bash
bitcoin-cli -signet -rpcwallet=default sendtoaddress <pending-facility-address> 0.00001
```

Ejemplo:

```bash
bitcoin-cli -signet -rpcwallet=default sendtoaddress tb1qh3pqgmmpp4lqna4kh6ypcz3umsrta92g49q99g 0.00001
```

El comando retorna un ID de transacción que puedes inspeccionar en un explorador de Signet:

```text
https://mempool.space/signet/tx/<txid>
```

## Cuando Lana cuenta los fondos

Lana solo cuenta el saldo confirmado en auto-custodia.

- Las transacciones no confirmadas en mempool no cuentan
- Una confirmación es suficiente
- El proceso de sincronización del saldo de auto-custodia se ejecuta cada 60 segundos

En la práctica, espera que la página de la facilidad pendiente se actualice aproximadamente un minuto después de la primera confirmación.

## Resolución de problemas

### No hay billetera cargada

Crea o carga una billetera antes de ejecutar `getnewaddress`:

```bash
bitcoin-cli -signet createwallet "default"
```

### Múltiples monederos cargados

Especifica el monedero explícitamente:

```bash
bitcoin-cli -signet -rpcwallet=mykeys_receive_test listdescriptors
```

### La transacción está confirmada pero la instalación aún indica pendiente

Verifica lo siguiente en orden:

1. La transacción tiene al menos una confirmación
2. La configuración activa de Lana incluye `signet_url`
3. Han pasado al menos 60 segundos desde la confirmación
4. El monto depositado es lo suficientemente grande para cumplir con el CVL requerido por la instalación

El último caso es común: el saldo del monedero puede estar presente mientras la instalación sigue `UNDER_COLLATERALIZED`.
