# rhll

A Rust-powered [HyperLogLog](https://en.wikipedia.org/wiki/HyperLogLog)
cardinality estimator for Python, exposed via [PyO3](https://pyo3.rs).

## Install

```sh
pip install rhll
```

Prebuilt wheels (Linux/macOS/Windows, Python 3.11+) via `abi3`, no Rust
toolchain needed at install time.

## Benchmarks

Up to 10M inserts, single-threaded, release build, `rhll` vs. `datasketch`
vs. the `HLL` C extension. Full methodology, machine details, and
count-accuracy numbers: [`benchmarks/results.md`](./benchmarks/results.md).

| n | rhll | datasketch | HLL |
|---|---|---|---|
| 10,000 | 0.001s | 0.014s | 0.001s |
| 100,000 | 0.005s | 0.123s | 0.009s |
| 1,000,000 | 0.057s | 1.288s | 0.086s |
| 10,000,000 | 0.555s | 12.399s | 0.891s |

`rhll` is ~22x faster than `datasketch` at 10M inserts, and slightly faster
than the `HLL` C extension. `count()` accuracy is in the same ballpark as
both (0.64% error, vs. 0.77% for `HLL` and 0.57% for `datasketch`, on a
10M-item dataset at `p=14`).

## Usage

```python
from rhll import HyperLogLog

h = HyperLogLog(p=12)          # default p=12 -> 4096 registers
h.update(b"some-value")        # add a single item (bytes)
h.update_str("some-value")     # convenience: str -> utf8 -> update

h.count()                      # -> float, estimated cardinality

h2 = HyperLogLog(p=12)
h2.update(b"other-value")
h.merge(h2)                    # in-place union, requires matching p

h3 = h.copy()                  # deep copy
len(h.registers())             # raw register bytes, for debugging/serialization

data = h.to_bytes()
restored = HyperLogLog.from_bytes(data)
```

> [!NOTE]
> **Compatibility with `datasketch.HyperLogLog`**
>
> `update(bytes)`, `count() -> float`, `merge(other)`, and `copy()` line up
> directly. Everything else (constructor defaults/args, serialization
> format, register access, `union()`) differs, so it's not a drop-in
> replacement.

## Design notes

- **Hashing**: `xxh3_64` (via `xxhash-rust`) rather than `datasketch`'s SHA1,
  much faster and still well-distributed for this use case.
- **Precision `p`**: valid range 4–18, `m = 2^p` registers, one `u8` per
  register (no bit-packing in v1).
- **Merging**: sketches must share the same `p`; merging mismatched sketches
  raises `ValueError` rather than silently resizing.
- **Serialization**: `to_bytes()`/`from_bytes()` is `[p][registers...]`, no
  compression, no versioning beyond `p`-derived length checks.

Out of scope for v1: HyperLogLog++ (sparse mode), arbitrary hashable objects,
distributed merge/persistence beyond raw bytes.
