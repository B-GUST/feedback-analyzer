pub mod hashing;
pub mod signing;
pub mod merkle;

pub use hashing::hash_data;
pub use signing::{sign_data, verify_signature, generate_keypair};
pub use merkle::{build_merkle_tree, compute_merkle_root, generate_merkle_proof, verify_merkle_proof};
