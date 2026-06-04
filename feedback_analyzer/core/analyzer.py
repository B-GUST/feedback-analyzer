"""Core analysis orchestrator - bridges Python and Rust."""

import uuid
from datetime import datetime, timezone
from typing import Optional

from feedback_analyzer.api.models import (
    FeedbackInput,
    FeedbackAnalysis,
    BatchMetadata,
)
from feedback_analyzer.core.rewards import RewardsEngine
from feedback_analyzer.core.metrics import RealTimeMetrics

# Try to import Rust extension, fallback to Python implementations
try:
    import feedback_analyzer.rust_analyzer as rust
    HAS_RUST = True
except ImportError:
    HAS_RUST = False


class Analyzer:
    """Main analysis engine."""
    
    def __init__(self):
        self.rewards = RewardsEngine()
        self.metrics = RealTimeMetrics()
    
    def analyze(self, feedback: FeedbackInput) -> FeedbackAnalysis:
        """Analyze a single feedback item."""
        text = feedback.text
        
        # Language detection
        if HAS_RUST:
            language = rust.detect_language(text)
        else:
            language = self._detect_language_fallback(text)
        
        # Sentiment analysis
        if HAS_RUST:
            sentiment_score = rust.calculate_sentiment_score(text)
        else:
            sentiment_score = self._sentiment_fallback(text)
        
        # Sentiment label
        sentiment_label = self._get_sentiment_label(sentiment_score)
        
        # Keyword extraction
        if HAS_RUST:
            keywords = rust.extract_keywords(text, 5)
        else:
            keywords = self._keywords_fallback(text)
        
        # Category determination
        category = feedback.category or self._determine_category(text, keywords)
        
        # Quality scoring
        if HAS_RUST:
            quality_score = rust.calculate_quality_score(text)
        else:
            quality_score = self._quality_fallback(text)
        
        # Create analysis record
        analysis = FeedbackAnalysis(
            id=str(uuid.uuid4()),
            text=text,
            language=language,
            sentiment_score=round(sentiment_score, 4),
            sentiment_label=sentiment_label,
            category=category,
            keywords=keywords,
            quality_score=round(quality_score, 4),
            created_at=datetime.now(timezone.utc).isoformat(),
        )
        
        # Update metrics
        self.metrics.record(analysis)
        
        return analysis
    
    def analyze_batch(
        self,
        feedbacks: list[FeedbackInput],
        user_id: Optional[str] = None,
    ) -> tuple[list[FeedbackAnalysis], BatchMetadata]:
        """Analyze a batch of feedbacks."""
        analyses = [self.analyze(fb) for fb in feedbacks]
        
        # Calculate batch metadata
        sentiments = [a.sentiment_score for a in analyses]
        avg_sentiment = sum(sentiments) / len(sentiments) if sentiments else 0.0
        
        # Sentiment distribution
        pos = sum(1 for s in sentiments if s > 0.1)
        neg = sum(1 for s in sentiments if s < -0.1)
        neu = len(sentiments) - pos - neg
        total = len(sentiments) or 1
        
        metadata = BatchMetadata(
            total_records=len(analyses),
            avg_sentiment=round(avg_sentiment, 4),
            sentiment_distribution={
                "positive": round(pos / total, 4),
                "negative": round(neg / total, 4),
                "neutral": round(neu / total, 4),
            },
            created_at=datetime.now(timezone.utc).isoformat(),
            analyzer_version="0.1.0",
        )
        
        return analyses, metadata
    
    def _get_sentiment_label(self, score: float) -> str:
        """Convert sentiment score to label."""
        if score > 0.1:
            return "positive"
        elif score < -0.1:
            return "negative"
        else:
            return "neutral"
    
    def _determine_category(self, text: str, keywords: list[str]) -> str:
        """Determine feedback category based on content."""
        text_lower = text.lower()
        
        category_keywords = {
            "product_quality": ["quality", "quality", "calidad", "material", "defect"],
            "customer_service": ["service", "support", "help", "servicio", "ayuda"],
            "usability": ["easy", "difficult", "confusing", "fácil", "difícil"],
            "pricing": ["price", "cost", "expensive", "cheap", "precio", "costo"],
            "delivery": ["shipping", "delivery", "late", "envío", "entrega"],
            "general": [],
        }
        
        for category, cats in category_keywords.items():
            if any(kw in text_lower for kw in cats):
                return category
            if any(keyword in cats for keyword in keywords):
                return category
        
        return "general"
    
    # Fallback methods when Rust is not available
    
    def _detect_language_fallback(self, text: str) -> str:
        """Simple language detection fallback."""
        if any(c in text for c in "ñ¿¡"):
            return "es"
        return "en"
    
    def _sentiment_fallback(self, text: str) -> float:
        """Simple sentiment analysis fallback."""
        positive = ["excellent", "great", "good", "amazing", "love", "best"]
        negative = ["bad", "terrible", "horrible", "worst", "hate", "poor"]
        
        text_lower = text.lower()
        pos = sum(1 for w in positive if w in text_lower)
        neg = sum(1 for w in negative if w in text_lower)
        
        total = pos + neg
        if total == 0:
            return 0.0
        return (pos - neg) / total
    
    def _keywords_fallback(self, text: str) -> list[str]:
        """Simple keyword extraction fallback."""
        stopwords = {"the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for"}
        
        words = text.lower().split()
        words = [w.strip(".,!?;:") for w in words if len(w) > 3]
        words = [w for w in words if w not in stopwords]
        
        from collections import Counter
        return [w for w, _ in Counter(words).most_common(5)]
    
    def _quality_fallback(self, text: str) -> float:
        """Simple quality scoring fallback."""
        score = 0.0
        
        length = len(text)
        if 20 <= length <= 200:
            score += 0.5
        elif length > 200:
            score += 0.3
        
        word_count = len(text.split())
        if 3 <= word_count <= 50:
            score += 0.3
        
        if text.endswith((".", "!", "?")):
            score += 0.2
        
        return min(score, 1.0)


# Global analyzer instance
analyzer = Analyzer()
