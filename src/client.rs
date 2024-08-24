use merkleproofs::merkle_tree::MerkleTree;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct UploadRequest {
    root_hash: String,
    files: Vec<FileData>,
}

#[derive(Serialize, Deserialize)]
struct FileData {
    path: String,
    content: String,
}

async fn upload_files(server_url: &str, file_paths: &[String]) -> Result<(), reqwest::Error> {
    let file_contents: Vec<FileData> = file_paths
        .iter()
        .map(|path| FileData {
            path: path.clone(),
            content: fs::read_to_string(path).expect("Unable to read file"),
        })
        .collect();

    let mut tree = MerkleTree::new();
    let file_contents_strings: Vec<String> =
        file_contents.iter().map(|f| f.content.clone()).collect();
    tree.build(&file_contents_strings);
    let root_hash = tree
        .root()
        .clone()
        .unwrap_or_else(|| "empty_root".to_string());

    // Prepare the upload request
    let request = UploadRequest {
        root_hash,
        files: file_contents,
    };

    let response = Client::new()
        .post(format!("{}/upload", server_url))
        .json(&request)
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await?;

    println!("Response status: {:?}", status);
    println!("Response body: {:?}", body);

    Ok(())
}

#[tokio::main]
async fn main() {
    // Collect command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: client <server_url> <file1> [file2 ... fileN]");
        std::process::exit(1);
    }

    let server_url = &args[1];
    let file_paths = &args[2..];

    // Call the upload function
    if let Err(e) = upload_files(server_url, file_paths).await {
        eprintln!("Error uploading files: {}", e);
        std::process::exit(1);
    }
}
