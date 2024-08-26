use hex;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, Read};

/// Function to calculate the SHA-256 hash of a file and return it as a hexadecimal `String`
pub fn hash_file(file_path: &str) -> io::Result<String> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(hash_bytes(&buffer))
}

pub fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
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
