use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct KvMapExport{
    pub operation: String,
    pub key: String,
    pub value: String
}

impl KvMapExport {
    pub fn new(key: String, value: String) -> KvMapExport {
        Self{
            operation: "SET".to_string(),
            key, value
        }
    }
}