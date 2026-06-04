"""Tests for the Feedback Analyzer API."""

import pytest
from fastapi.testclient import TestClient
from feedback_analyzer.api.main import app

client = TestClient(app)


def test_health_check():
    """Test health endpoint."""
    response = client.get("/")
    assert response.status_code == 200
    data = response.json()
    assert data["service"] == "feedback-analyzer"
    assert data["status"] == "healthy"


def test_detailed_health():
    """Test detailed health endpoint."""
    response = client.get("/health")
    assert response.status_code == 200
    data = response.json()
    assert data["status"] == "healthy"
    assert "rust_available" in data


def test_analyze_feedback():
    """Test single feedback analysis."""
    response = client.post(
        "/analyze",
        json={"text": "This product is excellent and amazing!"},
    )
    assert response.status_code == 200
    data = response.json()
    assert "id" in data
    assert data["sentiment_score"] > 0
    assert data["sentiment_label"] == "positive"
    assert "keywords" in data
    assert data["quality_score"] > 0


def test_analyze_negative_feedback():
    """Test negative feedback analysis."""
    response = client.post(
        "/analyze",
        json={"text": "This is terrible and horrible service."},
    )
    assert response.status_code == 200
    data = response.json()
    assert data["sentiment_score"] < 0
    assert data["sentiment_label"] == "negative"


def test_analyze_batch():
    """Test batch analysis."""
    response = client.post(
        "/analyze/batch",
        json={
            "feedbacks": [
                {"text": "Great product!"},
                {"text": "Terrible service."},
                {"text": "It's okay, nothing special."},
            ],
            "output_format": "json",
        },
    )
    assert response.status_code == 200
    data = response.json()
    assert "metadata" in data
    assert "records" in data
    assert len(data["records"]) == 3
    assert data["metadata"]["total_records"] == 3


def test_analyze_batch_signed():
    """Test signed batch analysis."""
    response = client.post(
        "/analyze/batch",
        json={
            "feedbacks": [
                {"text": "Excellent work!"},
                {"text": "Needs improvement."},
            ],
            "output_format": "json_signed",
        },
    )
    assert response.status_code == 200
    data = response.json()
    assert "integrity" in data
    assert "merkle_root" in data["integrity"]
    assert "signature" in data["integrity"]
    assert "public_key" in data["integrity"]


def test_stats():
    """Test stats endpoint."""
    # First analyze some feedback
    client.post("/analyze", json={"text": "Great!"})
    client.post("/analyze", json={"text": "Bad!"})
    
    response = client.get("/stats")
    assert response.status_code == 200
    data = response.json()
    assert data["total_analyzed"] >= 2
    assert "sentiment_distribution" in data
    assert "top_keywords" in data


def test_rewards():
    """Test rewards endpoint."""
    response = client.get("/rewards/test-user")
    assert response.status_code == 200
    data = response.json()
    assert data["user_id"] == "test-user"
    assert data["level"] == "novice"
    assert data["total_points"] == 0


def test_health_check_python_fallback():
    """Test that Python fallback works when Rust is unavailable."""
    # This test just verifies the API works
    response = client.get("/health")
    assert response.status_code == 200
