use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::merkle_tree::MerkleTree;

#[derive(Serialize, Deserialize)]
struct UploadRequest {
    root_hash: String,
    file_paths: Vec<String>,
}

async fn upload_files(server_url: &str, file_paths: &[String]) -> Result<(), reqwest::Error> {
    let client = Client::new();

    // Compute hashes and Merkle tree root
    let file_contents: Vec<String> = file_paths
        .iter()
        .map(|path| fs::read_to_string(path).expect("Unable to read file"))
        .collect();

    let mut tree = MerkleTree::new();
    tree.build(&file_contents);
    let root_hash = tree
        .root()
        .clone()
        .unwrap_or_else(|| "empty_root".to_string());

    // Prepare the upload request
    let request = UploadRequest {
        root_hash,
        file_paths: file_paths.to_vec(),
    };

    // Send the request to the server
    client
        .post(server_url)
        .json(&request)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}
