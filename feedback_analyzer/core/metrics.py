"""Real-time metrics tracking system."""

from datetime import datetime, timezone, timedelta
from collections import deque, Counter
from dataclasses import dataclass, field
from typing import Optional


@dataclass
class AnalysisRecord:
    """Record of a single analysis for metrics."""
    sentiment_score: float
    category: str
    keywords: list[str]
    quality_score: float
    created_at: datetime


class SlidingWindow:
    """Sliding window for recent analyses."""
    
    def __init__(self, size: int = 1000):
        self.size = size
        self.records: deque[AnalysisRecord] = deque(maxlen=size)
    
    def add(self, record: AnalysisRecord):
        """Add a record to the window."""
        self.records.append(record)
    
    def get_sentiment_distribution(self) -> dict[str, float]:
        """Get sentiment distribution in current window."""
        if not self.records:
            return {"positive": 0.0, "negative": 0.0, "neutral": 0.0}
        
        pos = sum(1 for r in self.records if r.sentiment_score > 0.1)
        neg = sum(1 for r in self.records if r.sentiment_score < -0.1)
        neu = len(self.records) - pos - neg
        total = len(self.records)
        
        return {
            "positive": round(pos / total, 4),
            "negative": round(neg / total, 4),
            "neutral": round(neu / total, 4),
        }
    
    def get_average_sentiment(self) -> float:
        """Get average sentiment in current window."""
        if not self.records:
            return 0.0
        return sum(r.sentiment_score for r in self.records) / len(self.records)
    
    def get_top_keywords(self, n: int = 10) -> list[str]:
        """Get top N keywords in current window."""
        keyword_counts = Counter()
        for record in self.records:
            keyword_counts.update(record.keywords)
        return [kw for kw, _ in keyword_counts.most_common(n)]
    
    def get_category_distribution(self) -> dict[str, int]:
        """Get category distribution in current window."""
        return Counter(r.category for r in self.records)
    
    def get_trend(self, window_hours: int = 24) -> str:
        """Determine sentiment trend over time."""
        if len(self.records) < 10:
            return "insufficient_data"
        
        now = datetime.now(timezone.utc)
        cutoff = now - timedelta(hours=window_hours)
        
        recent = [r for r in self.records if r.created_at >= cutoff]
        older = [r for r in self.records if r.created_at < cutoff]
        
        if not recent or not older:
            return "insufficient_data"
        
        recent_avg = sum(r.sentiment_score for r in recent) / len(recent)
        older_avg = sum(r.sentiment_score for r in older) / len(older)
        
        diff = recent_avg - older_avg
        
        if diff > 0.1:
            return "improving"
        elif diff < -0.1:
            return "declining"
        else:
            return "stable"


class RealTimeMetrics:
    """Real-time metrics aggregator."""
    
    def __init__(self):
        self.window = SlidingWindow(size=1000)
        self.total_analyzed = 0
        self.alerts: list[str] = []
        self.reward_totals = {
            "total_points_distributed": 0,
            "active_users": 0,
        }
    
    def record(self, analysis):
        """Record an analysis for metrics."""
        record = AnalysisRecord(
            sentiment_score=analysis.sentiment_score,
            category=analysis.category,
            keywords=analysis.keywords,
            quality_score=analysis.quality_score,
            created_at=datetime.now(timezone.utc),
        )
        
        self.window.add(record)
        self.total_analyzed += 1
        
        # Check for alerts
        self._check_alerts()
    
    def add_reward_points(self, points: int):
        """Track reward points distributed."""
        self.reward_totals["total_points_distributed"] += points
    
    def add_active_user(self):
        """Track active users."""
        self.reward_totals["active_users"] += 1
    
    def get_dashboard(self) -> dict:
        """Get full metrics dashboard."""
        return {
            "total_analyzed": self.total_analyzed,
            "sentiment_distribution": self.window.get_sentiment_distribution(),
            "avg_sentiment": round(self.window.get_average_sentiment(), 4),
            "top_keywords": self.window.get_top_keywords(10),
            "trend": self.window.get_trend(),
            "alerts": self.alerts.copy(),
            "reward_totals": self.reward_totals.copy(),
        }
    
    def _check_alerts(self):
        """Check for alert conditions."""
        self.alerts.clear()
        
        # Check for negative sentiment spike
        dist = self.window.get_sentiment_distribution()
        if dist["negative"] > 0.3 and len(self.window.records) >= 50:
            self.alerts.append(
                f"Negative sentiment spike: {dist['negative']:.1%} of recent feedback is negative"
            )
        
        # Check for quality drop
        if len(self.window.records) >= 20:
            recent_quality = sum(
                r.quality_score for r in list(self.window.records)[-20:]
            ) / 20
            if recent_quality < 0.3:
                self.alerts.append(
                    f"Low quality feedback: Average quality score is {recent_quality:.2f}"
                )
