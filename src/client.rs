use clap::Arg;
use clap::ArgAction;
use clap::Command;
use merkleproofs::client_state::ClientState;
use merkleproofs::merkle_tree::calculate_hash;
use merkleproofs::merkle_tree::MerkleTree;
use reqwest::Client;
use serde::{Deserialize, Serialize};
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

// Example: cargo run --bin client -- upload http://127.0.0.1:8000 file1.txt file2.txt
// Example2: cargo run --bin client -- verify http://127.0.0.1:8000 1
// cargo run --bin client -- delete_all http://127.0.0.1:8000
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
                .arg(
                    Arg::new("file_index")
                        .help("The index of the file to verify")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("delete_all")
                .about("Deletes all files and state from the server")
                .arg(Arg::new("server_url").help("The server URL").required(true)),
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
            let file_index: usize = sub_m
                .get_one::<String>("file_index")
                .unwrap()
                .parse()
                .expect("File index must be a number");
            verify_file(server_url, file_index)
                .await
                .expect("Failed to verify file");
        }
        Some(("delete_all", sub_m)) => {
            let server_url = sub_m.get_one::<String>("server_url").unwrap();
            delete_all_server_data(server_url)
                .await
                .expect("Failed to delete all server data");
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

    // If upload was successful, delete local files
    /*   if status.is_success() {
        for file_name in file_paths {
            let path = Path::new(STORAGE_DIR).join(file_name);
            if let Err(e) = fs::remove_file(&path) {
                eprintln!("Failed to delete file {}: {}", file_name, e);
            } else {
                println!("Deleted local file: {}", file_name);
            }
        }
        println!("All local files have been deleted after successful upload.");
    } else {
        eprintln!("Upload failed. Local files were not deleted.");
    } */

    Ok(())
}

async fn verify_file(server_url: &str, file_index: usize) -> Result<(), reqwest::Error> {
    let client = Client::new();

    let response = client
        .get(format!("{}/file/{}", server_url, file_index))
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_message = response.text().await?;
        println!("Server error: {} - {}", status, error_message);
        return Ok(());
    }

    let response_data: serde_json::Value = response.json().await?;
    println!("Received response: {}", response_data);

    let proof: Vec<(String, bool)> =
        serde_json::from_value(response_data["proof"].clone()).unwrap_or_else(|_| Vec::new());
    let content: String =
        serde_json::from_value(response_data["content"].clone()).unwrap_or_default();
    let file_name: String =
        serde_json::from_value(response_data["name"].clone()).unwrap_or_default();

    let stored_state = ClientState::load(Path::new(STORAGE_DIR).join(STATE_STORAGE))
        .expect("Failed to load client state");

    // Calculate the hash of the content
    let mut current_hash = calculate_hash(&content);

    // Verify the Merkle proof
    for (sibling, is_right) in proof.iter() {
        let combined = if *is_right {
            format!("{}{}", current_hash, sibling)
        } else {
            format!("{}{}", sibling, current_hash)
        };
        current_hash = calculate_hash(&combined);
    }

    if current_hash == stored_state.root_hash {
        println!(
            "File '{}' at index {} is verified and correct.",
            file_name, file_index
        );
    } else {
        println!(
            "File '{}' at index {} verification failed.",
            file_name, file_index
        );
        println!("Calculated hash: {}", current_hash);
        println!("Stored root hash: {}", stored_state.root_hash);
    }

    Ok(())
}

async fn delete_all_server_data(server_url: &str) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let response = client
        .delete(format!("{}/delete_all", server_url))
        .send()
        .await?;

    if response.status().is_success() {
        println!("All server data has been deleted successfully.");
    } else {
        eprintln!(
            "Failed to delete server data. Status: {:?}",
            response.status()
        );
    }

    Ok(())
}
