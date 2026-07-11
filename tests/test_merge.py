from rhll import HyperLogLog


def test_merge_disjoint_sketches_approximates_union():
    a = HyperLogLog(p=14)
    b = HyperLogLog(p=14)
    for i in range(50_000):
        a.update_str(f"a-{i}")
    for i in range(50_000):
        b.update_str(f"b-{i}")
    a.merge(b)
    estimate = a.count()
    assert abs(estimate - 100_000) / 100_000 < 0.05


def test_merge_overlapping_sketches_approximates_union():
    a = HyperLogLog(p=14)
    b = HyperLogLog(p=14)
    for i in range(50_000):
        a.update_str(f"item-{i}")
    for i in range(25_000, 75_000):
        b.update_str(f"item-{i}")
    a.merge(b)
    estimate = a.count()
    assert abs(estimate - 75_000) / 75_000 < 0.05


def test_merge_is_order_independent():
    a = HyperLogLog(p=12)
    b = HyperLogLog(p=12)
    for i in range(1_000):
        a.update_str(f"x-{i}")
        b.update_str(f"y-{i}")

    merged_ab = a.copy()
    merged_ab.merge(b)

    merged_ba = b.copy()
    merged_ba.merge(a)

    assert merged_ab.count() == merged_ba.count()
