#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(u64);

impl NodeId {
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeId(u64);

impl EdgeId {
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub id: NodeId,
}

impl Node {
    pub fn get_property(&self, key: &str) -> Option<&PropertyValue> {
        None
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edge {
    pub id: EdgeId,
    pub target: NodeId,
}

impl Edge {
    pub fn get_property(&self, key: &str) -> Option<&PropertyValue> {
        None
    }

    pub fn has_label_str(&self, label: &str) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(i64);

impl Timestamp {
    pub fn wallclock(&self) -> i64 {
        self.0
    }
}

impl From<i64> for Timestamp {
    fn from(val: i64) -> Self {
        Self(val)
    }
}

#[derive(Clone, Debug)]
pub enum PropertyValue {
    Int(i64),
    Str(String),
}

impl PropertyValue {
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(i) => Some(*i),
            _ => None,
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

pub mod time {
    use super::Timestamp;
    pub fn now() -> Timestamp {
        Timestamp(0)
    }
}

pub struct AletheiaDB;

pub struct WriteTransaction;
impl WriteTransaction {
    pub fn apply(self, db: &AletheiaDB) -> Result<(), std::io::Error> { Ok(()) }
    pub fn add_node(self, properties: PropertyMap) -> (Self, NodeId) { (self, NodeId(0)) }
    pub fn add_edge(self, source: NodeId, target: NodeId, label: &str, properties: PropertyMap) -> (Self, EdgeId) { (self, EdgeId(0)) }
    pub fn create_node(&mut self, label: &str, properties: PropertyMap) -> Result<NodeId, std::io::Error> { Ok(NodeId(0)) }
    pub fn create_edge(&mut self, source: NodeId, target: NodeId, label: &str, properties: PropertyMap) -> Result<EdgeId, std::io::Error> { Ok(EdgeId(0)) }
    pub fn commit(self) -> Result<(), std::io::Error> { Ok(()) }
    pub fn create_node_with_valid_time(&mut self, label: &str, properties: PropertyMap, valid_time: Option<Timestamp>) -> Result<NodeId, Error> { Ok(NodeId(0)) }
    pub fn create_edge_with_valid_time(&mut self, source: NodeId, target: NodeId, label: &str, properties: PropertyMap, valid_time: Option<Timestamp>) -> Result<EdgeId, Error> { Ok(EdgeId(0)) }
}

impl AletheiaDB {
    pub fn new() -> Self {
        Self
    }

    pub fn get_node(&self, id: NodeId) -> Result<Node, std::io::Error> {
        Ok(Node { id })
    }

    pub fn get_edge(&self, id: EdgeId) -> Result<Edge, std::io::Error> {
        Ok(Edge { id, target: NodeId(0) })
    }

    pub fn get_outgoing_edges(&self, id: NodeId) -> Vec<EdgeId> {
        vec![]
    }

    pub fn get_outgoing_edges_with_label(&self, id: NodeId, label: &str) -> Vec<EdgeId> {
        vec![]
    }

    pub fn get_incoming_edges_with_label(&self, id: NodeId, label: &str) -> Vec<EdgeId> {
        vec![]
    }

    pub fn scan_nodes_by_label(&self, label: &str) -> Vec<NodeId> {
        vec![]
    }

    pub fn write_transaction(&self) -> Result<WriteTransaction, std::io::Error> {
        Ok(WriteTransaction)
    }

    pub fn write(&self, ops: WriteOps) -> Result<(), std::io::Error> {
        Ok(())
    }

    pub fn open(path: &std::path::Path) -> Result<Self, std::io::Error> {
        Ok(Self)
    }

    pub fn open_with_config(path: &std::path::Path, config: AletheiaDBConfig) -> Result<Self, std::io::Error> {
        Ok(Self)
    }

    pub fn with_unified_config(config: AletheiaDBConfig) -> Result<Self, std::io::Error> {
        Ok(Self)
    }

    pub fn as_of_us(&self, valid_us: i64, tx_us: i64) -> Result<Self, std::io::Error> {
        Ok(Self)
    }

    pub fn root_node_id(&self) -> NodeId {
        NodeId(0)
    }

    pub fn get_outgoing_edges_at_time(&self, id: NodeId, valid_time: Timestamp, tx_time: Timestamp) -> Vec<EdgeId> { vec![] }
    pub fn get_node_at_time(&self, id: NodeId, valid_time: Timestamp, tx_time: Timestamp) -> Result<Node, Error> { Ok(Node { id }) }
    pub fn get_edge_at_time(&self, id: EdgeId, valid_time: Timestamp, tx_time: Timestamp) -> Result<Edge, Error> { Ok(Edge { id, target: NodeId(0) }) }
}

#[derive(Default)]
pub struct AletheiaPersistenceConfig {
    pub data_dir: std::path::PathBuf,
}

#[derive(Default)]
pub struct AletheiaDBConfig {
    pub persistence: AletheiaPersistenceConfig,
}

impl AletheiaDBConfig {
    pub fn new() -> Self { Self::default() }
    pub fn builder() -> Self { Self::default() }
    pub fn wal(self, config: WalConfigBuilder) -> Self { self }
    pub fn build(self) -> Self { self }
    pub fn with_durability(self, durability: DurabilityMode) -> Self { self }
    pub fn with_wal_config(self, wal_config: WalConfigBuilder) -> Self { self }
}

pub enum DurabilityMode {
    Strict,
    Relaxed,
    Synchronous,
}

pub struct PropertyMapBuilder;
impl PropertyMapBuilder {
    pub fn new() -> Self { Self }
    pub fn insert(self, key: &str, value: impl Into<PropertyValue>) -> Self { self }
    pub fn insert_str(self, key: &str, value: &str) -> Self { self }
    pub fn insert_int(self, key: &str, value: i64) -> Self { self }
    pub fn build(self) -> PropertyMap { PropertyMap }
}

pub struct PropertyMap;

pub struct WalConfigBuilder;
impl WalConfigBuilder {
    pub fn new() -> Self { Self }
    pub fn max_segment_size(self, size: u64) -> Self { self }
    pub fn durability_mode(self, mode: DurabilityMode) -> Self { self }
    pub fn wal_dir(self, path: impl AsRef<std::path::Path>) -> Self { self }
    pub fn build(self) -> Self { self }
}

pub struct WriteOps;
impl WriteOps {
    pub fn new() -> Self { Self }
    pub fn add_node(self, properties: PropertyMap) -> (Self, NodeId) { (self, NodeId(0)) }
    pub fn add_edge(self, source: NodeId, target: NodeId, label: &str, properties: PropertyMap) -> (Self, EdgeId) { (self, EdgeId(0)) }
}

impl From<&str> for PropertyValue {
    fn from(val: &str) -> Self { PropertyValue::Str(val.to_owned()) }
}
impl From<&String> for PropertyValue {
    fn from(val: &String) -> Self { PropertyValue::Str(val.to_owned()) }
}
impl From<String> for PropertyValue {
    fn from(val: String) -> Self { PropertyValue::Str(val) }
}
impl From<i64> for PropertyValue {
    fn from(val: i64) -> Self { PropertyValue::Int(val) }
}
impl From<u64> for PropertyValue {
    fn from(val: u64) -> Self { PropertyValue::Int(val as i64) }
}

#[derive(Debug)]
pub enum Error {
    Storage(StorageError),
    Temporal(TemporalError),
}
impl std::fmt::Display for Error { fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { Ok(()) } }
impl std::error::Error for Error {}

#[derive(Debug)]
pub enum StorageError {
    NodeNotFound(NodeId),
    EdgeNotFound(EdgeId),
}
impl std::fmt::Display for StorageError { fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { Ok(()) } }
impl std::error::Error for StorageError {}

#[derive(Debug)]
pub enum TemporalError {
    NodeNotFoundAtTime { id: NodeId },
}
impl std::fmt::Display for TemporalError { fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { Ok(()) } }
impl std::error::Error for TemporalError {}

impl Error {
    #[allow(non_snake_case)]
    pub fn Storage(e: StorageError) -> Self { Error::Storage(e) }
    #[allow(non_snake_case)]
    pub fn Temporal(e: TemporalError) -> Self { Error::Temporal(e) }
}
