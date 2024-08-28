# Merkle proof storage for files

This project demonstrates a Merkle Proof file storage system between a client and a server.

## Overview

This project demonstrates a Merkle Proof system implemented in Rust, facilitating secure file verification between a client and a server. The system uses a Merkle tree to efficiently prove the integrity and inclusion of files.

The client can store files on the server, and ask the server for proof that it still has the files stored. This way, the client does not need to store the files itself and can trust that the server has them.

## Components

### Client

The client component provides the following functionality:
- Upload files to the server
- Send a verification request to the server
- Manage local file storage and state
- Ask the server to delete its state and files

### Server

The server component is responsible for:
- Receiving and storing uploaded files
- Generating and maintaining its own Merkle tree for hashes of the file contents
- Providing Merkle proofs for file verification requests
- Deleting the server's state and files upon request

### Merkle Tree

The Merkle tree implementation includes:
- Tree construction from a list of strings
- Root hash calculation
- Generation of Merkle proofs for specific tree nodes

## Deployment

The server is meant to be deployed with (Shuttle)[https://shuttle.rs/]. Once you have Shuttle configured, you can deploy it:
- Locally: `cargo shuttle run`
- On Shuttle: `cargo shuttle deploy`

### Existing deployment

The server has been deployed on Shuttle and can be accessed via the client at https://merkleproofs.shuttleapp.rs .

## Installation

1. Ensure you have Rust and Cargo installed on your system.
1. Clone this repository:
   ```
   git clone https://github.com/microbecode/file-merkle-proofs.git
   cd file-merkle-proofs
   ```
1. Build the project: `cargo build --release`
1. Run the server (using Shuttle): `cargo shuttle run`

## Usage

First, make sure you have a server running, either locally or online. In the following instructions, we assume the server is running locally.

If need to generate some dummy files for testing, you can use the `generate.sh` script.

### Upload files

Add the files you want to upload to a folder called "client_storage". After that, you can either upload all of them with: `cargo run --bin client -- upload http://127.0.0.1:8000 all` or specify the filenames instead of "all", separated by a space.

Note that you should remember the index of the files you upload - later verification relies on file indexes. When using the "all" option, the files will be uploaded in alphabetical order.

The server should respond with a success message and a root hash it calculated.

The files will be automatically deleted from your local folder after the upload is complete. Note that you can only upload files once.

### Verify files

Once you have uploaded files to the server, you can verify that the server really has the files. This is done with zero-based file indexes. For example, to verify the second file, run: `cargo run --bin client -- verify http://127.0.0.1:8000 1`.

The server should respond with a Merkle proof for the file, the file name and its contents. The client will then calculate a hash for the given content, use the Merkle proof to calculate a root hash and compare it against its stored root hash. If they match, the client is convinced that the server has the right contents for the file.

### Delete files and cache

The client can request the server to delete its local files and state. This is mostly useful for testing and debugging reasons.

To issue this request to the server, you can run: `cargo run --bin client -- delete_all http://127.0.0.1:8000`

## Disclaimer

This project is not production ready. It does not include any sort of security measures. It is only intended for demonstration purposes.
