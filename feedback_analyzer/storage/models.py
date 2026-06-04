"""Database models for persistence."""

from datetime import datetime, timezone
from sqlalchemy import create_engine, Column, String, Float, Integer, DateTime, Text, JSON
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker

Base = declarative_base()


class FeedbackRecord(Base):
    """Store analyzed feedback records."""
    __tablename__ = "feedback_records"
    
    id = Column(String, primary_key=True)
    text = Column(Text, nullable=False)
    language = Column(String(10))
    sentiment_score = Column(Float)
    sentiment_label = Column(String(20))
    category = Column(String(50))
    keywords = Column(JSON)
    quality_score = Column(Float)
    user_id = Column(String(100), index=True)
    created_at = Column(DateTime, default=lambda: datetime.now(timezone.utc))


class BatchRecord(Base):
    """Store batch analysis records."""
    __tablename__ = "batch_records"
    
    id = Column(String, primary_key=True)
    total_records = Column(Integer)
    avg_sentiment = Column(Float)
    sentiment_distribution = Column(JSON)
    merkle_root = Column(String)
    signature = Column(Text)
    public_key = Column(String)
    created_at = Column(DateTime, default=lambda: datetime.now(timezone.utc))


class UserReward(Base):
    """Store user reward data."""
    __tablename__ = "user_rewards"
    
    user_id = Column(String, primary_key=True)
    total_points = Column(Integer, default=0)
    level = Column(String(20), default="novice")
    badges = Column(JSON, default=list)
    streak_days = Column(Integer, default=0)
    last_contribution = Column(DateTime)
    created_at = Column(DateTime, default=lambda: datetime.now(timezone.utc))


# Database engine and session
_engine = None
_Session = None


def init_db(db_url: str = "sqlite:///feedback_analyzer.db"):
    """Initialize database connection."""
    global _engine, _Session
    _engine = create_engine(db_url, echo=False)
    Base.metadata.create_all(_engine)
    _Session = sessionmaker(bind=_engine)


def get_session():
    """Get a database session."""
    if _Session is None:
        init_db()
    return _Session()
