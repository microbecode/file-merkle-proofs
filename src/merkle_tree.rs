use hex;
use sha2::{Digest, Sha256};

#[derive(Debug)]
pub struct MerkleTree {
    root: Option<String>,
    levels: Vec<Vec<String>>,
}

/// Function to calculate SHA-256 hash of a `String`
pub fn calculate_hash(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes()); // Hash the bytes of the string
    let result = hasher.finalize();
    hex::encode(result) // Convert the hash to a hexadecimal string
}

impl MerkleTree {
    pub fn new() -> Self {
        MerkleTree {
            root: None,
            levels: Vec::new(),
        }
    }

    /// Build the Merkle tree from a list of elements
    // For example, with three elements A, B, C, the tree will be:
    //
    //     root
    //    /    \
    //    D    E      // level 1, where D = hash(AB) and E = hash(CC)
    //   / \  / \
    //  A  B C  C     // level 0
    pub fn build(&mut self, elements: &[String]) {
        // Hash the input elements
        let mut hashes: Vec<String> = elements.iter().map(|e| calculate_hash(e)).collect();

        // Ensure an even number of hashes by duplicating the last one if necessary
        if hashes.len() % 2 != 0 {
            hashes.push(hashes[hashes.len() - 1].clone());
        }

        let mut nodes = Vec::new();
        nodes.push(hashes.clone());

        while hashes.len() > 1 {
            let mut new_hashes = Vec::new();

            // Process pairs of hashes
            for chunk in hashes.chunks(2) {
                if chunk.len() == 2 {
                    let combined_hash = format!("{}{}", chunk[0], chunk[1]);
                    new_hashes.push(calculate_hash(&combined_hash));
                } else {
                    let combined_hash = format!("{}{}", chunk[0], chunk[0]);
                    new_hashes.push(calculate_hash(&combined_hash));
                }
            }

            nodes.push(new_hashes.clone());

            hashes = new_hashes;
        }

        // Set the root and levels
        self.root = hashes.pop();
        self.levels = nodes;
    }

    pub fn root(&self) -> Option<String> {
        self.root.clone()
    }

