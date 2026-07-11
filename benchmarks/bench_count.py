"""Accuracy + speed of count(): rhll vs datasketch vs HLL.

Inserts a known-cardinality dataset (with duplicates) into each library and
records (estimated_count, true_count, error_pct, insert_time, count_time).

Usage: python benchmarks/bench_count.py
"""

from __future__ import annotations

import time

from rhll import HyperLogLog as RHLL

try:
    from datasketch import HyperLogLog as DatasketchHLL
except ImportError:
    DatasketchHLL = None

try:
    from HLL import HyperLogLog as CHLL
except ImportError:
    CHLL = None

TRUE_CARDINALITY = 100_000
DUPLICATE_FACTOR = 3  # each distinct item inserted this many times
P = 14


def make_items() -> list[bytes]:
    items = [f"item-{i}".encode() for i in range(TRUE_CARDINALITY)]
    return items * DUPLICATE_FACTOR


def run(name: str, make_sketch, insert, count) -> None:
    items = make_items()
    sketch = make_sketch()

    start = time.perf_counter()
    for item in items:
        insert(sketch, item)
    insert_time = time.perf_counter() - start

    start = time.perf_counter()
    estimate = count(sketch)
    count_time = time.perf_counter() - start

    error_pct = abs(estimate - TRUE_CARDINALITY) / TRUE_CARDINALITY * 100
    print(
        f"{name:<12} estimate={estimate:>12,.0f}  true={TRUE_CARDINALITY:>10,}  "
        f"error={error_pct:>6.2f}%  insert={insert_time:>7.3f}s  count={count_time:>8.5f}s"
    )


def main() -> None:
    run(
        "rhll",
        lambda: RHLL(p=P),
        lambda s, item: s.update(item),
        lambda s: s.count(),
    )

    if DatasketchHLL:
        run(
            "datasketch",
            lambda: DatasketchHLL(p=P),
            lambda s, item: s.update(item),
            lambda s: s.count(),
        )
    else:
        print("datasketch not installed, skipping")

    if CHLL:
        run(
            "HLL",
            lambda: CHLL(p=P),
            lambda s, item: s.add(item),
            lambda s: s.cardinality(),
        )
    else:
        print("HLL not installed, skipping")


if __name__ == "__main__":
    main()
