# Feedback Analyzer

> **Status: Alpha (MVP)** — Software en fase de desarrollo activo.

Sistema de análisis de feedback con arquitectura híbrida **Python + Rust**, diseñado para ofrecer análisis de sentimiento de alto rendimiento con integridad criptográfica verificable.

---

## Características Principales

- **Motor NLP en Rust**: Sentiment analysis, detección de idioma y extracción de keywords via candle
- **Integridad Criptográfica**: SHA-256, firmas Ed25519 y árboles Merkle para verificación de datos
- **Sistema de Recompensas**: Gamificación con niveles (Novato → Maestro) y badges
- **Métricas en Tiempo Real**: Dashboard con distribución de sentimiento y alertas automáticas
- **Salida Firmada**: JSON con firmas criptográficas o Parquet encriptado con AES-256-GCM

---

## Stack Tecnológico

| Capa | Tecnología | Propósito |
|------|------------|-----------|
| API | FastAPI (Python) | Endpoints REST con auto-documentación |
| NLP Core | candle + tokenizers (Rust) | Inferencia de ML de alto rendimiento |
| Criptografía | ed25519-dalek + sha2 (Rust) | Firmas y hashes verificables |
| Datos | Pydantic + SQLAlchemy | Modelos y persistencia |
| Build | maturin + PyO3 | Bridge Python ↔ Rust |

---

## Instalación

### Prerrequisitos

- Python 3.10+
- Rust 1.70+
- maturin (`pip install maturin`)

### Pasos

```bash
# Clonar el repositorio
git clone https://github.com/tu-usuario/feedback-analyzer.git
cd feedback-analyzer

# Crear entorno virtual
python -m venv .venv
source .venv/bin/activate  # Linux/Mac
# o .venv\Scripts\activate  # Windows

# Instalarr dependencias y compilar extensión Rust
pip install -e .
```

---

## Uso

### Iniciar el Servidor

```bash
uvicorn feedback_analyzer.api.main:app --reload
```

La API estará disponible en `http://localhost:8000`

### Endpoints Disponibles

#### `POST /analyze`
Analiza un solo feedback.

```bash
curl -X POST http://localhost:8000/analyze \
  -H "Content-Type: application/json" \
  -d '{"text": "This product is excellent!"}'
```

**Respuesta:**
```json
{
  "id": "uuid-...",
  "text": "This product is excellent!",
  "language": "en",
  "sentiment_score": 1.0,
  "sentiment_label": "positive",
  "category": "general",
  "keywords": ["excellent", "product"],
  "quality_score": 0.85,
  "created_at": "2026-06-04T05:30:00Z"
}
```

#### `POST /analyze/batch`
Analiza múltiples feedbacks con opción de firma criptográfica.

```bash
curl -X POST http://localhost:8000/analyze/batch \
  -H "Content-Type: application/json" \
  -d '{
    "feedbacks": [
      {"text": "Great product!"},
      {"text": "Terrible service."}
    ],
    "output_format": "json_signed"
  }'
```

**Formatos de salida soportados:**
- `json` — Respuesta JSON estándar
- `json_signed` — JSON con firma Ed25519 y Merkle root
- `parquet` — Archivo Parquet (próximamente)
- `parquet_encrypted` — Parquet encriptado con AES-256-GCM (próximamente)

#### `GET /stats`
Dashboard de métricas en tiempo real.

```json
{
  "total_analyzed": 15234,
  "sentiment_distribution": {
    "positive": 0.62,
    "neutral": 0.23,
    "negative": 0.15
  },
  "avg_sentiment": 0.34,
  "top_keywords": ["producto", "servicio", "calidad"],
  "trend": "improving",
  "alerts": []
}
```

#### `GET /rewards/{user_id}`
Resumen de recompensas del usuario.

```json
{
  "user_id": "user-123",
  "level": "analyst",
  "total_points": 1250,
  "badges": ["first_feedback", "streak_7", "quality_star"],
  "streak_days": 12,
  "next_level_points": 2000
}
```

#### `GET /health`
Health check detallado.

---

## Sistema de Recompensas

### Niveles

| Nivel | Puntos Requeridos | Beneficios |
|-------|-------------------|------------|
| Novato | 0 - 100 | Acceso básico |
| Colaborador | 101 - 500 | Reportes básicos |
| Analista | 501 - 2,000 | Reportes detallados + export |
| Experto | 2,001 - 5,000 | API access + prioridad |
| Maestro | 5,001+ | Acceso completo + governance |

