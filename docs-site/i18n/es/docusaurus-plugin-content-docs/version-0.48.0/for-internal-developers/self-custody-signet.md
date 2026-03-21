---
id: self-custody-signet
title: Prueba de Autocustodia en Signet
---

# Prueba de Auto-custodia en Signet

Esta guía recorre el flujo local de Signet para el proveedor de auto-custodia.

En los ejemplos a continuación:

- `default` es la billetera emisora y representa la parte externa que financia la billetera de préstamo
- `mykeys_receive_test` es una billetera de recepción local que puedes usar para inspeccionar descriptores
- Lana almacena solo el `xpub` de la cuenta; el `xpriv` correspondiente permanece fuera del backend

## Requisitos previos

Inicia Lana con una configuración que incluya soporte de esplora de Signet bajo `app.custody.custody_providers.self_custody_directory`:

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

También necesitas un nodo Bitcoin Core en ejecución con Signet habilitado para que `bitcoin-cli -signet` pueda comunicarse con él.

## Generación de claves preferida

El flujo compatible con Lana consiste en generar la clave de cuenta de auto-custodia localmente con `lana-cli` y pegar solo el `account_xpub` en el panel de administración:

```bash
cargo run -p lana-cli -- genxpriv --network signet
```

El comando imprime:

- `network`
- `account_path`
- `account_xpriv`
- `account_xpub`
- `receive_path_template`

Solo `account_xpub` debe estar en Lana. Mantén `account_xpriv` fuera del backend.

## Opcional: Inspeccionar una billetera de recepción de Signet en Bitcoin Core

Si deseas una billetera local de Bitcoin Core para inspeccionar descriptores de Signet, crea una billetera de descriptores:

```bash
bitcoin-cli -signet createwallet "mykeys_receive_test" false false "" false true
```

Si hay varias billeteras cargadas, siempre pasa `-rpcwallet=<walletname>`:

```bash
bitcoin-cli -signet -rpcwallet=mykeys_receive_test listdescriptors
```

Para extraer el `xpub` de cuenta BIP84 externo de la salida del descriptor público:

```bash
bitcoin-cli -signet -rpcwallet=mykeys_receive_test listdescriptors \
  | jq -r '.descriptors[]
    | select(.internal == false)
    | select(.desc | startswith("wpkh("))
    | .desc
    | capture("wpkh\\(\\[[^]]+\\](?<xpub>[^/]+)")
    | .xpub'
```

No uses `listdescriptors true` para este paso. Eso devuelve descriptores privados que contienen `tprv`, que no deben pegarse en Lana.

## Crear una Cartera Remitente

Si aún no tienes una cartera Signet con fondos para financiar transacciones, créala primero:

```bash
bitcoin-cli -signet createwallet "default"
bitcoin-cli -signet -rpcwallet=default getnewaddress
bitcoin-cli -signet -rpcwallet=default getbalance
```

Si `bitcoin-cli -signet getnewaddress` falla con `No wallet is loaded`, crea o carga una cartera antes de reintentar.

## Crear el Custodio de Autocustodia en Lana

En el panel de administración:

1. Abre el diálogo de creación de custodio.
2. Selecciona `Self-Custody`.
3. Establece `Network` en `Signet`.
4. Pega el `account_xpub` de `lana-cli genxpriv --network signet` o del paso de extracción del descriptor anterior.

No necesitas ingresar una URL de esplora en la interfaz. Lana selecciona el backend esplora de Signet desde la configuración de inicio.

## Financiar una Línea de Crédito Pendiente

Después de aprobar una propuesta de línea de crédito, Lana crea una línea de crédito pendiente con una dirección de recepción Signet derivada.

Abre la página de la línea de crédito pendiente, copia la dirección de la cartera y luego finánciala desde la cartera remitente:

```bash
bitcoin-cli -signet -rpcwallet=default sendtoaddress <pending-facility-address> 0.00001
```

Ejemplo:

```bash
bitcoin-cli -signet -rpcwallet=default sendtoaddress tb1qh3pqgmmpp4lqna4kh6ypcz3umsrta92g49q99g 0.00001
```

El comando devuelve un ID de transacción que puedes inspeccionar en un explorador de Signet:

```text
https://mempool.space/signet/tx/<txid>
```

## Cuando Lana Contabiliza los Fondos

Lana contabiliza únicamente el saldo de autocustodia confirmado.

- Las transacciones no confirmadas en mempool no se contabilizan
- Una confirmación es suficiente
- El trabajo de sincronización de saldo de autocustodia consulta cada 60 segundos

En la práctica, espera que la página de la línea de crédito pendiente se actualice aproximadamente un minuto después de que llegue la primera confirmación.

## Solución de Problemas

### No hay cartera cargada

Crea o carga una cartera antes de llamar a `getnewaddress`:

```bash
bitcoin-cli -signet createwallet "default"
```

### Múltiples billeteras cargadas

Especifica la billetera explícitamente:

```bash
bitcoin-cli -signet -rpcwallet=mykeys_receive_test listdescriptors
```

### La transacción está confirmada pero la instalación aún aparece como pendiente

Verifica lo siguiente en orden:

1. La transacción tiene al menos una confirmación
2. La configuración de Lana en ejecución incluye `signet_url`
3. Han transcurrido al menos 60 segundos desde la confirmación
4. El monto depositado es lo suficientemente grande para cumplir con el CVL requerido por la instalación

El último caso es común: el saldo de la billetera puede estar presente mientras la instalación permanece como `UNDER_COLLATERALIZED`.
