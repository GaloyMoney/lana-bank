from typing import Callable

def one_asset():
    print("I'm running!")

def another_asset():
    print("I'm executing!")


lana_el_callables: tuple[Callable, ...] = (one_asset, another_asset)