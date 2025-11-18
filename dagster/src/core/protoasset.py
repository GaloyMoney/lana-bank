from typing import Callable


class Protoasset:
    """
    All the ingredients required to make a dagster asset, but not quite the
    dagster asset yet.
    """

    def __init__(self, key: str, callable: Callable):
        self.key = key
        self.callable = callable
