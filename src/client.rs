use clap::Arg;
use clap::ArgAction;
use clap::Command;
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

// Example: cargo run --bin client -- upload http://127.0.0.1:8000/upload file1.txt file2.txt
// Example2: cargo run --bin client -- verify http://127.0.0.1:8000 file2.txt
#[tokio::main]
async fn main() {
    let matches = Command::new("Merkle Client")
        .version("1.0")
        .about("Uploads files to a server or verifies a file")
        .subcommand(
            Command::new("upload")
                .about("Uploads files to the server")
                .arg(Arg::new("server_url").help("The server URL").required(true))
                .arg(
                    Arg::new("files")
                        .help("List of files to upload")
                        .required(true)
                        .action(ArgAction::Append),
                ), // Use Append action
        )
        .subcommand(
            Command::new("verify")
                .about("Verifies a file from the server")
                .arg(Arg::new("server_url").help("The server URL").required(true))
                .arg(Arg::new("file").help("The file to verify").required(true)),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("upload", sub_m)) => {
            let server_url = sub_m.get_one::<String>("server_url").unwrap();
            let files: Vec<String> = sub_m
                .get_many::<String>("files")
                .unwrap()
                .map(|s| s.to_string())
                .collect();
            upload_files(server_url, &files)
                .await
                .expect("Failed to upload files");
        }
        Some(("verify", sub_m)) => {
            let server_url = sub_m.get_one::<String>("server_url").unwrap();
            let file = sub_m.get_one::<String>("file").unwrap();
            verify_file(server_url, file)
                .await
                .expect("Failed to verify file");
        }
        _ => eprintln!("Unknown command"),
    }
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

async fn verify_file(server_url: &str, file_name: &str) -> Result<(), reqwest::Error> {
    let client = Client::new();

    let response = client
        .get(format!("{}/file/{}", server_url, file_name))
        .send()
        .await?
        .error_for_status()?;

    let response_data: serde_json::Value = response.json().await?;
    let proof: Vec<String> = serde_json::from_value(response_data["proof"].clone()).unwrap();
    let content: String = serde_json::from_value(response_data["content"].clone()).unwrap();

    let stored_state = ClientState::load(Path::new(STORAGE_DIR).join(STATE_STORAGE)).expect("Failed to load client state");
    let mut tree = MerkleTree::new();
    tree.build(&[content]);

    if tree.root().unwrap_or_default() == stored_state.root_hash {
        println!("File {} is verified and correct.", file_name);
    } else {
        println!("File {} verification failed.", file_name);
    }

    Ok(())
}
