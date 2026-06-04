use pyo3::prelude::*;
use sha2::{Sha256, Digest};

/// Build a Merkle tree from leaf hashes
/// Returns the root hash
#[pyfunction]
pub fn compute_merkle_root(leaf_hashes: Vec<String>) -> Result<String, PyErr> {
    if leaf_hashes.is_empty() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Cannot compute Merkle root of empty tree".to_string()
        ));
    }
    
    let mut current: Vec<Vec<u8>> = leaf_hashes.iter()
        .map(|h| hex::decode(h).unwrap_or_else(|_| h.as_bytes().to_vec()))
        .collect();
    
    while current.len() > 1 {
        let mut next = Vec::new();
        for pair in current.chunks(2) {
            let combined = if pair.len() == 2 {
                [pair[0].as_slice(), pair[1].as_slice()].concat()
            } else {
                pair[0].clone()
            };
            let hash = Sha256::digest(&combined).to_vec();
            next.push(hash);
        }
        current = next;
    }
    
    Ok(hex::encode(&current[0]))
}

/// Generate a Merkle proof for a specific leaf
#[pyfunction]
pub fn generate_merkle_proof(leaf_hashes: Vec<String>, leaf_index: usize) -> Result<Vec<String>, PyErr> {
    if leaf_index >= leaf_hashes.len() {
        return Err(PyErr::new::<pyo3::exceptions::PyIndexError, _>(
            "Leaf index out of bounds".to_string()
        ));
    }
    
    let mut current: Vec<Vec<u8>> = leaf_hashes.iter()
        .map(|h| hex::decode(h).unwrap_or_else(|_| h.as_bytes().to_vec()))
        .collect();
    
    let mut proof = Vec::new();
    let mut index = leaf_index;
    
    while current.len() > 1 {
        let mut next = Vec::new();
        let mut i = 0;
        
        while i < current.len() {
            let left = &current[i];
            let right = if i + 1 < current.len() {
                &current[i + 1]
            } else {
                left
            };
            
            // Add sibling to proof if this pair contains our target
            if index == i && i + 1 < current.len() {
                proof.push(hex::encode(right));
            } else if index == i + 1 {
                proof.push(hex::encode(left));
            }
            
            let combined = [left.as_slice(), right.as_slice()].concat();
            next.push(Sha256::digest(&combined).to_vec());
            
            i += 2;
        }
        
        index /= 2;
        current = next;
    }
    
    Ok(proof)
}

/// Verify a Merkle proof
#[pyfunction]
pub fn verify_merkle_proof(
    leaf_hash: String,
    proof: Vec<String>,
    root_hash: String,
    leaf_index: usize
) -> Result<bool, PyErr> {
    let mut current = hex::decode(&leaf_hash)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Invalid leaf hash: {}", e)
        ))?;
    
    let mut index = leaf_index;
    
    for sibling_hex in proof {
        let sibling = hex::decode(&sibling_hex)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Invalid proof hash: {}", e)
            ))?;
        
        let combined = if index % 2 == 0 {
            [current.as_slice(), sibling.as_slice()].concat()
        } else {
            [sibling.as_slice(), current.as_slice()].concat()
        };
        
        current = Sha256::digest(&combined).to_vec();
        index /= 2;
    }
    
    Ok(hex::encode(current) == root_hash)
}

/// Build full Merkle tree and return all levels
#[pyfunction]
pub fn build_merkle_tree(leaf_hashes: Vec<String>) -> Result<Vec<Vec<String>>, PyErr> {
    if leaf_hashes.is_empty() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Cannot build Merkle tree from empty leaves".to_string()
        ));
    }
    
    let mut tree = Vec::new();
    tree.push(leaf_hashes.clone());
    
    let mut current: Vec<Vec<u8>> = leaf_hashes.iter()
        .map(|h| hex::decode(h).unwrap_or_else(|_| h.as_bytes().to_vec()))
        .collect();
    
    while current.len() > 1 {
        let mut next = Vec::new();
        let mut level = Vec::new();
        
        for pair in current.chunks(2) {
            let combined = if pair.len() == 2 {
                [pair[0].as_slice(), pair[1].as_slice()].concat()
            } else {
                pair[0].clone()
            };
            let hash = Sha256::digest(&combined).to_vec();
            level.push(hex::encode(&hash));
            next.push(hash);
        }
        
        tree.push(level);
        current = next;
    }
    
    Ok(tree)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_root_single_leaf() {
        let leaves = vec![hash_leaf("data1")];
        let root = compute_merkle_root(leaves).unwrap();
        assert!(!root.is_empty());
    }

    #[test]
    fn test_merkle_root_two_leaves() {
        let leaves = vec![hash_leaf("data1"), hash_leaf("data2")];
        let root = compute_merkle_root(leaves).unwrap();
        assert!(!root.is_empty());
    }

    #[test]
    fn test_merkle_proof_and_verify() {
        let leaves: Vec<String> = (0..4)
            .map(|i| hash_leaf(&format!("data{}", i)))
            .collect();
        
        let root = compute_merkle_root(leaves.clone()).unwrap();
        
        for (i, leaf) in leaves.iter().enumerate() {
            let proof = generate_merkle_proof(leaves.clone(), i).unwrap();
            let valid = verify_merkle_proof(
                leaf.clone(),
                proof,
                root.clone(),
                i
            ).unwrap();
            assert!(valid, "Proof failed for leaf {}", i);
        }
    }

    fn hash_leaf(data: &str) -> String {
        hex::encode(Sha256::digest(data.as_bytes()))
    }
}
