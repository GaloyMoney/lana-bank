from typing import Callable, Optional

import dagster as dg

import dagster as dg


class Protoasset:
    """
    All the ingredients required to make a dagster asset, but not quite the
    dagster asset yet.
    """

    def __init__(
        self,
        key: dg.AssetKey,
        callable: Optional[Callable] = None,
        tags: Optional[dict[str, str]] = None,
        deps: Optional[list[dg.AssetKey]] = None,
        required_resource_keys: Optional[set[str]] = None,
        automation_condition: Optional[dg.AutomationCondition] = None,
    ):
        self.key = key
        self.callable = callable
        self.tags = tags
        self.deps = deps
        self.required_resource_keys = required_resource_keys
        self.automation_condition = automation_condition

    @property
    def is_external(self) -> bool:
        # An external asset is basically the same as an asset, just that it doesn't
        # have anything to call for materializing.
        # https://docs.dagster.io/api/dagster/assets#dagster.AssetSpec
        return self.callable is None
