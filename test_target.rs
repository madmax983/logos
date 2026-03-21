#[test]
fn test_transactions_as_of_error_when_mismatched_txn_id() {
    use aletheiadb::{EdgeId, Error as DbError, NodeId, StorageError};
    let node_err = DbError::Storage(StorageError::NodeNotFound(NodeId::new(1).unwrap()));
    assert!(crate::read::is_node_not_visible(&node_err));

    let edge_err = DbError::Storage(StorageError::EdgeNotFound(EdgeId::new(1).unwrap()));
    assert!(crate::read::is_edge_not_visible(&edge_err));
}
