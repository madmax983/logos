import re

with open("crates/logos-store-aletheia/src/read.rs", "r") as f:
    content = f.read()

replacement = """    #[test]
    fn test_optional_node_timestamp_property() {
        use aletheiadb::{NodeId, PropertyMapBuilder, WriteOps};

        let path = temp_db_path("opt-ts");
        let db = crate::open_embedded_db(&path).unwrap();
        let node_id = db.write(|tx| {
            let props = PropertyMapBuilder::new()
                .insert("time", 123_456i64)
                .build();
            let node_id = tx.create_node("Test", props).unwrap();
            Ok::<NodeId, aletheiadb::Error>(node_id)
        }).unwrap();

        let node = db.get_node(node_id).unwrap();

        assert_eq!(
            super::optional_node_timestamp_property(&node, "time"),
            Some(123_456i64.into())
        );
        assert_eq!(
            super::optional_node_timestamp_property(&node, "missing"),
            None
        );

        if path.exists() {
            let _ = std::fs::remove_dir_all(&path);
        }
    }

    #[test]
    fn test_transactions_as_of_error_when_mismatched_txn_id() {"""

content = content.replace("    #[test]\n    fn test_transactions_as_of_error_when_mismatched_txn_id() {", replacement)

with open("crates/logos-store-aletheia/src/read.rs", "w") as f:
    f.write(content)

print("Patched!")
