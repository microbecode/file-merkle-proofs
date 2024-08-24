use serde::{Deserialize, Serialize};
use warp::reject::Reject;
use std::collections::HashMap;
use std::{fmt, fs};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use warp::Filter;
use warp::{Rejection, Reply};

use merkleproofs::merkle_tree::MerkleTree;

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
    proof: Option<Vec<String>>, // Optional proof
}

#[derive(Clone)]
struct AppState {
    //file_store: Mutex<HashMap<String, String>>, // File paths to file contents
    // merkle_trees: Mutex<HashMap<String, MerkleTree>>, // Root hash to Merkle tree
}

impl AppState {
    fn new() -> Self {
        Self {
           
           // file_store: Mutex::new(HashMap::new()),
           // merkle_trees: Mutex::new(HashMap::new()),
        }
    }
}

fn ensure_storage_dir_exists() {
    if !Path::new(STORAGE_DIR).exists() {
        fs::create_dir_all(STORAGE_DIR).expect("Failed to create storage directory");
    }
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState::new());

    let upload = warp::post()
        .and(warp::path("upload"))
        .and(warp::body::json())
        .and(with_state(state.clone())) // Ensure this matches the state filter
        .and_then(|request: UploadRequest, state: Arc<AppState>| async move {
            upload_files(request, state).await
        });

    /* let get_file = warp::get()
    .and(warp::path("file"))
    .and(warp::query::<HashMap<String, String>>())
    .and(with_state(state.clone()))
    .and_then(get_file_content); */

    println!("Starting server on http://127.0.0.1:8080");

    warp::serve(upload).run(([127, 0, 0, 1], 8080)).await;
}

fn with_state(
    state: Arc<AppState>,
) -> impl Filter<Extract = (Arc<AppState>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}


async fn upload_files(
    request: UploadRequest,
    state: Arc<AppState>,
) -> Result<impl Reply, Rejection> {
    ensure_storage_dir_exists();

    for file in request.files {
        let file_path = Path::new(STORAGE_DIR).join(file.name);
        if let Err(e) = fs::write(&file_path, file.content) {
            return Err(warp::reject::custom(CustomError::new("Failed to write file")));
        }
    }

    Ok(warp::reply::with_status(
        "Files uploaded successfully",
        warp::http::StatusCode::OK,
    ))
}



/* 
async fn get_file_content(
    query: HashMap<String, String>,
    state: Arc<AppState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let path = query.get("path").ok_or(warp::reject::not_found())?.clone();
    let file_store = state.file_store.lock().unwrap();

    let content = file_store
        .get(&path)
        .ok_or(warp::reject::not_found())?
        .clone();

    // Generate proof if needed
    let root_hash = query
        .get("root_hash")
        .ok_or(warp::reject::not_found())?
        .clone();
    let merkle_trees = state.merkle_trees.lock().unwrap();
    let merkle_tree = merkle_trees
        .get(&root_hash)
        .ok_or(warp::reject::not_found())?;

    let index = file_store.keys().position(|p| p == &path).unwrap_or(0);
    let proof = merkle_tree.get_merkle_proof(index);

    Ok(warp::reply::json(&FileResponse { content, proof }))
}
 */

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