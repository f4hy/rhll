from rhll import HyperLogLog


def test_round_trip_preserves_count():
    h = HyperLogLog(p=12)
    for i in range(5_000):
        h.update_str(f"item-{i}")

    restored = HyperLogLog.from_bytes(h.to_bytes())

    assert restored.count() == h.count()
    assert restored.registers() == h.registers()


def test_round_trip_empty_sketch():
    h = HyperLogLog(p=10)
    restored = HyperLogLog.from_bytes(h.to_bytes())
    assert restored.count() == 0.0
