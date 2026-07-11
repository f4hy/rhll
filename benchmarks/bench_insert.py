"""Insert-throughput benchmark: rhll vs datasketch vs HLL.

Usage: python benchmarks/bench_insert.py
"""

from __future__ import annotations

import time
import uuid

from rhll import HyperLogLog as RHLL

try:
    from datasketch import HyperLogLog as DatasketchHLL
except ImportError:
    DatasketchHLL = None

try:
    from HLL import HyperLogLog as CHLL
except ImportError:
    CHLL = None

SIZES = [10_000, 100_000, 1_000_000, 10_000_000]
P = 12


def make_items(n: int) -> list[bytes]:
    return [uuid.uuid4().bytes for _ in range(n)]


def bench_rhll(items: list[bytes]) -> float:
    h = RHLL(p=P)
    start = time.perf_counter()
    for item in items:
        h.update(item)
    return time.perf_counter() - start


def bench_datasketch(items: list[bytes]) -> float:
    h = DatasketchHLL(p=P)
    start = time.perf_counter()
    for item in items:
        h.update(item)
    return time.perf_counter() - start


def bench_hll(items: list[bytes]) -> float:
    h = CHLL(p=P)
    start = time.perf_counter()
    for item in items:
        h.add(item)
    return time.perf_counter() - start


def main() -> None:
    print(f"{'n':>10} | {'rhll':>10} | {'datasketch':>12} | {'HLL':>10}")
    print("-" * 52)
    for n in SIZES:
        items = make_items(n)
        rhll_t = bench_rhll(items)
        ds_t = bench_datasketch(items) if DatasketchHLL else float("nan")
        hll_t = bench_hll(items) if CHLL else float("nan")
        print(f"{n:>10} | {rhll_t:>9.3f}s | {ds_t:>11.3f}s | {hll_t:>9.3f}s")


if __name__ == "__main__":
    main()
