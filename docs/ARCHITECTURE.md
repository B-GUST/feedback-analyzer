# Arquitectura Técnica - Feedback Analyzer

## Diagrama de Flujo de Datos

```
                    ┌─────────────────┐
                    │   Feedback      │
                    │   (texto)       │
                    └────────┬────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │  Validación     │
                    │  (Pydantic)     │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
              ▼              ▼              ▼
     ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
     │  Language   │ │  Sentiment  │ │  Keywords   │
     │  Detection  │ │  Analysis   │ │  Extraction │
     │  (whatlang) │ │  (candle)   │ │  (TF-IDF)   │
     └──────┬──────┘ └──────┬──────┘ └──────┬──────┘
            │               │               │
            └───────────────┼───────────────┘
                            │
                            ▼
                   ┌─────────────────┐
                   │  Reward         │
                   │  Calculation    │
                   │  (Python)       │
                   └────────┬────────┘
                            │
                            ▼
                   ┌─────────────────┐
                   │  Store +        │
                   │  Update Metrics │
                   └────────┬────────┘
                            │
              ┌─────────────┼─────────────┐
              │             │             │
              ▼             ▼             ▼
     ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
     │  SQLite     │ │  Real-time  │ │  Cache      │
     │  Database   │ │  Metrics    │ │  (RAM)      │
     └─────────────┘ └─────────────┘ └─────────────┘
```

## Flujo de Exportación con Integridad

```
                    ┌─────────────────┐
                    │  Records        │
                    │  (DataFrame)    │
                    └────────┬────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │  Hash each      │
                    │  record SHA-256 │
                    └────────┬────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │  Build Merkle   │
                    │  Tree           │
                    └────────┬────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │  Sign root      │
                    │  Ed25519        │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
              ▼              ▼              ▼
     ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
     │  JSON       │ │  Parquet    │ │  Parquet    │
     │  Signed     │ │  Plain      │ │  Encrypted  │
     │             │ │             │ │  (AES-GCM)  │
     └─────────────┘ └─────────────┘ └─────────────┘
```

## Modelo de Datos

### FeedbackRecord
```rust
#[derive(Serialize, Deserialize)]
struct FeedbackRecord {
    id: String,              // UUID v4
    text: String,            // Texto original
    language: String,        // Código ISO 639-1
    sentiment: f64,          // -1.0 a 1.0
    sentiment_label: String, // positive/negative/neutral
    category: String,        // Clasificación de riesgo
    keywords: Vec<String>,   // Top keywords extraídas
    quality_score: f64,      // 0.0 a 1.0
    created_at: String,      // ISO 8601
    hash: String,            // SHA-256 del registro
}
```

### Reward
```python
@dataclass
class Reward:
    user_id: str
    points: int
    level: str              # novice/collaborator/analyst/expert/master
    badges: List[str]       # Lista de badges ganados
    total_points: int       # Acumulado histórico
    streak_days: int        # Días consecutivos
    last_contribution: datetime
```

### SignedBatch
```rust
#[derive(Serialize, Deserialize)]
struct SignedBatch {
    version: String,           // "1.0"
    metadata: BatchMetadata,   // stats del batch
    records: Vec<FeedbackRecord>,
    merkle_root: Vec<u8>,      // Root hash del Merkle tree
    signature: Vec<u8>,        // Firma Ed25519
    public_key: Vec<u8>,       // Clave pública para verificación
}
```

## API Endpoints

### POST /analyze
Analiza un solo feedback.
```json
// Request
{ "text": "El producto es excelente pero el servicio al cliente falló" }

// Response
{
  "id": "uuid-abc-123",
  "sentiment": -0.2,
  "sentiment_label": "mixed",
  "category": "service_quality",
  "keywords": ["servicio", "cliente", "producto"],
  "quality_score": 0.85,
  "reward": {
    "points": 18,
    "badges": ["quality_star"]
  }
}
```

### POST /analyze/batch
Analiza múltiples feedbacks y retorna JSON/Parquet firmado.
```json
// Request
{
  "feedbacks": [
    { "text": "..." },
    { "text": "..." }
  ],
  "output_format": "json_signed",  // or "parquet" or "parquet_encrypted"
  "encryption_key": "..."          // only for parquet_encrypted
}

// Response (json_signed)
{
  "version": "1.0",
  "metadata": { "total_records": 2, "avg_sentiment": 0.3 },
  "records": [...],
  "integrity": {
    "merkle_root": "hex...",
    "signature": "ed25519:hex...",
    "public_key": "hex..."
  }
}
```

### GET /stats
Métricas en tiempo real.
```json
{
  "total_analyzed": 15234,
  "sentiment_distribution": {
    "positive": 0.62,
    "neutral": 0.23,
    "negative": 0.15
  },
  "avg_sentiment": 0.34,
  "top_keywords": ["producto", "servicio", "calidad", "precio"],
  "trend": "improving",
  "alerts": [],
  "reward_totals": {
    "total_points_distributed": 284500,
    "active_users": 89
  }
}
```

### GET /rewards/{user_id}
Recompensas de un usuario.
```json
{
  "user_id": "user-123",
  "level": "analyst",
  "total_points": 1250,
  "badges": ["first_feedback", "streak_7", "quality_star"],
  "streak_days": 12,
  "next_level_points": 2000,
  "history": [...]
}
```

### GET /verify/{batch_id}
Verifica integridad de un batch.
```json
{
  "valid": true,
  "merkle_verified": true,
  "signature_verified": true,
  "record_count": 1500,
  "created_at": "2026-06-03T23:00:00Z"
}
```

## Seguridad

### Claves Criptográficas
- **Ed25519**: 32 bytes private key, 32 bytes public key
- **Almacenamiento**: Variables de entorno o Vault
- **Rotación**: Soporte para múltiples claves (key rotation)

### Encriptación Parquet
- **Algoritmo**: AES-256-GCM
- **IV**: Generado aleatoriamente por archivo
- **Tag**: 16 bytes de autenticación
- **Header**: `[magic][iv][tag][encrypted_data]`

### Verificación de Integridad
1. Cualquier persona con la clave pública puede verificar
2. Merkle proof permite verificar un registro individual sin todo el dataset
3. Timestamp firmado previene replay attacks

## Benchmarks Esperados

| Operación | Python Puro | Rust (candle) | Speedup |
|-----------|-------------|---------------|---------|
| Sentiment (1 texto) | 45ms | 2ms | 22x |
| Sentiment (1000 textos) | 45s | 1.2s | 37x |
| Keyword extraction | 12ms | 0.3ms | 40x |
| SHA-256 (1MB) | 3ms | 0.1ms | 30x |
| Parquet write (10K rows) | 180ms | 15ms | 12x |
| Merkle tree (10K leaves) | 250ms | 8ms | 31x |
