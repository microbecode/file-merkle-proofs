//This code is adjusted from an earlier version I wrote: https://github.com/microbecode/stark-from-zero/blob/master/src/merkle_tree.rs

use crate::hashing::{self, hash};

#[derive(Debug)]
pub struct MerkleTree {
    root: Option<u128>,
    levels: Vec<Vec<u128>>,
}

impl MerkleTree {
    pub fn new() -> Self {
        MerkleTree {
            root: None,
            levels: Vec::new(),
        }
    }

    pub fn build(&mut self, elements: &[u128]) {
        let mut hashes: Vec<u128> = elements.iter().map(|e| hash(*e)).collect();
        if hashes.len() % 2 != 0 {
            // If odd number, duplicate the last element
            hashes.push(hashes[hashes.len() - 1]);
        }
        let mut nodes = Vec::new();
        nodes.push(hashes.clone());

        while hashes.len() > 1 {
            let mut new_hashes = Vec::new();
            for chunk in hashes.chunks(2) {
                let hash = hashing::hash(chunk[0].wrapping_add(chunk[1]));

                new_hashes.push(hash);
            }
            nodes.push(new_hashes.clone());
            hashes = new_hashes;
        }
        self.root = hashes.pop();
        self.levels = nodes;
    }

    pub fn root(&self) -> Option<u128> {
        self.root
    }

    pub fn get_merkle_proof(&self, index: usize) -> Option<Vec<u128>> {
        if index >= self.levels[0].len() {
            return None;
        }
        let mut proof = Vec::new();
        let mut idx = index;
        for level in self.levels.iter() {
            if level.len() == 1 {
                proof.push(level[0]);
                break; // Reached the root node, no need to continue
            }
            let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
            proof.push(level[sibling_idx]);
            idx /= 2;
        }
        Some(proof)
    }
}

#[cfg(test)]
mod tests {
    use crate::hashing::hash;

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

        let elements: Vec<u128> = Vec::new();
        tree.build(&elements);

        assert_eq!(tree.root, None);
        assert_eq!(tree.levels.len(), 1);
        assert_eq!(tree.levels[0].len(), 0);
    }

    #[test]
    fn build_tree_one_element() {
        let mut tree = MerkleTree::new();

        let val: u128 = 3;
        let mut elements: Vec<u128> = Vec::new();

        elements.push(val);
        tree.build(&elements);

        let expected_leaf = hash(val);
        let expected_root = hash(expected_leaf.wrapping_add(expected_leaf));

        assert_eq!(tree.levels.len(), 2);
        assert_eq!(tree.levels[0].len(), 2);

        assert_eq!(tree.root, Some(expected_root));
        assert_eq!(tree.levels[0][0], expected_leaf);
        assert_eq!(tree.levels[0][1], expected_leaf);
    }

    #[test]
    fn build_tree_two_elements() {
        let mut tree = MerkleTree::new();

        let val1: u128 = 3;
        let val2: u128 = 4;
        let mut elements: Vec<u128> = Vec::new();

        elements.push(val1);
        elements.push(val2);
        tree.build(&elements);

        let expected_leaf_1 = hash(val1);
        let expected_leaf_2 = hash(val2);
        let expected_root = hash(expected_leaf_1.wrapping_add(expected_leaf_2));

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

        let val1: u128 = 3;
        let val2: u128 = 4;
        let val3: u128 = 5;
        let mut elements: Vec<u128> = Vec::new();

        elements.push(val1);
        elements.push(val2);
        elements.push(val3);
        tree.build(&elements);

        let expected_leaf_1 = hash(val1);
        let expected_leaf_2 = hash(val2);
        let expected_leaf_3 = hash(val3);
        let expected_leaf_4 = hash(val3);

        let expected_mid_node1 = hash(expected_leaf_1.wrapping_add(expected_leaf_2));
        let expected_mid_node2 = hash(expected_leaf_3.wrapping_add(expected_leaf_4));

        let expected_root = hash(expected_mid_node1.wrapping_add(expected_mid_node2));

        assert_eq!(tree.levels.len(), 3);
        assert_eq!(tree.levels[0].len(), 4);
        assert_eq!(tree.levels[1].len(), 2);
        assert_eq!(tree.levels[2].len(), 1);

        assert_eq!(tree.root, Some(expected_root));
        assert_eq!(tree.levels[2][0], expected_root);

        assert_eq!(tree.levels[0][0], expected_leaf_1);
        assert_eq!(tree.levels[0][1], expected_leaf_2);
        assert_eq!(tree.levels[0][2], expected_leaf_3);
        assert_eq!(tree.levels[0][3], expected_leaf_4);

        assert_eq!(tree.levels[1][0], expected_mid_node1);
        assert_eq!(tree.levels[1][1], expected_mid_node2);
    }

    #[test]
    fn get_merkle_proof_with_three_elements() {
        let mut tree = MerkleTree::new();

        let val1: u128 = 3;
        let val2: u128 = 4;
        let val3: u128 = 5;
        let mut elements: Vec<u128> = Vec::new();

        elements.push(val1);
        elements.push(val2);
        elements.push(val3);
        tree.build(&elements);

        let expected_leaf_1 = hash(val1);
        let expected_leaf_2 = hash(val2);
        let expected_leaf_3 = hash(val3);
        let expected_leaf_4 = hash(val3);

        let expected_mid_node1 = hash(expected_leaf_1.wrapping_add(expected_leaf_2));
        let expected_mid_node2 = hash(expected_leaf_3.wrapping_add(expected_leaf_4));

        let expected_root = hash(expected_mid_node1.wrapping_add(expected_mid_node2));

        // Test proofs for each leaf
        {
            let proof = tree.get_merkle_proof(0).unwrap();
            let expected_proof = vec![expected_leaf_2, expected_mid_node2, expected_root];

            assert_eq!(proof.len(), expected_proof.len());

            for (elem1, elem2) in proof.iter().zip(expected_proof.iter()) {
                assert_eq!(elem1, elem2); // Ensure each pair of corresponding elements is equal
            }
        }
        {
            let proof = tree.get_merkle_proof(1).unwrap();
            let expected_proof = vec![expected_leaf_1, expected_mid_node2, expected_root];

            assert_eq!(proof.len(), expected_proof.len());

            for (elem1, elem2) in proof.iter().zip(expected_proof.iter()) {
                assert_eq!(elem1, elem2); // Ensure each pair of corresponding elements is equal
            }
        }
        {
            let proof = tree.get_merkle_proof(2).unwrap();
            let expected_proof = vec![expected_leaf_4, expected_mid_node1, expected_root];

            assert_eq!(proof.len(), expected_proof.len());

            for (elem1, elem2) in proof.iter().zip(expected_proof.iter()) {
                assert_eq!(elem1, elem2); // Ensure each pair of corresponding elements is equal
            }
        }
        {
            let proof = tree.get_merkle_proof(3).unwrap();
            let expected_proof = vec![expected_leaf_3, expected_mid_node1, expected_root];

            assert_eq!(proof.len(), expected_proof.len());

            for (elem1, elem2) in proof.iter().zip(expected_proof.iter()) {
                assert_eq!(elem1, elem2); // Ensure each pair of corresponding elements is equal
            }
        }
    }
}
