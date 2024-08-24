use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientState {
    pub root_hash: String,
}

impl ClientState {
    pub fn new(root_hash: String) -> Self {
        Self { root_hash }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        if path.as_ref().exists() {
            let data = fs::read_to_string(path)?;
            let state = serde_json::from_str(&data)?;
            Ok(state)
        } else {
            Ok(Self::new("".to_string())) // Default empty root hash
        }
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let data = serde_json::to_string(self)?;
        fs::write(path, data)?;
        Ok(())
    }
}
