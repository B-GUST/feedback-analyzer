# Feedback Analyzer

A high-performance feedback analysis system built with **Python + Rust hybrid architecture**. Features cryptographic integrity, gamification rewards, and real-time metrics.

## Features

- **Rust-Powered NLP**: Sentiment analysis, language detection, and keyword extraction via PyO3
- **Cryptographic Integrity**: SHA-256 hashing, Ed25519 signatures, and Merkle trees
- **Gamification System**: Points, levels, and badges for contributors
- **Real-Time Metrics**: Live dashboard with sentiment trends and alerts
- **Signed Output**: JSON with cryptographic proofs and encrypted Parquet files

## Tech Stack

| Layer | Technology |
|-------|------------|
| API | FastAPI (Python) |
| NLP Core | whatlang + custom tokenizers (Rust) |
| Cryptography | ed25519-dalek + sha2 (Rust) |
| Data | Pydantic + SQLAlchemy |
| Build | maturin + PyO3 |

## Quick Start

### Prerequisites

- Python 3.10+
- Rust 1.70+
- maturin

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/feedback-analyzer.git
cd feedback-analyzer

# Create virtual environment
python -m venv .venv
source .venv/bin/activate  # Linux/Mac
# or .venv\Scripts\activate  # Windows

# Install dependencies and build Rust extension
pip install -e .
```

### Run the API

```bash
uvicorn feedback_analyzer.api.main:app --reload
```

The API will be available at `http://localhost:8000`

## API Endpoints

### `POST /analyze`
Analyze a single feedback item.

```bash
curl -X POST http://localhost:8000/analyze \
  -H "Content-Type: application/json" \
  -d '{"text": "This product is excellent!"}'
```

Response:
```json
{
  "id": "uuid-...",
  "text": "This product is excellent!",
  "language": "en",
  "sentiment_score": 1.0,
  "sentiment_label": "positive",
  "category": "general",
  "keywords": ["excellent", "product"],
  "quality_score": 0.85
}
```

### `POST /analyze/batch`
Analyze multiple feedbacks with optional cryptographic signing.

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

### `GET /stats`
Real-time statistics dashboard.

### `GET /rewards/{user_id}`
User reward summary with levels and badges.

## Architecture

```
┌─────────────────────────────────────────────┐
│              Python Layer                    │
│  FastAPI + RewardsEngine + MetricsTracker   │
└─────────────────────┬───────────────────────┘
                      │ PyO3
┌─────────────────────┴───────────────────────┐
│               Rust Layer                     │
│  SHA-256 + Ed25519 + Merkle + NLP Core      │
└─────────────────────────────────────────────┘
```

## Gamification

- **Levels**: Novice → Collaborator → Analyst → Expert → Master
- **Badges**: first_feedback, streak_7, quality_star, century, diversity
- **Points**: Earn based on feedback quality, sentiment, and completeness

## Cryptographic Integrity

Every batch analysis is signed with Ed25519 and includes a Merkle root for verification:

```json
{
  "integrity": {
    "merkle_root": "abc123...",
    "signature": "ed25519:...",
    "public_key": "..."
  }
}
```

## Development

### Run Tests

```bash
pytest tests/
```

### Build Rust Extension

```bash
maturin develop
```

### Project Structure

```
feedback-analyzer/
├── src/                    # Rust source code
│   ├── crypto/            # Hashing, signing, Merkle
│   ├── nlp/               # Language detection, keywords
│   └── data/              # Metrics, sentiment scoring
├── feedback_analyzer/     # Python package
│   ├── api/               # FastAPI routes
│   └── core/              # Business logic
├── tests/                 # Test suite
└── docs/                  # Documentation
```

## License

MIT
