use hex;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, Read};

/// Function to calculate the SHA-256 hash of a file and return it as a hexadecimal `String`
pub fn hash_file(file_path: &str) -> io::Result<String> {
    let mut hasher = Sha256::new();
    let mut file = File::open(file_path)?;

    // Read the file in chunks to avoid loading the entire file into memory
    let mut buffer = [0u8; 1024];
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    // Finalize the hash and convert it to a hexadecimal string
    let result = hasher.finalize();
    Ok(hex::encode(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::write;
    use tempfile::NamedTempFile; // Use tempfile crate for creating temporary files

    #[test]
    fn test_hash_file() {
        let content = b"Hello, world!";
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        write(temp_file.path(), content).expect("Failed to write to temp file");

        let expected_hash_hex = "315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3";

        let hash = hash_file(temp_file.path().to_str().unwrap()).expect("Failed to hash file");

        assert_eq!(hash, expected_hash_hex);
    }

    #[test]
    fn test_hash_empty_file() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");

        let expected_hash_hex = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

        let hash = hash_file(temp_file.path().to_str().unwrap()).expect("Failed to hash file");

        assert_eq!(hash, expected_hash_hex);
    }
}
