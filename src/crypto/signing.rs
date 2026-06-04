use pyo3::prelude::*;
use ed25519_dalek::{SigningKey, Signer, VerifyingKey, Verifier, Signature};
use rand_core::OsRng;
use rand::RngCore;

/// Generate a new Ed25519 keypair
/// Returns (private_key_hex, public_key_hex)
#[pyfunction]
pub fn generate_keypair() -> (String, String) {
    let mut secret_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut secret_bytes);
    let signing_key = SigningKey::from_bytes(&secret_bytes);
    let verifying_key: VerifyingKey = signing_key.verifying_key();
    
    (
        hex::encode(signing_key.to_bytes()),
        hex::encode(verifying_key.as_bytes())
    )
}

/// Sign data with Ed25519
/// private_key_hex: 64 character hex string (32 bytes)
/// message: data to sign
/// Returns signature as hex string
#[pyfunction]
pub fn sign_data(private_key_hex: String, message: String) -> Result<String, PyErr> {
    let key_bytes = hex::decode(&private_key_hex)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Invalid hex key: {}", e)
        ))?;
    
    let key_array: [u8; 32] = key_bytes.try_into()
        .map_err(|_| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Key must be 32 bytes (64 hex chars)".to_string()
        ))?;
    
    let signing_key = SigningKey::from_bytes(&key_array);
    let signature = signing_key.sign(message.as_bytes());
    
    Ok(hex::encode(signature.to_bytes()))
}

/// Verify Ed25519 signature
/// public_key_hex: 64 character hex string (32 bytes)
/// signature_hex: 128 character hex string (64 bytes)
/// message: original message
/// Returns true if signature is valid
#[pyfunction]
pub fn verify_signature(
    public_key_hex: String,
    signature_hex: String,
    message: String
) -> Result<bool, PyErr> {
    let key_bytes = hex::decode(&public_key_hex)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Invalid hex public key: {}", e)
        ))?;
    
    let key_array: [u8; 32] = key_bytes.try_into()
        .map_err(|_| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Public key must be 32 bytes (64 hex chars)".to_string()
        ))?;
    
    let sig_bytes = hex::decode(&signature_hex)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Invalid hex signature: {}", e)
        ))?;
    
    let sig_array: [u8; 64] = sig_bytes.try_into()
        .map_err(|_| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Signature must be 64 bytes (128 hex chars)".to_string()
        ))?;
    
    let verifying_key = VerifyingKey::from_bytes(&key_array)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Invalid public key: {}", e)
        ))?;
    
    let signature = Signature::from_bytes(&sig_array);
    
    match verifying_key.verify(message.as_bytes(), &signature) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let (private, public) = generate_keypair();
        assert_eq!(private.len(), 64);
        assert_eq!(public.len(), 64);
    }

    #[test]
    fn test_sign_and_verify() {
        let (private, public) = generate_keypair();
        let message = "test message";
        
        let signature = sign_data(private, message.to_string()).unwrap();
        assert_eq!(signature.len(), 128);
        
        let valid = verify_signature(public, signature, message.to_string()).unwrap();
        assert!(valid);
    }

    #[test]
    fn test_invalid_signature() {
        let (private, public) = generate_keypair();
        let signature = sign_data(private, "message 1".to_string()).unwrap();
        
        let valid = verify_signature(public, signature, "message 2".to_string()).unwrap();
        assert!(!valid);
    }
}
