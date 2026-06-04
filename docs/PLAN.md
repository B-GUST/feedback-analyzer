# Feedback Analyzer - Plan Maestro

## Visión General

Sistema de análisis de feedback con:
- **NLP de alto rendimiento** (Rust + candle)
- **Sistema de recompensas/gamificación** (contribuidores obtienen tokens de reputación)
- **Métricas en tiempo real** (métricas agregadas actualizadas al vuelo)
- **Integridad criptográfica** (firmas Ed25519 + árboles Merkle)
- **Salida encriptada** (JSON/Parquet con firma verificable)

**Stack**: Python (API + orquestación) + Rust (NLP + criptografía + procesamiento paralelo)

---

## Análisis de Repositorios Existentes

| Repo | Funcionalidad Clave | Reutilizable |
|------|---------------------|--------------|
| B-GUST/feedback-analyzer | Solo README, sin código | Planificación conceptual |
| RezaGooner/Sentiment-Survey-Analyzer | LSTM + Word2Vec para persa, batch processing | Pipeline de NLP, arquitectura de modelo |
| Divya-Shivanand/Student-Feedback-Analyzer | FastAPI + VADER, keywords, stats | API pattern, extracción de keywords |

---

## Arquitectura del Sistema

```
┌─────────────────────────────────────────────────────────┐
│                    Python Layer                          │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────┐  │
│  │  FastAPI    │  │  Orquestador │  │  Gestor de    │  │
│  │  Endpoints  │──│  de Análisis  │──│  Recompensas  │  │
│  └─────────────┘  └──────────────┘  └───────────────┘  │
│         │                │                   │           │
│  ┌──────┴────────────────┴───────────────────┴────────┐ │
│  │           PyO3 Bridge (maturin)                     │ │
│  └─────────────────────────┬──────────────────────────┘ │
└────────────────────────────┼────────────────────────────┘
                             │
┌────────────────────────────┼────────────────────────────┐
│                    Rust Layer                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │  candle NLP  │  │  Criptografía │  │  Polars +    │  │
│  │  Sentiment   │  │  Ed25519 +    │  │  Parquet +   │  │
│  │  + Keywords  │  │  SHA-256 +    │  │  Rayon       │  │
│  │              │  │  Merkle       │  │              │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
└─────────────────────────────────────────────────────────┘
```

---

## Módulos del Sistema

### 1. Motor de Análisis NLP (Rust)

**Tecnologías**: `candle` + `tokenizers` (Hugging Face)

**Funcionalidades**:
- Análisis de sentimiento con modelos transformer (distilbert, bert)
- Clasificación de categorías de riesgo psicológico
- Extracción de keywords con TF-IDF nativo
- Detección de idioma con `whatlang`

**Ventaja Rust**: Procesamiento paralelo con `rayon`, 10-50x más rápido que Python para inference

### 2. Sistema de Recompensas / Gamificación

**Concepto**: Cada feedback analizado genera "reputación" para el contribuidor

**Componentes**:
```python
class RewardsEngine:
    def calculate_reward(self, feedback: Feedback) -> Reward:
        # Recompensa base por feedback válido
        base = 10
        
        # Bonus por calidad del texto (longitud, completitud)
        quality_bonus = self.quality_score(feedback.text) * 5
        
        # Bonus por sentimiento extremo (más datos valiosos)
        sentiment_bonus = abs(feedback.sentiment) * 3
        
        # Bonus por ser respuesta completa (survey largo)
        completeness_bonus = feedback.completeness * 2
        
        return Reward(
            points=base + quality_bonus + sentiment_bonus + completeness_bonus,
            level=self.calculate_level(total_points),
            badges=self.check_badges(feedback)
        )
```

**Sistema de Niveles**:
| Nivel | Puntos Requeridos | Beneficio |
|-------|-------------------|-----------|
| Novato | 0-100 | - |
| Colaborador | 101-500 | Acceso a reportes básicos |
| Analista | 501-2000 | Reportes detallados + export |
| Experto | 2001-5000 | API access + prioridad |
| Maestro | 5001+ | Acceso completo + governance |

**Badges**:
- `first_feedback` - Primer feedback enviado
- `streak_7` - 7 días consecutivos
- `quality_star` - Feedback con score de calidad > 0.9
- `diversity` - Feedback en múltiples categorías