    /// Get the Merkle proof for a given index
    /// Generates (duplicates) nodes on the fly if missing from the tree
    pub fn get_merkle_proof(&self, index: usize) -> Option<Vec<(String, bool)>> {
        if index >= self.levels[0].len() {
            return None;
        }

        let mut proof = Vec::new();
        let mut current_index = index;

        for level in self.levels.iter().take(self.levels.len() - 1) {
            let sibling_index = current_index ^ 1; // XOR with 1 flips the last bit

            let sibling_hash = if sibling_index < level.len() {
                level[sibling_index].clone()
            } else {
                // Duplicate the current node if sibling is out of bounds
                level[current_index].clone()
            };

            proof.push((sibling_hash, sibling_index > current_index));
            current_index /= 2;
        }

        Some(proof)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn empty_tree() {
        let tree = MerkleTree::new();
        assert_eq!(tree.root, None);
        assert_eq!(tree.levels.len(), 0);
    }

    #[test]
    fn build_empty_tree() {
        let mut tree = MerkleTree::new();

        let elements: Vec<String> = Vec::new();
        tree.build(&elements);

        assert_eq!(tree.root, None);
        assert_eq!(tree.levels.len(), 1);
        assert_eq!(tree.levels[0].len(), 0);
    }

    #[test]
    fn build_tree_one_element() {
        let mut tree = MerkleTree::new();

        let val: String = "a".to_string();
        let mut elements: Vec<String> = Vec::new();
        elements.push(val.clone()); // Use `val.clone()` to avoid moving `val` if needed elsewhere

        tree.build(&elements);

        let expected_leaf = calculate_hash(&val);
        // Concatenate `expected_leaf` with itself
        let combined_leaf = format!("{}{}", expected_leaf, expected_leaf);
        let expected_root = calculate_hash(&combined_leaf);

        // Verify levels
        assert_eq!(tree.levels.len(), 2);
        assert_eq!(tree.levels[0].len(), 2);

        // Verify leaf and root
        assert_eq!(tree.root, Some(expected_root));
        assert_eq!(tree.levels[0][0], expected_leaf);
        assert_eq!(tree.levels[0][1], expected_leaf);
    }

    #[test]
    fn build_tree_two_elements() {
        let mut tree = MerkleTree::new();

        let val1: String = "a".to_string();
        let val2: String = "b".to_string();
        let mut elements: Vec<String> = Vec::new();

        elements.push(val1.clone());
        elements.push(val2.clone());
        tree.build(&elements);

        let expected_leaf_1 = calculate_hash(&val1);
        let expected_leaf_2 = calculate_hash(&val2);
        let expected_root = calculate_hash(&format!("{}{}", expected_leaf_1, expected_leaf_2));
        //let expected_root = calculate_hash(&expected_leaf_1.wrapping_add(expected_leaf_2));

        assert_eq!(tree.levels.len(), 2);
        assert_eq!(tree.levels[0].len(), 2);
        assert_eq!(tree.levels[1].len(), 1);

        assert_eq!(tree.root, Some(expected_root));
        assert_eq!(tree.levels[0][0], expected_leaf_1);
        assert_eq!(tree.levels[0][1], expected_leaf_2);
    }

    #[test]
    fn build_tree_three_elements() {
        let mut tree = MerkleTree::new();

        let val1: String = "a".to_string();
        let val2: String = "b".to_string();
        let val3: String = "c".to_string();
        let elements: Vec<String> = vec![val1, val2, val3];

        tree.build(&elements);

        // Calculate the expected hashes
        let expected_leaf_1 = calculate_hash(&elements[0]);
        let expected_leaf_2 = calculate_hash(&elements[1]);
        let expected_leaf_3 = calculate_hash(&elements[2]);

        // Duplicate the last leaf hash to ensure even number of hashes
        let expected_leaf_4 = expected_leaf_3.clone();

        // Calculate the intermediate hashes
        let expected_mid_node1 = calculate_hash(&format!("{}{}", expected_leaf_1, expected_leaf_2));
        let expected_mid_node2 = calculate_hash(&format!("{}{}", expected_leaf_3, expected_leaf_4));

        // Calculate the root hash
        let expected_root =
            calculate_hash(&format!("{}{}", expected_mid_node1, expected_mid_node2));

        // Assertions
        assert_eq!(tree.levels.len(), 3);
        assert_eq!(tree.levels[0].len(), 4); // 3 leaves + 1 duplicated leaf
        assert_eq!(tree.levels[1].len(), 2); // 2 intermediate nodes
        assert_eq!(tree.levels[2].len(), 1); // 1 root node

        assert_eq!(tree.root, Some(expected_root.clone()));
        assert_eq!(tree.levels[2][0], expected_root);

        assert_eq!(tree.levels[0][0], expected_leaf_1);
        assert_eq!(tree.levels[0][1], expected_leaf_2);
        assert_eq!(tree.levels[0][2], expected_leaf_3);
        assert_eq!(tree.levels[0][3], expected_leaf_4);

        assert_eq!(tree.levels[1][0], expected_mid_node1);
        assert_eq!(tree.levels[1][1], expected_mid_node2);
    }

    // Test a tree that has an odd amount of middle nodes.
    #[test]
    fn build_tree_three_elements_in_middle() {
        let mut tree = MerkleTree::new();

        let val1: String = "a".to_string();
        let val2: String = "b".to_string();
        let val3: String = "c".to_string();
        let val4: String = "d".to_string();
        let val5: String = "e".to_string();
        let elements: Vec<String> = vec![val1, val2, val3, val4, val5];

        tree.build(&elements);

        // Calculate the expected hashes
        let expected_leaf_1 = calculate_hash(&elements[0]);
        let expected_leaf_2 = calculate_hash(&elements[1]);
        let expected_leaf_3 = calculate_hash(&elements[2]);
        let expected_leaf_4 = calculate_hash(&elements[3]);
        let expected_leaf_5 = calculate_hash(&elements[4]);

        // Duplicate the last leaf hash to ensure even number of hashes
        let expected_leaf_6 = expected_leaf_5.clone();

        // Calculate the intermediate hashes
        let expected_mid1_node1 =
            calculate_hash(&format!("{}{}", expected_leaf_1, expected_leaf_2));
        let expected_mid1_node2 =
            calculate_hash(&format!("{}{}", expected_leaf_3, expected_leaf_4));
        let expected_mid1_node3 =
            calculate_hash(&format!("{}{}", expected_leaf_5, expected_leaf_5));

        let expected_mid2_node1 =
            calculate_hash(&format!("{}{}", expected_mid1_node1, expected_mid1_node2));
        let expected_mid3_node2 =
            calculate_hash(&format!("{}{}", expected_mid1_node3, expected_mid1_node3));

        // Calculate the root hash
        let expected_root =
            calculate_hash(&format!("{}{}", expected_mid2_node1, expected_mid3_node2));

        // Assertions
        assert_eq!(tree.levels.len(), 4);
        assert_eq!(tree.levels[0].len(), 6); // 5 leaves + 1 duplicated leaf
        assert_eq!(tree.levels[1].len(), 3); // 3 intermediate nodes
        assert_eq!(tree.levels[2].len(), 2);
        assert_eq!(tree.levels[3].len(), 1); // 1 root node

        assert_eq!(tree.root, Some(expected_root.clone()));
        assert_eq!(tree.levels[3][0], expected_root);

        assert_eq!(tree.levels[0][0], expected_leaf_1);
        assert_eq!(tree.levels[0][1], expected_leaf_2);
        assert_eq!(tree.levels[0][2], expected_leaf_3);
        assert_eq!(tree.levels[0][3], expected_leaf_4);
        assert_eq!(tree.levels[0][4], expected_leaf_5);
        assert_eq!(tree.levels[0][5], expected_leaf_6);

        assert_eq!(tree.levels[1][0], expected_mid1_node1);
        assert_eq!(tree.levels[1][1], expected_mid1_node2);
        assert_eq!(tree.levels[1][2], expected_mid1_node3);

        assert_eq!(tree.levels[2][0], expected_mid2_node1);
        assert_eq!(tree.levels[2][1], expected_mid3_node2);
    }

    #[test]
    fn get_merkle_proof_with_three_elements() {
        let mut tree = MerkleTree::new();

        let val1: String = "3".to_string();
        let val2: String = "4".to_string();
        let val3: String = "5".to_string();
        let elements: Vec<String> = vec![val1.clone(), val2.clone(), val3.clone()];

        tree.build(&elements);

        let expected_leaf_1 = calculate_hash(&val1);
        let expected_leaf_2 = calculate_hash(&val2);
        let expected_leaf_3 = calculate_hash(&val3);

        // Duplicate the last leaf hash to ensure even number of hashes
        let expected_leaf_4 = expected_leaf_3.clone();

        // Calculate intermediate hashes
        let expected_mid_node1 = calculate_hash(&format!("{}{}", expected_leaf_1, expected_leaf_2));
        let expected_mid_node2 = calculate_hash(&format!("{}{}", expected_leaf_3, expected_leaf_4));

        // Function to verify the proof
        fn verify_proof(proof: Vec<(String, bool)>, expected_proof: Vec<String>) {
            assert_eq!(proof.len(), expected_proof.len());
            for ((elem1, _), elem2) in proof.iter().zip(expected_proof.iter()) {
                assert_eq!(elem1, elem2);
            }
        }

        // Test proofs for each leaf
        {
            let proof = tree.get_merkle_proof(0).unwrap();
            let expected_proof = vec![expected_leaf_2, expected_mid_node2.clone()];
            verify_proof(proof, expected_proof);
        }
        {
            let proof = tree.get_merkle_proof(1).unwrap();
            let expected_proof = vec![expected_leaf_1, expected_mid_node2.clone()];
            verify_proof(proof, expected_proof);
        }
        {
            let proof = tree.get_merkle_proof(2).unwrap();
            let expected_proof = vec![expected_leaf_4, expected_mid_node1];
            verify_proof(proof, expected_proof);
        }
    }

    #[test]
    fn get_merkle_proof_with_five_elements() {
        let mut tree = MerkleTree::new();

        let val1: String = "3".to_string();
        let val2: String = "4".to_string();
        let val3: String = "5".to_string();
        let val4: String = "6".to_string();
        let val5: String = "7".to_string();
        let elements: Vec<String> = vec![
            val1.clone(),
            val2.clone(),
            val3.clone(),
            val4.clone(),
            val5.clone(),
        ];

        tree.build(&elements);

        let expected_leaf_1 = calculate_hash(&val1);
        let expected_leaf_2 = calculate_hash(&val2);
        let expected_leaf_3 = calculate_hash(&val3);
        let expected_leaf_4 = calculate_hash(&val4);
        let expected_leaf_5 = calculate_hash(&val5);

        // Duplicate the last leaf hash to ensure even number of hashes
        let expected_leaf_6 = expected_leaf_5.clone();

        // Calculate intermediate hashes
        let expected_mid1_node1 =
            calculate_hash(&format!("{}{}", expected_leaf_1, expected_leaf_2));
        let expected_mid1_node2 =
            calculate_hash(&format!("{}{}", expected_leaf_3, expected_leaf_4));
        let expected_mid1_node3 =
            calculate_hash(&format!("{}{}", expected_leaf_5, expected_leaf_6));

        let expected_mid2_node1 =
            calculate_hash(&format!("{}{}", expected_mid1_node1, expected_mid1_node2));
        let expected_mid2_node2 =
            calculate_hash(&format!("{}{}", expected_mid1_node3, expected_mid1_node3));

        // Function to verify the proof
        fn verify_proof(proof: Vec<(String, bool)>, expected_proof: Vec<(String, bool)>) {
            assert_eq!(proof.len(), expected_proof.len());
            for ((elem1, is_right1), (expected_elem, expected_is_right)) in
                proof.iter().zip(expected_proof.iter())
            {
                assert_eq!(elem1, expected_elem);
                assert_eq!(is_right1, expected_is_right);
            }
        }

        // Test proofs for each leaf
        {
            let proof = tree.get_merkle_proof(0).unwrap();
            println!("Received proof: {:?}", proof);
            let expected_proof = vec![
                (expected_leaf_2.clone(), true),
                (expected_mid1_node2.clone(), true),
                (expected_mid2_node2.clone(), true),
            ];
            verify_proof(proof, expected_proof);
        }
        {
            let proof = tree.get_merkle_proof(1).unwrap();
            println!("Received proof: {:?}", proof);
            let expected_proof = vec![
                (expected_leaf_1.clone(), false),
                (expected_mid1_node2.clone(), true),
                (expected_mid2_node2.clone(), true),
            ];
            verify_proof(proof, expected_proof);
        }
        {
            let proof = tree.get_merkle_proof(2).unwrap();
            println!("Received proof: {:?}", proof);
            let expected_proof = vec![
                (expected_leaf_4.clone(), true),
                (expected_mid1_node1.clone(), false),
                (expected_mid2_node2.clone(), true),
            ];
            verify_proof(proof, expected_proof);
        }
        {
            let proof = tree.get_merkle_proof(3).unwrap();
            println!("Received proof: {:?}", proof);
            let expected_proof = vec![
                (expected_leaf_3.clone(), false),
                (expected_mid1_node1.clone(), false),
                (expected_mid2_node2.clone(), true),
            ];
            verify_proof(proof, expected_proof);
        }
        {
            let proof = tree.get_merkle_proof(4).unwrap();
            println!("Received proof: {:?}", proof);
            let expected_proof = vec![
                (expected_leaf_5.clone(), true),
                (expected_mid1_node3.clone(), true),
                (expected_mid2_node1.clone(), false),
            ];
            verify_proof(proof, expected_proof);
        }
    }
}
