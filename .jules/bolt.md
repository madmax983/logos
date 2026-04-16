**[Diesel `.eq_any()` Allocation Removal]**
**Learning:** `eq_any` requires an argument that implements `AsInExpression`, which includes slices and iterators returning items that match the underlying expression type, avoiding `collect` allocations for `Vec`.
**Action:** Replaced `.collect::<Vec<&str>>()` and `.collect::<Vec<String>>()` with chained iterators to `eq_any()` to eliminate unnecessary memory allocation.

**[DataFrame Construction Optimization]**
**Learning:** Polars DataFrames and Series can consume Iterators directly without requiring pre-allocation into intermediate `Vec` collections.
**Action:** Eliminated unnecessary `.collect::<Vec<T>>()` calls inside DataFrame construction in `logos-runtime`.

**[HashMap Allocation in `hydrate_transactions`]**
**Learning:** Over-allocating HashMap capacity based on `transaction_ids.len()` instead of `rows.len()` can lead to wasted capacity and trigger unnecessary allocations, because `transaction_ids` length implies a smaller unique capacity which could cause resizing under heavy duplication if mappings change.
**Action:** Replaced over-sized or mismatched allocation capacities where applicable.