### 3. Métricas en Tiempo Real

**Componentes**:
```python
class RealTimeMetrics:
    def __init__(self):
        self.window = SlidingWindow(size=1000)  # Últimos 1000 feedbacks
        self.counters = {
            'total_analyzed': AtomicCounter(),
            'sentiment_distribution': DistributionCounter(),
            'avg_response_time': MovingAverage(),
        }
    
    def get_dashboard(self) -> dict:
        return {
            'total_analyzed': self.counters['total_analyzed'].get(),
            'sentiment_last_hour': self.window.get_sentiment_distribution(),
            'avg_sentiment': self.window.get_average_sentiment(),
            'top_keywords': self.window.get_top_keywords(n=10),
            'trend': self.window.get_trend(),  # improving/declining/stable
            'alerts': self.check_alerts(),  # spikes negativos, etc.
        }
```

**Alertas Automáticas**:
- Spike de sentimiento negativo > 30% en últimos 100 feedbacks
- Caída de participación > 50% vs semana anterior
- Categoría con sentimiento promedio < -0.5

### 4. Integridad Criptográfica (Rust)

**Componentes**:
- **Firma Ed25519**: Cada batch de análisis se firma con clave privada
- **SHA-256 Hashing**: Hash verificable de cada registro individual
- **Árbol Merkle**: Permite verificar integridad sin descargar todo el dataset

**Flujo**:
```
1. Feedback llega → Hash SHA-256 del registro
2. Batch de N feedbacks → Árbol Merkle → Root hash
3. Root hash + metadata → Firmar con Ed25519
4. Output incluye: datos + firma + Merkle proof
```

**Verificación** (cualquier persona puede validar):
```python
def verify_batch(signed_batch: SignedBatch, public_key: bytes) -> bool:
    # 1. Verificar firma Ed25519
    if not ed25519_verify(signed_batch.data, signed_batch.signature, public_key):
        return False
    
    # 2. Verificar Merkle root
    computed_root = compute_merkle_root(signed_batch.data.records)
    if computed_root != signed_batch.merkle_root:
        return False
    
    return True
```

### 5. Salida Encriptada (JSON/Parquet)

**Formatos de Salida**:

**JSON firmado**:
```json
{
  "version": "1.0",
  "metadata": {
    "created_at": "2026-06-03T23:30:00Z",
    "total_records": 1500,
    "analyzer_version": "1.0.0"
  },
  "records": [...],
  "integrity": {
    "merkle_root": "abc123...",
    "signature": "ed25519:...",
    "public_key": "..."
  }
}
```

**Parquet encriptado**:
```
1. Escribir Parquet con Arrow/Polars
2. Cifrar archivo con AES-256-GCM
3. Incluir IV + tag en header
4. Output: archivo .parquet.enc
```

---

## Estructura del Proyecto

```
feedback-analyzer/
├── Cargo.toml                    # Rust workspace
├── pyproject.toml                # Python config
├── rust_analyzer/                # Crate Rust principal
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                # Entry point + PyO3 bindings
│       ├── nlp/
│       │   ├── mod.rs
│       │   ├── sentiment.rs      # candle inference
│       │   ├── keywords.rs       # TF-IDF extraction
│       │   └── language.rs       # whatlang detection
│       ├── crypto/
│       │   ├── mod.rs
│       │   ├── signing.rs        # Ed25519
│       │   ├── hashing.rs        # SHA-256
│       │   └── merkle.rs         # Merkle tree
│       └── data/
│           ├── mod.rs
│           ├── parquet.rs        # Arrow/Parquet I/O
│           └── metrics.rs        # Real-time aggregation
├── feedback_analyzer/            # Python package
│   ├── __init__.py
│   ├── api/
│   │   ├── __init__.py
│   │   ├── main.py               # FastAPI app
│   │   ├── routes/
│   │   │   ├── analyze.py        # POST /analyze
│   │   │   ├── batch.py          # POST /analyze/batch
│   │   │   ├── stats.py          # GET /stats
│   │   │   └── rewards.py        # GET /rewards/{user}
│   │   └── models.py             # Pydantic schemas
│   ├── core/
│   │   ├── __init__.py
│   │   ├── analyzer.py           # Orquestador principal
│   │   ├── rewards.py            # Sistema de recompensas
│   │   ├── metrics.py            # Métricas en tiempo real
│   │   └── output.py             # Generador JSON/Parquet
│   ├── storage/
│   │   ├── __init__.py
│   │   ├── database.py           # SQLAlchemy + SQLite
│   │   └── cache.py              # Redis (opcional)
│   └── config.py                 # Settings
├── tests/
│   ├── test_nlp.rs
│   ├── test_crypto.rs
│   ├── test_rewards.py
│   └── test_api.py
├── models/                       # Modelos pre-entrenados
│   └── sentiment/
│       └── config.json
├── docs/
│   ├── PLAN.md
│   └── ARCHITECTURE.md
├── Dockerfile
└── README.md
```

