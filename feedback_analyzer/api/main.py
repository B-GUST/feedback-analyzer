"""FastAPI application for Feedback Analyzer."""

from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware

from feedback_analyzer.api.models import (
    FeedbackInput,
    FeedbackAnalysis,
    BatchAnalysisRequest,
    BatchAnalysisResponse,
    StatsResponse,
    UserRewards,
    VerificationResponse,
)
from feedback_analyzer.core.analyzer import analyzer
from feedback_analyzer.core.output import output_generator

app = FastAPI(
    title="Feedback Analyzer API",
    description="Feedback analysis with Rust-powered NLP and cryptographic integrity",
    version="0.1.0",
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.get("/")
async def root():
    """Health check endpoint."""
    return {
        "service": "feedback-analyzer",
        "version": "0.1.0",
        "status": "healthy",
    }


@app.post("/analyze", response_model=FeedbackAnalysis)
async def analyze_feedback(feedback: FeedbackInput):
    """Analyze a single feedback item."""
    try:
        result = analyzer.analyze(feedback)
        return result
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/analyze/batch")
async def analyze_batch(request: BatchAnalysisRequest):
    """Analyze a batch of feedbacks with optional signing."""
    try:
        analyses, metadata = analyzer.analyze_batch(request.feedbacks)
        
        if request.output_format == "json":
            return {
                "metadata": metadata.model_dump(),
                "records": [a.model_dump() for a in analyses],
            }
        elif request.output_format == "json_signed":
            return output_generator.generate_signed_json(analyses, metadata)
        else:
            raise HTTPException(
                status_code=400,
                detail=f"Unsupported output format: {request.output_format}"
            )
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/stats", response_model=StatsResponse)
async def get_stats():
    """Get real-time statistics."""
    try:
        dashboard = analyzer.metrics.get_dashboard()
        return StatsResponse(**dashboard)
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/rewards/{user_id}", response_model=UserRewards)
async def get_user_rewards(user_id: str):
    """Get reward summary for a user."""
    try:
        rewards = analyzer.rewards.get_user_rewards(user_id)
        return UserRewards(**rewards)
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/analyze/{feedback_id}/reward")
async def get_feedback_reward(
    feedback_id: str,
    feedback: FeedbackInput,
    user_id: str = "anonymous",
):
    """Calculate reward for a specific feedback."""
    try:
        analysis = analyzer.analyze(feedback)
        reward = analyzer.rewards.calculate_reward(
            user_id=user_id,
            text=feedback.text,
            sentiment_score=analysis.sentiment_score,
            quality_score=analysis.quality_score,
            category=analysis.category,
        )
        return {
            "feedback_id": feedback_id,
            "analysis": analysis.model_dump(),
            "reward": {
                "points": reward.points,
                "level": reward.level,
                "badges": reward.badges,
                "total_points": reward.total_points,
                "streak_days": reward.streak_days,
            },
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/verify/{batch_id}", response_model=VerificationResponse)
async def verify_batch(batch_id: str):
    """Verify integrity of a batch (placeholder - would check stored batches)."""
    # In production, this would retrieve the batch from storage and verify
    return VerificationResponse(
        valid=True,
        merkle_verified=True,
        signature_verified=True,
        record_count=0,
        created_at="",
    )


@app.get("/health")
async def health():
    """Detailed health check."""
    try:
        import feedback_analyzer.rust_analyzer as rust
        rust_available = True
    except ImportError:
        rust_available = False
    
    return {
        "status": "healthy",
        "rust_available": rust_available,
        "version": "0.1.0",
    }
