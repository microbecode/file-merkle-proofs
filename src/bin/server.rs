use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::{fmt, fs};
use tokio::sync::RwLock;
use warp::reject::Reject;
use warp::Filter;
use warp::{Rejection, Reply};

use merkleproofs::merkle_tree::MerkleTree;

/// Directory where the files are stored
const STORAGE_DIR: &str = "server_storage";

#[derive(Serialize, Deserialize)]
struct FileData {
    name: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
struct UploadRequest {
    root_hash: String,
    files: Vec<FileData>,
}

#[derive(Serialize, Deserialize)]
struct FileResponse {
    content: String,
    proof: Option<Vec<String>>,
}

#[derive(Clone)]
struct AppState {
    file_store: Arc<RwLock<Vec<(String, String)>>>, // Ordered list of (filename, content)
    file_index: Arc<RwLock<HashMap<String, usize>>>, // Filename to index mapping
    merkle_tree: Arc<RwLock<Option<MerkleTree>>>,   // The Merkle tree
    root_hash: Arc<RwLock<Option<String>>>,         // The root hash of the Merkle tree
}

impl AppState {
    fn new() -> Self {
        Self {
            file_store: Arc::new(RwLock::new(Vec::new())),
            file_index: Arc::new(RwLock::new(HashMap::new())),
            merkle_tree: Arc::new(RwLock::new(None)),
            root_hash: Arc::new(RwLock::new(None)),
        }
    }
}

fn ensure_storage_dir_exists() {
    if !Path::new(STORAGE_DIR).exists() {
        fs::create_dir_all(STORAGE_DIR).expect("Failed to create storage directory");
    }
}

/// Main function that sets up the server
#[shuttle_runtime::main]
async fn warp() -> shuttle_warp::ShuttleWarp<(impl Reply,)> {
    let state = Arc::new(AppState::new());

    // Route for uploading files
    let upload_route = warp::post()
        .and(warp::path("upload"))
        .and(warp::body::json())
        .and(with_state(state.clone())) // Ensure this matches the state filter
        .and_then(|request: UploadRequest, state: Arc<AppState>| async move {
            upload_files(request, state).await
        });

    // Route for verifying a file
    let verify_route = warp::get()
        .and(warp::path!("file" / usize))
        .and(with_state(state.clone()))
        .and_then(get_file_content);

    // Route for deleting all files and state
    let delete_route = warp::delete()
        .and(warp::path("delete_all"))
        .and(with_state(state.clone()))
        .and_then(delete_all);

    let routes = upload_route.or(verify_route).or(delete_route);

    Ok((routes).boxed().into())
}

fn with_state(
    state: Arc<AppState>,
) -> impl Filter<Extract = (Arc<AppState>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}

/// Uploads files to the server and updates the Merkle tree
async fn upload_files(
    request: UploadRequest,
    state: Arc<AppState>,
) -> Result<impl Reply, Rejection> {
    ensure_storage_dir_exists();

    let mut file_contents: Vec<String> = Vec::new();
    let mut file_store = state.file_store.write().await;
    let mut file_index = state.file_index.write().await;

    for file in request.files {
        let file_path = Path::new(STORAGE_DIR).join(&file.name);
        if let Err(_) = fs::write(&file_path, &file.content) {
            return Err(warp::reject::custom(CustomError::new(
                "Failed to write file",
            )));
        }
        let index = file_store.len();
        file_store.push((file.name.clone(), file.content.clone()));
        file_index.insert(file.name.clone(), index);
        file_contents.push(file.content.clone());
        println!(
            "Stored file {:?} at index {}",
            file_path.file_name().unwrap(),
            index
        );
    }

    for (index, (name, content)) in file_store.iter().enumerate() {
        println!("Index {}: {} ({})", index, name, content.len());
    }

    let mut merkle_tree = MerkleTree::new();
    merkle_tree.build(&file_contents);
    let root_hash = merkle_tree.root().unwrap_or_default();

    *state.merkle_tree.write().await = Some(merkle_tree);
    *state.root_hash.write().await = Some(root_hash.clone());

    Ok(warp::reply::json(&json!({
        "message": "Files uploaded successfully",
        "root_hash": root_hash
    })))
}

/// Verifies a file by its index. Sends a verification object as a response
async fn get_file_content(
    file_index: usize,
    state: Arc<AppState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    println!(
        "Received verification request for file index: {}",
        file_index
    );
    let file_store = state.file_store.read().await;

    let (file_name, content) = file_store.get(file_index).ok_or_else(|| {
        warp::reject::custom(CustomError::new(&format!(
            "File at index {} not found",
            file_index
        )))
    })?;

    let merkle_tree = state.merkle_tree.read().await;
    let tree = merkle_tree.as_ref().ok_or(warp::reject::not_found())?;

    let proof = tree.get_merkle_proof(file_index);

    let response = json!({
        "name": file_name,
        "content": content,
        "proof": proof
    });

    Ok(warp::reply::json(&response))
}

/// Deletes all files and state from the server
async fn delete_all(state: Arc<AppState>) -> Result<impl Reply, Rejection> {
    // Clear the file store and index
    let mut file_store = state.file_store.write().await;
    file_store.clear();
    let mut file_index = state.file_index.write().await;
    file_index.clear();

    // Reset the Merkle tree and root hash
    let mut merkle_tree = state.merkle_tree.write().await;
    *merkle_tree = None;

    let mut root_hash = state.root_hash.write().await;
    *root_hash = None;

    // Delete all files in the storage directory
    if let Err(e) = fs::remove_dir_all(STORAGE_DIR) {
        eprintln!("Failed to delete storage directory: {}", e);
        return Err(warp::reject::custom(CustomError::new(
            "Failed to delete storage directory",
        )));
    }

    // Recreate the empty storage directory
    ensure_storage_dir_exists();

    Ok(warp::reply::json(&json!({
        "message": "All files and state have been deleted"
    })))
}

#[derive(Debug)]
struct CustomError {
    message: String,
}

impl CustomError {
    fn new(message: &str) -> Self {
        CustomError {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Reject for CustomError {}