---

## Fases de Implementación

### Fase 1: Fundación (Semana 1)
- [ ] Setup Rust workspace + PyO3 con maturin
- [ ] Implementar hashing SHA-256 + firma Ed25519 en Rust
- [ ] Python FastAPI skeleton con endpoints básicos
- [ ] Schema de base de datos para feedback + rewards

### Fase 2: Motor NLP (Semana 2)
- [ ] Integrar candle + tokenizers para sentiment analysis
- [ ] Implementar extracción de keywords en Rust
- [ ] Benchmarking vs Python (mostrar speedup)
- [ ] Endpoint /analyze funcionando end-to-end

### Fase 3: Recompensas y Métricas (Semana 3)
- [ ] Implementar RewardsEngine con niveles y badges
- [ ] Sistema de métricas en tiempo real con SlidingWindow
- [ ] Dashboard de stats en /stats endpoint
- [ ] Alertas automáticas

### Fase 4: Integridad y Salida (Semana 4)
- [ ] Árbol Merkle para batches
- [ ] Generación de JSON firmado
- [ ] Escritura Parquet encriptado (AES-256-GCM)
- [ ] Endpoint de verificación de integridad

### Fase 5: Pulido y Portfolio (Semana 5)
- [ ] Tests completos (Rust + Python)
- [ ] Documentación de API
- [ ] Dockerfile optimizado
- [ ] README con demo y screenshots
- [ ] Benchmark suite para mostrar rendimiento

---

## Dependencias Clave

### Rust
```toml
[dependencies]
pyo3 = { version = "0.28", features = ["extension-module"] }
candle-core = "0.8"
candle-transformers = "0.8"
tokenizers = "0.21"
ed25519-dalek = "2.2"
sha2 = "0.11"
serde = { version = "1", features = ["derive"] }
bincode = "1.3"
polars = { version = "0.50", features = ["lazy", "parquet"] }
arrow = "54"
parquet = "54"
rayon = "1.10"
rand_core = "0.6"
hex = "0.4"
```

### Python
```toml
[project]
dependencies = [
    "fastapi>=0.115",
    "uvicorn>=0.34",
    "pydantic>=2.10",
    "sqlalchemy>=2.0",
    "python-jose[cryptography]>=3.3",  # JWT para rewards
    "cryptography>=44.0",               # AES-256-GCM
]
```

---

## Stack Final

| Capa | Tecnología | Justificación |
|------|------------|---------------|
| API | FastAPI (Python) | Rápido de desarrollar, auto-docs |
| NLP Core | candle + tokenizers (Rust) | 10-50x más rápido que Python |
| Criptografía | ed25519-dalek + sha2 (Rust) | Firma verificable, integridad |
| Datos | Polars + Arrow (Rust) | Parquet nativo, DataFrames rápidos |
| Paralelismo | rayon (Rust) | Procesamiento multi-core |
| Recompensas | Python | Lógica de negocio flexible |
| Métricas | Python + Rust | Agregación en Rust, API en Python |
| Almacenamiento | SQLite | Simple, portátil, sin setup |

---

## Diferenciadores para Portafolio

1. **Híbrido Python+Rust**: Demuestra capacidad de trabajar con múltiples lenguajes
2. **Criptografía aplicada**: Firma Ed25519 + Merkle trees no son comunes en proyectos similares
3. **Métricas en tiempo real**: Sistema de alertas y dashboard en vivo
4. **Gamificación**: Sistema de recompensas con niveles y badges
5. **Rendimiento**: Benchmarks que muestran speedup de Rust vs Python puro
6. **Integridad verificable**: Cualquiera puede validar que los datos no fueron manipulados
