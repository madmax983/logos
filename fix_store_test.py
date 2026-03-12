import re
with open("crates/logos-store-aletheia/src/read.rs", "r") as f:
    content = f.read()

# move the misplaced test into the actual mod tests block.
match = re.search(r'#\[cfg\(test\)\]\nmod tests \{', content)
if match:
    tests_block_start = match.end()

    test_code = """
    #[test]
    fn test_transactions_as_of_error_when_mismatched_txn_id() {
        use aletheiadb::{Error as DbError, StorageError, NodeId, EdgeId};
        use logos_core::TransactionId;
        let node_err = DbError::Storage(StorageError::NodeNotFound(NodeId::new(1).unwrap()));
        assert!(is_node_not_visible(&node_err));

        let edge_err = DbError::Storage(StorageError::EdgeNotFound(EdgeId::new(1).unwrap()));
        assert!(is_edge_not_visible(&edge_err));
    }
"""
    content = content[:tests_block_start] + test_code + content[tests_block_start:]

    # remove it from the bottom
    content = re.sub(r'    #\[test\]\n    fn test_transactions_as_of_error_when_mismatched_txn_id\(\) \{\n        use aletheiadb::\{Error as DbError, StorageError, NodeId, EdgeId\};\n        use logos_core::TransactionId;\n        // Since we cannot mock AletheiaDB easily, we just test the guard clauses directly\n        let node_err = DbError::Storage\(StorageError::NodeNotFound\(NodeId::new\(1\)\.unwrap\(\)\)\);\n        assert!\(is_node_not_visible\(&node_err\)\);\n        \n        let edge_err = DbError::Storage\(StorageError::EdgeNotFound\(EdgeId::new\(1\)\.unwrap\(\)\)\);\n        assert!\(is_edge_not_visible\(&edge_err\)\);\n    \}\n', '', content)

with open("crates/logos-store-aletheia/src/read.rs", "w") as f:
    f.write(content)
