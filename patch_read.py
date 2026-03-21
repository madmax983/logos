import re

filepath = "crates/logos-store-aletheia/src/read.rs"
with open(filepath, "r") as f:
    content = f.read()

new_tests = """
    #[test]
    fn test_get_node_at_as_of_pass_through_other_errors() {
        // Since we can't easily force the embedded db to return a random error,
        // we'll simulate the mapping logic directly.
        let err = DbError::Storage(StorageError::StorageFileCorrupted);
        assert!(!is_node_not_visible(&err));
        let mapped = map_load_error("test", err);
        assert!(matches!(mapped, StoreError::Load(_)));
    }

    #[test]
    fn test_get_edge_at_as_of_pass_through_other_errors() {
        // Similarly for edges.
        let err = DbError::Storage(StorageError::StorageFileCorrupted);
        assert!(!is_edge_not_visible(&err));
        let mapped = map_load_error("test", err);
        assert!(matches!(mapped, StoreError::Load(_)));
    }
"""

# Insert after test_get_edge_at_as_of_error_handling
pattern = r"let result = super::get_edge_at_as_of\(&db, edge_id, as_of\);\n        assert!\(result\.is_ok\(\)\);\n        assert!\(result\.unwrap\(\)\.is_none\(\)\);\n    }"

if re.search(pattern, content):
    content = re.sub(pattern, lambda m: m.group(0) + "\n" + new_tests, content)
    with open(filepath, "w") as f:
        f.write(content)
    print("Patch applied.")
else:
    print("Pattern not found.")
