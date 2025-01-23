"""BitfinexApi entry point."""

from __future__ import annotations

from tap_bitfinexapi.tap import TapBitfinexApi

TapBitfinexApi.cli()
