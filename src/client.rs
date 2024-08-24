use merkleproofs::client_state::ClientState;
use merkleproofs::merkle_tree::MerkleTree;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;

const STORAGE_DIR: &str = "client_storage";
const STATE_STORAGE: &str = "state.json";

#[derive(Serialize, Deserialize)]
struct UploadRequest {
    root_hash: String,
    files: Vec<FileData>,
}

#[derive(Serialize, Deserialize)]
struct FileData {
    name: String,
    content: String,
}

fn ensure_storage_dir_exists() {
    if !Path::new(STORAGE_DIR).exists() {
        fs::create_dir_all(STORAGE_DIR).expect("Failed to create storage directory");
    }
}

async fn upload_files(server_url: &str, file_paths: &[String]) -> Result<(), reqwest::Error> {
    ensure_storage_dir_exists();
    // Read file contents and prepare file data
    let files: Vec<FileData> = file_paths
        .iter()
        .map(|file_name| {
            let path = Path::new(STORAGE_DIR).join(file_name);
            let content = fs::read_to_string(&path).expect("Unable to read file");
            FileData {
                name: file_name.clone(),
                content,
            }
        })
        .collect();

    // Compute Merkle tree root
    let file_contents: Vec<String> = files.iter().map(|file| file.content.clone()).collect();
    let mut tree = MerkleTree::new();
    tree.build(&file_contents);
    let root_hash = tree
        .root()
        .clone()
        .unwrap_or_else(|| "empty_root".to_string());

    // Save the client state
    let state = ClientState::new(root_hash.clone());
    match state.save(Path::new(STORAGE_DIR).join(STATE_STORAGE)) {
        Ok(_) => println!("Client state saved successfully."),
        Err(e) => eprintln!("Failed to save client state: {}", e),
    }

    // Prepare the upload request with file data
    let request = UploadRequest {
        root_hash: root_hash.clone(),
        files,
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
