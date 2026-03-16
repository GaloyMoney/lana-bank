---
id: self-custody-signet
title: Testing Self-Custody on Signet
---

# Testing Self-Custody on Signet

This guide walks through the local Signet flow for the self-custody provider.

In the examples below:

- `default` is the sender wallet and represents the outside party funding the loan wallet
- `mykeys_receive_test` is a local receive wallet you can use to inspect descriptors
- Lana stores only the account `xpub`; the matching `xpriv` stays outside the backend

## Prerequisites

Start Lana with a config that includes Signet esplora support under `app.custody.custody_providers.self_custody_directory`:

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

You also need a running Bitcoin Core node with Signet enabled so `bitcoin-cli -signet` can talk to it.

## Preferred Key Generation

The supported Lana flow is to generate the self-custody account key locally with `lana-cli` and paste only the `account_xpub` into the admin panel:

```bash
cargo run -p lana-cli -- genxpriv --network signet
```

The command prints:

- `network`
- `account_path`
- `account_xpriv`
- `account_xpub`
- `receive_path_template`

Only `account_xpub` belongs in Lana. Keep `account_xpriv` out of the backend.

## Optional: Inspect a Signet Receive Wallet in Bitcoin Core

If you want a local Bitcoin Core wallet to inspect Signet descriptors, create a descriptor wallet:

```bash
bitcoin-cli -signet createwallet "mykeys_receive_test" false false "" false true
```

If multiple wallets are loaded, always pass `-rpcwallet=<walletname>`:

```bash
bitcoin-cli -signet -rpcwallet=mykeys_receive_test listdescriptors
```

To extract the external BIP84 account `xpub` from the public descriptor output:

```bash
bitcoin-cli -signet -rpcwallet=mykeys_receive_test listdescriptors \
  | jq -r '.descriptors[]
    | select(.internal == false)
    | select(.desc | startswith("wpkh("))
    | .desc
    | capture("wpkh\\(\\[[^]]+\\](?<xpub>[^/]+)")
    | .xpub'
```

Do not use `listdescriptors true` for this step. That returns private descriptors containing `tprv`, which must not be pasted into Lana.

## Create a Sender Wallet

If you do not already have a loaded Signet wallet for funding transactions, create one first:

```bash
bitcoin-cli -signet createwallet "default"
bitcoin-cli -signet -rpcwallet=default getnewaddress
bitcoin-cli -signet -rpcwallet=default getbalance
```

If `bitcoin-cli -signet getnewaddress` fails with `No wallet is loaded`, create or load a wallet before retrying.

## Create the Self-Custody Custodian in Lana

In the admin panel:

1. Open the custodian create dialog.
2. Choose `Self-Custody`.
3. Set `Network` to `Signet`.
4. Paste the `account_xpub` from `lana-cli genxpriv --network signet` or from the descriptor extraction step above.

You do not need to enter an esplora URL in the UI. Lana selects the Signet esplora backend from startup config.

## Fund a Pending Facility

After you approve a credit facility proposal, Lana creates a pending facility with a derived Signet receive address.

Open the pending facility page, copy the wallet address, then fund it from the sender wallet:

```bash
bitcoin-cli -signet -rpcwallet=default sendtoaddress <pending-facility-address> 0.00001
```

Example:

```bash
bitcoin-cli -signet -rpcwallet=default sendtoaddress tb1qh3pqgmmpp4lqna4kh6ypcz3umsrta92g49q99g 0.00001
```

The command returns a transaction id you can inspect in a Signet explorer:

```text
https://mempool.space/signet/tx/<txid>
```

## When Lana Counts the Funds

Lana counts only confirmed self-custody balance.

- Unconfirmed mempool transactions do not count
- One confirmation is enough
- The self-custody balance sync job polls every 60 seconds

In practice, expect the pending facility page to update within about a minute after the first confirmation lands.

## Troubleshooting

### No wallet loaded

Create or load a wallet before calling `getnewaddress`:

```bash
bitcoin-cli -signet createwallet "default"
```

### Multiple wallets loaded

Specify the wallet explicitly:

```bash
bitcoin-cli -signet -rpcwallet=mykeys_receive_test listdescriptors
```

### The transaction is confirmed but the facility still says pending

Check these in order:

1. The transaction has at least one confirmation
2. The running Lana config includes `signet_url`
3. At least 60 seconds have passed since confirmation
4. The deposited amount is large enough to meet the facility's required CVL

The last case is common: the wallet balance can be present while the facility remains `UNDER_COLLATERALIZED`.
