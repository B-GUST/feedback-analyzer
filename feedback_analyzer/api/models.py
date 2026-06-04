"""Pydantic models for API requests and responses."""

from datetime import datetime
from typing import Optional
from pydantic import BaseModel, Field


class FeedbackInput(BaseModel):
    """Single feedback input."""
    text: str = Field(..., min_length=1, max_length=5000, description="Feedback text")
    user_id: Optional[str] = Field(None, description="Optional user identifier")
    category: Optional[str] = Field(None, description="Optional feedback category")


class FeedbackAnalysis(BaseModel):
    """Analysis result for a single feedback."""
    id: str
    text: str
    language: str
    sentiment_score: float = Field(..., ge=-1.0, le=1.0)
    sentiment_label: str  # positive, negative, neutral, mixed
    category: str
    keywords: list[str]
    quality_score: float = Field(..., ge=0.0, le=1.0)
    created_at: str


class Reward(BaseModel):
    """Reward information for a feedback contribution."""
    points: int
    level: str
    badges: list[str]
    total_points: int
    streak_days: int


class BatchAnalysisRequest(BaseModel):
    """Request for batch analysis."""
    feedbacks: list[FeedbackInput] = Field(..., min_length=1, max_length=10000)
    output_format: str = Field("json", description="json, json_signed, parquet, parquet_encrypted")
    encryption_key: Optional[str] = Field(None, description="Key for parquet_encrypted format")


class BatchMetadata(BaseModel):
    """Metadata for a batch analysis."""
    version: str = "1.0"
    total_records: int
    avg_sentiment: float
    sentiment_distribution: dict[str, float]
    created_at: str
    analyzer_version: str


class IntegrityInfo(BaseModel):
    """Cryptographic integrity information."""
    merkle_root: str
    signature: str
    public_key: str


class BatchAnalysisResponse(BaseModel):
    """Response for batch analysis with integrity."""
    version: str = "1.0"
    metadata: BatchMetadata
    records: list[FeedbackAnalysis]
    integrity: IntegrityInfo


class StatsResponse(BaseModel):
    """Real-time statistics."""
    total_analyzed: int
    sentiment_distribution: dict[str, float]
    avg_sentiment: float
    top_keywords: list[str]
    trend: str  # improving, declining, stable
    alerts: list[str]
    reward_totals: dict[str, int]


class UserRewards(BaseModel):
    """User reward summary."""
    user_id: str
    level: str
    total_points: int
    badges: list[str]
    streak_days: int
    next_level_points: int
    recent_rewards: list[dict]


class VerificationResponse(BaseModel):
    """Batch verification result."""
    valid: bool
    merkle_verified: bool
    signature_verified: bool
    record_count: int
    created_at: str
