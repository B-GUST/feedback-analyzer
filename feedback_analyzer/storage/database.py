"""Database operations for persistence."""

from datetime import datetime, timezone
from typing import Optional
from feedback_analyzer.storage.models import (
    init_db, get_session, FeedbackRecord, BatchRecord, UserReward
)
from feedback_analyzer.api.models import FeedbackAnalysis, BatchMetadata


class DatabaseManager:
    """Handle all database operations."""
    
    def __init__(self, db_url: str = "sqlite:///feedback_analyzer.db"):
        init_db(db_url)
    
    def save_feedback(self, analysis: FeedbackAnalysis, user_id: Optional[str] = None):
        """Save a feedback analysis to database."""
        session = get_session()
        try:
            record = FeedbackRecord(
                id=analysis.id,
                text=analysis.text,
                language=analysis.language,
                sentiment_score=analysis.sentiment_score,
                sentiment_label=analysis.sentiment_label,
                category=analysis.category,
                keywords=analysis.keywords,
                quality_score=analysis.quality_score,
                user_id=user_id,
                created_at=datetime.fromisoformat(analysis.created_at),
            )
            session.merge(record)
            session.commit()
        finally:
            session.close()
    
    def save_batch(
        self,
        batch_id: str,
        metadata: BatchMetadata,
        merkle_root: str,
        signature: str,
        public_key: str,
    ):
        """Save batch analysis record."""
        session = get_session()
        try:
            record = BatchRecord(
                id=batch_id,
                total_records=metadata.total_records,
                avg_sentiment=metadata.avg_sentiment,
                sentiment_distribution=metadata.sentiment_distribution,
                merkle_root=merkle_root,
                signature=signature,
                public_key=public_key,
            )
            session.merge(record)
            session.commit()
        finally:
            session.close()
    
    def get_feedbacks(
        self,
        user_id: Optional[str] = None,
        limit: int = 100,
        offset: int = 0,
    ) -> list[dict]:
        """Get feedback records from database."""
        session = get_session()
        try:
            query = session.query(FeedbackRecord)
            if user_id:
                query = query.filter(FeedbackRecord.user_id == user_id)
            query = query.order_by(FeedbackRecord.created_at.desc())
            query = query.offset(offset).limit(limit)
            
            return [
                {
                    "id": r.id,
                    "text": r.text,
                    "language": r.language,
                    "sentiment_score": r.sentiment_score,
                    "sentiment_label": r.sentiment_label,
                    "category": r.category,
                    "keywords": r.keywords,
                    "quality_score": r.quality_score,
                    "user_id": r.user_id,
                    "created_at": r.created_at.isoformat() if r.created_at else None,
                }
                for r in query.all()
            ]
        finally:
            session.close()
    
    def get_stats(self) -> dict:
        """Get overall statistics from database."""
        session = get_session()
        try:
            total = session.query(FeedbackRecord).count()
            
            if total == 0:
                return {
                    "total_records": 0,
                    "avg_sentiment": 0.0,
                    "sentiment_distribution": {"positive": 0, "negative": 0, "neutral": 0},
                    "top_categories": [],
                }
            
            # Sentiment distribution
            positive = session.query(FeedbackRecord).filter(
                FeedbackRecord.sentiment_label == "positive"
            ).count()
            negative = session.query(FeedbackRecord).filter(
                FeedbackRecord.sentiment_label == "negative"
            ).count()
            neutral = total - positive - negative
            
            # Average sentiment
            from sqlalchemy import func
            avg_sentiment = session.query(func.avg(FeedbackRecord.sentiment_score)).scalar() or 0.0
            
            # Top categories
            from sqlalchemy import desc
            top_categories = (
                session.query(FeedbackRecord.category, func.count(FeedbackRecord.id))
                .group_by(FeedbackRecord.category)
                .order_by(desc(func.count(FeedbackRecord.id)))
                .limit(5)
                .all()
            )
            
            return {
                "total_records": total,
                "avg_sentiment": round(float(avg_sentiment), 4),
                "sentiment_distribution": {
                    "positive": round(positive / total, 4),
                    "negative": round(negative / total, 4),
                    "neutral": round(neutral / total, 4),
                },
                "top_categories": [{"category": c, "count": n} for c, n in top_categories],
            }
        finally:
            session.close()


# Global instance
db_manager = DatabaseManager()
