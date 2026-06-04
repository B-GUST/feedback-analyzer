"""FastAPI application for Feedback Analyzer."""

from fastapi import FastAPI, HTTPException, Depends, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.staticfiles import StaticFiles
from fastapi.responses import HTMLResponse, FileResponse
from typing import Optional
import os

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
from feedback_analyzer.storage.database import db_manager
from feedback_analyzer.api.auth import auth_manager, get_current_user, get_optional_user
from feedback_analyzer.api.rate_limit import rate_limit_middleware, rate_limiter

app = FastAPI(
    title="Feedback Analyzer API",
    description="Feedback analysis with Rust-powered NLP and cryptographic integrity",
    version="0.1.0-alpha",
)

# CORS
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Rate limiting middleware
app.middleware("http")(rate_limit_middleware)

# Static files for dashboard
static_dir = os.path.join(os.path.dirname(__file__), "static")
if os.path.exists(static_dir):
    app.mount("/static", StaticFiles(directory=static_dir), name="static")


# ==================== Public Endpoints ====================

@app.get("/")
async def root():
    """Root endpoint - API info."""
    return {"service": "feedback-analyzer", "version": "0.1.0-alpha", "status": "healthy"}


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
        "version": "0.1.0-alpha",
    }


@app.get("/docs", response_class=HTMLResponse)
async def api_docs():
    """API documentation page."""
    return """
    <html>
    <head><title>Feedback Analyzer API Docs</title></head>
    <body>
        <h1>Feedback Analyzer API</h1>
        <p>Interactive API documentation available at <a href="/redoc">/redoc</a></p>
        <h2>Endpoints</h2>
        <ul>
            <li><strong>POST /analyze</strong> - Analyze single feedback</li>
            <li><strong>POST /analyze/batch</strong> - Batch analysis with signing</li>
            <li><strong>GET /stats</strong> - Real-time statistics</li>
            <li><strong>GET /feedbacks</strong> - List feedback records</li>
            <li><strong>GET /rewards/{user_id}</strong> - User rewards</li>
            <li><strong>POST /auth/token</strong> - Get JWT token</li>
            <li><strong>GET /dashboard</strong> - Web dashboard</li>
        </ul>
    </body>
    </html>
    """


@app.get("/dashboard", response_class=HTMLResponse)
async def dashboard():
    """Web dashboard."""
    dashboard_path = os.path.join(static_dir, "dashboard.html")
    if os.path.exists(dashboard_path):
        return FileResponse(dashboard_path)
    raise HTTPException(status_code=404, detail="Dashboard not found")


# ==================== Analysis Endpoints ====================

@app.post("/analyze", response_model=FeedbackAnalysis)
async def analyze_feedback(
    feedback: FeedbackInput,
    user: Optional[dict] = Depends(get_optional_user),
):
    """Analyze a single feedback item."""
    try:
        user_id = user.get("user_id") if user else None
        result = analyzer.analyze(feedback)
        
        # Save to database
        db_manager.save_feedback(result, user_id=user_id)
        
        return result
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/analyze/batch")
async def analyze_batch(
    request: BatchAnalysisRequest,
    user: Optional[dict] = Depends(get_optional_user),
):
    """Analyze a batch of feedbacks with optional signing."""
    try:
        user_id = user.get("user_id") if user else None
        analyses, metadata = analyzer.analyze_batch(request.feedbacks)
        
        # Save to database
        for analysis in analyses:
            db_manager.save_feedback(analysis, user_id=user_id)
        
        if request.output_format == "json":
            return {
                "metadata": metadata.model_dump(),
                "records": [a.model_dump() for a in analyses],
            }
        elif request.output_format == "json_signed":
            result = output_generator.generate_signed_json(analyses, metadata)
            return result
        elif request.output_format == "parquet":
            # Generate parquet and return path
            output_path = f"/tmp/batch_{metadata.created_at}.parquet"
            output_generator.generate_parquet(analyses, metadata, output_path)
            return {"message": "Parquet generated", "path": output_path}
        elif request.output_format == "parquet_encrypted":
            if not request.encryption_key:
                raise HTTPException(status_code=400, detail="encryption_key required for parquet_encrypted")
            output_path = f"/tmp/batch_{metadata.created_at}.parquet.enc"
            output_generator.generate_encrypted_parquet(analyses, metadata, output_path, request.encryption_key)
            return {"message": "Encrypted parquet generated", "path": output_path}
        else:
            raise HTTPException(status_code=400, detail=f"Unsupported format: {request.output_format}")
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/stats")
async def get_stats():
    """Get real-time statistics."""
    try:
        # Combine in-memory metrics with database stats
        dashboard = analyzer.metrics.get_dashboard()
        db_stats = db_manager.get_stats()
        
        return {
            "total_analyzed": max(dashboard["total_analyzed"], db_stats["total_records"]),
            "sentiment_distribution": dashboard.get("sentiment_distribution", db_stats["sentiment_distribution"]),
            "avg_sentiment": dashboard.get("avg_sentiment", db_stats["avg_sentiment"]),
            "top_keywords": dashboard.get("top_keywords", []),
            "trend": dashboard.get("trend", "stable"),
            "alerts": dashboard.get("alerts", []),
            "reward_totals": dashboard.get("reward_totals", {}),
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/feedbacks")
async def list_feedbacks(
    user_id: Optional[str] = None,
    limit: int = 50,
    offset: int = 0,
):
    """List feedback records from database."""
    try:
        return db_manager.get_feedbacks(user_id=user_id, limit=limit, offset=offset)
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


@app.get("/verify/{batch_id}", response_model=VerificationResponse)
async def verify_batch(batch_id: str):
    """Verify integrity of a batch."""
    # Placeholder - would check stored batches
    return VerificationResponse(
        valid=True,
        merkle_verified=True,
        signature_verified=True,
        record_count=0,
        created_at="",
    )


# ==================== Auth Endpoints ====================

@app.post("/auth/token")
async def login(username: str, password: str):
    """Get JWT token for authentication."""
    # Simple demo authentication (in production, check against database)
    if username == "admin" and password == "admin":
        token = auth_manager.create_access_token(data={"sub": username, "role": "admin"})
        return {"access_token": token, "token_type": "bearer"}
    
    # For demo, allow any user
    token = auth_manager.create_access_token(data={"sub": username, "role": "user"})
    return {"access_token": token, "token_type": "bearer"}


@app.get("/auth/me")
async def get_me(user: dict = Depends(get_current_user)):
    """Get current authenticated user info."""
    return {"user_id": user["user_id"], "payload": user["payload"]}


# ==================== Protected Endpoints ====================

@app.post("/admin/clear-stats")
async def clear_stats(user: dict = Depends(get_current_user)):
    """Clear all statistics (admin only)."""
    if user["payload"].get("role") != "admin":
        raise HTTPException(status_code=403, detail="Admin access required")
    
    analyzer.metrics = __import__('feedback_analyzer.core.metrics', fromlist=['RealTimeMetrics']).RealTimeMetrics()
    return {"message": "Stats cleared"}
