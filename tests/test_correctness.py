import pytest

from rhll import HyperLogLog

try:
    from datasketch import HyperLogLog as DatasketchHLL
except ImportError:
    DatasketchHLL = None


@pytest.mark.parametrize("n", [1_000, 10_000, 100_000])
def test_estimate_within_error_bound(n):
    h = HyperLogLog(p=14)
    for i in range(n):
        h.update_str(f"item-{i}")
    estimate = h.count()
    error = abs(estimate - n) / n
    assert error < 0.05, f"n={n} estimate={estimate} error={error:.3%}"


def test_duplicates_do_not_inflate_count():
    h = HyperLogLog(p=14)
    for _ in range(10):
        for i in range(1_000):
            h.update_str(f"item-{i}")
    estimate = h.count()
    assert abs(estimate - 1_000) / 1_000 < 0.05


@pytest.mark.skipif(DatasketchHLL is None, reason="datasketch not installed")
def test_estimate_close_to_datasketch():
    n = 50_000
    items = [f"item-{i}".encode() for i in range(n)]

    rh = HyperLogLog(p=14)
    dh = DatasketchHLL(p=14)
    for item in items:
        rh.update(item)
        dh.update(item)

    rh_error = abs(rh.count() - n) / n
    dh_error = abs(dh.count() - n) / n
    assert rh_error < 0.05
    assert dh_error < 0.05
