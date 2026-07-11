import pytest

from rhll import HyperLogLog


def test_empty_sketch_counts_zero():
    h = HyperLogLog(p=12)
    assert h.count() == 0.0


def test_single_item():
    h = HyperLogLog(p=12)
    h.update(b"only-item")
    assert h.count() > 0


@pytest.mark.parametrize("p", [4, 18])
def test_p_bounds_are_valid(p):
    HyperLogLog(p=p)


@pytest.mark.parametrize("p", [3, 19, 0, 255])
def test_p_out_of_range_raises(p):
    with pytest.raises(ValueError):
        HyperLogLog(p=p)


def test_merge_requires_matching_p():
    a = HyperLogLog(p=10)
    b = HyperLogLog(p=12)
    with pytest.raises(ValueError):
        a.merge(b)


def test_len_matches_register_count():
    h = HyperLogLog(p=10)
    assert len(h.registers()) == 2**10
