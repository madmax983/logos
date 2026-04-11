## 2023-10-27 - Reduce unconditional `clone` in `HashMap` grouping
**Learning:** Using `HashMap::entry(key.clone()).or_default().push(val)` unconditionally clones the key, which is wasteful for `String` keys when appending to vectors, because the clone happens for *every* element rather than just once per group.
**Action:** Replace `entry` API with `get_mut` check and fallback to `insert` with `clone` when grouping elements by string keys to minimize heap allocations.
