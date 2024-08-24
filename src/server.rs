use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use warp::Filter;

use merkleproofs::merkle_tree::MerkleTree;

#[derive(Serialize, Deserialize)]
struct UploadRequest {
    root_hash: String,
    file_paths: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct FileResponse {
    content: String,
    proof: Option<Vec<String>>, // Optional proof
}

struct AppState {
    file_store: Mutex<HashMap<String, String>>, // File paths to file contents
    merkle_trees: Mutex<HashMap<String, MerkleTree>>, // Root hash to Merkle tree
}

impl AppState {
    fn new() -> Self {
        Self {
            file_store: Mutex::new(HashMap::new()),
            merkle_trees: Mutex::new(HashMap::new()),
        }
    }
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState::new());

    let upload = warp::post()
        .and(warp::path("upload"))
        .and(warp::body::json())
        .and(with_state(state.clone()))
        .and_then(upload_files);

    let get_file = warp::get()
        .and(warp::path("file"))
        .and(warp::query::<HashMap<String, String>>())
        .and(with_state(state.clone()))
        .and_then(get_file_content);

    warp::serve(upload.or(get_file))
        .run(([127, 0, 0, 1], 3030))
        .await;
}

fn with_state(
    state: Arc<AppState>,
) -> impl Filter<Extract = (Arc<AppState>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}

async fn upload_files(
    upload_req: UploadRequest,
    state: Arc<AppState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut file_store = state.file_store.lock().unwrap();
    let mut merkle_trees = state.merkle_trees.lock().unwrap();

    // Store files
    for path in &upload_req.file_paths {
        let content = fs::read_to_string(path).expect("Unable to read file");
        file_store.insert(path.clone(), content);
    }

    // Build and store Merkle tree
    let mut merkle_tree = MerkleTree::new();
    let file_contents: Vec<String> = upload_req
        .file_paths
        .iter()
        .map(|path| file_store.get(path).cloned().unwrap_or_default())
        .collect();
    merkle_tree.build(&file_contents);
    merkle_trees.insert(upload_req.root_hash, merkle_tree);

    Ok(warp::reply::json(&"Files uploaded successfully"))
}

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