### Badges Disponibles

- `first_feedback` — Primer feedback enviado
- `streak_7` — 7 días consecutivos de contribución
- `streak_30` — 30 días consecutivos
- `quality_star` — Feedback con calidad > 90%
- `diversity` — Feedback en 3+ categorías
- `century` — 100 feedbacks enviados

---

## Integridad Criptográfica

Cada batch de análisis genera:

1. **SHA-256 Hash** por registro individual
2. **Árbol Merkle** sobre el batch completo
3. **Firma Ed25519** del Merkle root

```json
{
  "integrity": {
    "merkle_root": "abc123...",
    "signature": "ed25519:def456...",
    "public_key": "789ghi..."
  }
}
```

**Verificación:** Cualquier persona con la clave pública puede validar que los datos no fueron manipulados.

---

## Entrenamiento del Modelo

### Modelo Actual

- **Arquitectura**: Simple classifier (embeddings + 2 FC layers)
- **Dataset**: SST-2 (Stanford Sentiment Treebank)
- **Parámetros**: ~4M
- **Tamaño**: 16MB (SafeTensors)
- **Accuracy**: ~55% (entrenamiento rápido, optimizable)

### Re-entrenar el Modelo

```bash
# Entrenar con más datos (mayor accuracy)
./target/release/train_sentiment \
  --epochs 10 \
  --batch-size 64 \
  --max-samples 50000 \
  --learning-rate 0.00002

# El modelo se guarda en models/sentiment/trained/
```

### Parámetros de Entrenamiento

| Parámetro | Default | Descripción |
|-----------|---------|-------------|
| `--epochs` | 3 | Número de épocas |
| `--batch-size` | 128 | Tamaño del batch |
| `--max-samples` | 5000 | Muestras máxima del dataset |
| `--learning-rate` | 0.00005 | Tasa de aprendizaje |
| `--max-length` | 64 | Longitud máxima de tokens |

---

## Estructura del Proyecto

```
feedback-analyzer/
├── Cargo.toml                    # Configuración Rust workspace
├── pyproject.toml                # Configuración Python + maturin
├── src/                          # Código Rust
│   ├── lib.rs                    # Entry point PyO3
│   ├── crypto/                   # Criptografía
│   │   ├── hashing.rs           # SHA-256
│   │   ├── signing.rs           # Ed25519
│   │   └── merkle.rs            # Árboles Merkle
│   ├── nlp/                      # Procesamiento de lenguaje
│   │   ├── language.rs          # Detección de idioma
│   │   ├── keywords.rs          # Extracción de keywords
│   │   └── candle_sentiment.rs  # Modelo de sentimiento
│   └── data/
│       └── metrics.rs           # Scoring de calidad
├── src/bin/
│   └── train_sentiment.rs       # Script de entrenamiento
├── feedback_analyzer/            # Paquete Python
│   ├── api/
│   │   ├── main.py              # FastAPI app
│   │   └── models.py            # Schemas Pydantic
│   └── core/
│       ├── analyzer.py          # Orquestador principal
│       ├── rewards.py           # Sistema de recompensas
│       ├── metrics.py           # Métricas en tiempo real
│       └── output.py            # Generador JSON/Parquet
├── models/sentiment/             # Modelo de sentimiento
│   ├── config.json              # Configuración BERT
│   ├── vocab.txt                # Vocabulario
│   ├── data/                    # Dataset SST-2
│   └── trained/                 # Modelo entrenado
│       └── model.safetensors
└── tests/
    └── test_api.py              # Suite de tests
```

---

## Desarrollo

### Ejecutar Tests

```bash
pytest tests/
```

### Compilar Extensión Rust

```bash
maturin develop
```

### Build de Producción

```bash
maturin build --release
```

---

## Roadmap (Alpha → Beta)

- [x] Arquitectura Python + Rust
- [x] Criptografía Ed25519 + Merkle
- [x] Sistema de recompensas
- [x] Métricas en tiempo real
- [x] Modelo de sentimiento entrenado
- [ ] Entrenamiento con modelo BERT completo
- [ ] Soporte Parquet encriptado
- [ ] Persistencia en base de datos
- [ ] Autenticación JWT
- [ ] Rate limiting
- [ ] Dashboard web

---

## Licencia

MIT

---

**Alpha v0.1.0** — Desarrollado como proyecto de portafolio demostrando arquitectura híbrida Python + Rust con aplicaciones criptográficas.
