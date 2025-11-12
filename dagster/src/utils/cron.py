# cron_expression.py
from __future__ import annotations
from dataclasses import dataclass
from typing import ClassVar

class CronError(ValueError):
    """Raised when a cron expression is invalid."""

@dataclass(frozen=True, slots=True)
class CronExpression:
    """
    Minimal cron expression value object.
    Validates a 5-field cron string on creation.
    """

    _raw: str

    _FIELD_RANGES: ClassVar[list[tuple[int, int]]] = [
        (0, 59),  # minute
        (0, 23),  # hour
        (1, 31),  # day of month
        (1, 12),  # month
        (0, 7),   # day of week (Sunday both 0 and 7)
    ]

    @classmethod
    def parse(cls, expr: str) -> CronExpression:
        if not expr or not isinstance(expr, str):
            raise CronError("Cron expression must be a non-empty string.")

        parts = expr.split()
        if len(parts) != 5:
            raise CronError(f"Expected 5 fields, got {len(parts)}: {expr!r}")

        for i, part in enumerate(parts):
            try:
                cls._validate_field(part, *cls._FIELD_RANGES[i])
            except CronError as e:
                raise CronError(f"Invalid field {i+1} ({part!r}): {e}") from e

        return cls(_raw=expr.strip())

    @staticmethod
    def _validate_field(field: str, min_: int, max_: int) -> None:
        """Very simple syntax + range validation."""
        if not field:
            raise CronError("Empty field.")

        def check_number(token: str) -> None:
            if not token.isdigit():
                raise CronError(f"Expected number, got {token!r}")
            n = int(token)
            if not (min_ <= n <= max_):
                raise CronError(f"Value {n} out of range [{min_}, {max_}].")

        def validate_range(range_expr: str) -> None:
            if "-" not in range_expr:
                raise CronError(f"Invalid range {range_expr!r}")
            start, end = (segment.strip() for segment in range_expr.split("-", 1))
            if not start or not end:
                raise CronError(f"Incomplete range {range_expr!r}")
            check_number(start)
            check_number(end)
            if int(start) > int(end):
                raise CronError(f"Range start {start} greater than end {end}.")

        def validate_step_expression(step_expr: str) -> None:
            base, step = (segment.strip() for segment in step_expr.split("/", 1))
            if not step.isdigit() or int(step) <= 0:
                raise CronError(f"Invalid step {step!r}")
            if base == "*":
                return
            if not base:
                raise CronError("Missing base value for step expression.")
            if "-" in base:
                validate_range(base)
                return
            check_number(base)

        for part in field.split(","):
            part = part.strip()
            if part == "*":
                continue
            if "/" in part:
                validate_step_expression(part)
                continue
            if "-" in part:
                validate_range(part)
                continue
            check_number(part)

    def __str__(self) -> str:
        return self._raw
