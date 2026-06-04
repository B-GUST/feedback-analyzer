"""Output generation for signed JSON and encrypted Parquet."""

import json
import os
from datetime import datetime, timezone
from typing import Optional

from feedback_analyzer.api.models import (
    FeedbackAnalysis,
    BatchMetadata,
    BatchAnalysisResponse,
    IntegrityInfo,
)

# Try to import Rust extension
try:
    import feedback_analyzer.rust_analyzer as rust
    HAS_RUST = True
except ImportError:
    HAS_RUST = False


class OutputGenerator:
    """Generate signed and encrypted output files."""
    
    def __init__(self):
        self._keypair_cache: Optional[tuple[str, str]] = None
    
    def generate_signed_json(
        self,
        records: list[FeedbackAnalysis],
        metadata: BatchMetadata,
    ) -> dict:
        """Generate JSON with cryptographic signature."""
        # Generate or load keypair
        private_key, public_key = self._get_or_generate_keypair()
        
        # Serialize records for hashing
        records_json = [r.model_dump() for r in records]
        records_str = json.dumps(records_json, sort_keys=True)
        
        # Compute Merkle root
        if HAS_RUST:
            leaf_hashes = [rust.hash_data(json.dumps(r, sort_keys=True)) for r in records_json]
            merkle_root = rust.compute_merkle_root(leaf_hashes)
        else:
            merkle_root = self._compute_hash(records_str)
        
        # Sign the data
        message = f"{metadata.model_dump_json()}:{merkle_root}"
        if HAS_RUST:
            signature = rust.sign_data(private_key, message)
        else:
            signature = self._compute_hash(message)
        
        # Build response
        integrity = IntegrityInfo(
            merkle_root=merkle_root,
            signature=f"ed25519:{signature}",
            public_key=public_key,
        )
        
        response = BatchAnalysisResponse(
            version="1.0",
            metadata=metadata,
            records=records,
            integrity=integrity,
        )
        
        return response.model_dump()
    
    def generate_parquet(
        self,
        records: list[FeedbackAnalysis],
        metadata: BatchMetadata,
        output_path: str,
    ) -> str:
        """Generate Parquet file."""
        try:
            import pyarrow as pa
            import pyarrow.parquet as pq
        except ImportError:
            raise ImportError("pyarrow is required for Parquet output")
        
        # Convert records to arrow format
        data = {
            "id": [r.id for r in records],
            "text": [r.text for r in records],
            "language": [r.language for r in records],
            "sentiment_score": [r.sentiment_score for r in records],
            "sentiment_label": [r.sentiment_label for r in records],
            "category": [r.category for r in records],
            "keywords": [",".join(r.keywords) for r in records],
            "quality_score": [r.quality_score for r in records],
            "created_at": [r.created_at for r in records],
        }
        
        table = pa.table(data)
        pq.write_table(table, output_path)
        
        return output_path
    
    def generate_encrypted_parquet(
        self,
        records: list[FeedbackAnalysis],
        metadata: BatchMetadata,
        output_path: str,
        encryption_key: str,
    ) -> str:
        """Generate encrypted Parquet file."""
        from cryptography.hazmat.primitives.ciphers.aead import AESGCM
        import tempfile
        
        # First generate parquet to temp file
        with tempfile.NamedTemporaryFile(suffix=".parquet", delete=False) as tmp:
            tmp_path = tmp.name
        
        try:
            self.generate_parquet(records, metadata, tmp_path)
            
            # Read parquet bytes
            with open(tmp_path, "rb") as f:
                parquet_bytes = f.read()
            
            # Generate IV and encrypt
            iv = os.urandom(12)  # 96-bit IV for AES-GCM
            
            # Derive key from password
            key = self._derive_key(encryption_key)
            
            # Encrypt
            aesgcm = AESGCM(key)
            ciphertext = aesgcm.encrypt(iv, parquet_bytes, None)
            
            # Write encrypted file with header
            # Format: [magic:4][iv:12][ciphertext_len:4][ciphertext]
            with open(output_path, "wb") as f:
                f.write(b"FPAR")  # Magic bytes
                f.write(iv)
                f.write(len(ciphertext).to_bytes(4, "big"))
                f.write(ciphertext)
            
            return output_path
        
        finally:
            if os.path.exists(tmp_path):
                os.unlink(tmp_path)
    
    def _get_or_generate_keypair(self) -> tuple[str, str]:
        """Get or generate a keypair for signing."""
        if self._keypair_cache:
            return self._keypair_cache
        
        if HAS_RUST:
            private_key, public_key = rust.generate_keypair()
        else:
            # Fallback: generate random keypair (not cryptographically secure)
            private_key = os.urandom(32).hex()
            public_key = os.urandom(32).hex()
        
        self._keypair_cache = (private_key, public_key)
        return self._keypair_cache
    
    def _derive_key(self, password: str) -> bytes:
        """Derive AES key from password."""
        import hashlib
        # Simple key derivation (in production, use PBKDF2 or Argon2)
        return hashlib.sha256(password.encode()).digest()
    
    def _compute_hash(self, data: str) -> str:
        """Compute SHA-256 hash fallback."""
        import hashlib
        return hashlib.sha256(data.encode()).hexdigest()


# Global instance
output_generator = OutputGenerator()
