import re

with open("crates/logos-store-aletheia/src/read.rs", "r") as f:
    content = f.read()

# Replace get_node_at_as_of
node_old = """fn get_node_at_as_of(
    db: &AletheiaDB,
    node_id: NodeId,
    as_of: AsOf,
) -> Result<Option<Node>, StoreError> {
    match db.get_node_at_time(node_id, as_of.valid_time(), as_of.tx_time()) {
        Ok(node) => Ok(Some(node)),
        Err(error) if is_node_not_visible(&error) => Ok(None),
        Err(error) => Err(map_load_error("unable to read node at as-of time", error)),
    }
}"""

node_new = """fn get_node_at_as_of(
    db: &AletheiaDB,
    node_id: NodeId,
    as_of: AsOf,
) -> Result<Option<Node>, StoreError> {
    let result = db.get_node_at_time(node_id, as_of.valid_time(), as_of.tx_time());
    if let Err(error) = &result {
        if is_node_not_visible(error) {
            return Ok(None);
        }
    }
    result
        .map(Some)
        .map_err(|error| map_load_error("unable to read node at as-of time", error))
}"""

content = content.replace(node_old, node_new)

# Replace get_edge_at_as_of
edge_old = """fn get_edge_at_as_of(
    db: &AletheiaDB,
    edge_id: EdgeId,
    as_of: AsOf,
) -> Result<Option<aletheiadb::Edge>, StoreError> {
    match db.get_edge_at_time(edge_id, as_of.valid_time(), as_of.tx_time()) {
        Ok(edge) => Ok(Some(edge)),
        Err(error) if is_edge_not_visible(&error) => Ok(None),
        Err(error) => Err(map_load_error("unable to read edge at as-of time", error)),
    }
}"""

edge_new = """fn get_edge_at_as_of(
    db: &AletheiaDB,
    edge_id: EdgeId,
    as_of: AsOf,
) -> Result<Option<aletheiadb::Edge>, StoreError> {
    let result = db.get_edge_at_time(edge_id, as_of.valid_time(), as_of.tx_time());
    if let Err(error) = &result {
        if is_edge_not_visible(error) {
            return Ok(None);
        }
    }
    result
        .map(Some)
        .map_err(|error| map_load_error("unable to read edge at as-of time", error))
}"""

content = content.replace(edge_old, edge_new)

with open("crates/logos-store-aletheia/src/read.rs", "w") as f:
    f.write(content)

print("Guards replaced!")
