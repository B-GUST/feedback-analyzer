"""Rewards and gamification system."""

from datetime import datetime, timezone, timedelta
from dataclasses import dataclass, field
from typing import Optional
from collections import defaultdict


@dataclass
class Reward:
    """Reward for a single contribution."""
    points: int
    level: str
    badges: list[str]
    total_points: int
    streak_days: int


@dataclass
class UserProfile:
    """User reward profile."""
    user_id: str
    total_points: int = 0
    level: str = "novice"
    badges: list[str] = field(default_factory=list)
    streak_days: int = 0
    last_contribution: Optional[datetime] = None
    contribution_dates: list[str] = field(default_factory=list)


# Level thresholds
LEVELS = {
    "novice": 0,
    "collaborator": 101,
    "analyst": 501,
    "expert": 2001,
    "master": 5001,
}

# Level order for progression
LEVEL_ORDER = ["novice", "collaborator", "analyst", "expert", "master"]


class RewardsEngine:
    """Manages user rewards, levels, and badges."""
    
    def __init__(self):
        self.profiles: dict[str, UserProfile] = {}
        self.badge_definitions = {
            "first_feedback": "First feedback submitted",
            "streak_7": "7-day contribution streak",
            "streak_30": "30-day contribution streak",
            "quality_star": "Feedback with quality score > 0.9",
            "diversity": "Feedback in 3+ different categories",
            "century": "100 feedbacks submitted",
            "sentiment_expert": "Consistent sentiment predictions",
            "early_adopter": "Joined in first month",
        }
    
    def calculate_reward(
        self,
        user_id: str,
        text: str,
        sentiment_score: float,
        quality_score: float,
        category: str,
    ) -> Reward:
        """Calculate reward for a feedback contribution."""
        profile = self._get_or_create_profile(user_id)
        
        # Base points
        base_points = 10
        
        # Quality bonus (0-5 points)
        quality_bonus = int(quality_score * 5)
        
        # Sentiment diversity bonus (0-3 points)
        # More extreme sentiments are more valuable for analysis
        sentiment_bonus = int(abs(sentiment_score) * 3)
        
        # Length/completeness bonus (0-2 points)
        word_count = len(text.split())
        completeness_bonus = min(2, word_count // 10)
        
        total_points = base_points + quality_bonus + sentiment_bonus + completeness_bonus
        
        # Update streak
        self._update_streak(profile)
        
        # Check for new badges
        new_badges = self._check_badges(profile, quality_score, category)
        
        # Update profile
        profile.total_points += total_points
        profile.last_contribution = datetime.now(timezone.utc)
        profile.contribution_dates.append(datetime.now(timezone.utc).strftime("%Y-%m-%d"))
        
        # Update level
        new_level = self._calculate_level(profile.total_points)
        if new_level != profile.level:
            profile.level = new_level
        
        return Reward(
            points=total_points,
            level=profile.level,
            badges=new_badges,
            total_points=profile.total_points,
            streak_days=profile.streak_days,
        )
    
    def get_user_rewards(self, user_id: str) -> dict:
        """Get full reward summary for a user."""
        profile = self._get_or_create_profile(user_id)
        
        # Calculate next level points
        current_level_idx = LEVEL_ORDER.index(profile.level)
        if current_level_idx < len(LEVEL_ORDER) - 1:
            next_level = LEVEL_ORDER[current_level_idx + 1]
            next_level_points = LEVELS[next_level]
        else:
            next_level_points = profile.total_points  # Already max level
        
        return {
            "user_id": user_id,
            "level": profile.level,
            "total_points": profile.total_points,
            "badges": profile.badges,
            "streak_days": profile.streak_days,
            "next_level_points": next_level_points,
            "recent_rewards": [],  # Would be populated from DB
        }
    
    def _get_or_create_profile(self, user_id: str) -> UserProfile:
        """Get or create user profile."""
        if user_id not in self.profiles:
            self.profiles[user_id] = UserProfile(user_id=user_id)
        return self.profiles[user_id]
    
    def _update_streak(self, profile: UserProfile):
        """Update contribution streak."""
        now = datetime.now(timezone.utc)
        today = now.strftime("%Y-%m-%d")
        
        if profile.last_contribution is None:
            profile.streak_days = 1
            return
        
        last_date = profile.last_contribution.date()
        days_since = (now.date() - last_date).days
        
        if days_since == 1:
            # Consecutive day
            profile.streak_days += 1
        elif days_since == 0:
            # Same day, no change
            pass
        else:
            # Streak broken
            profile.streak_days = 1
    
    def _calculate_level(self, total_points: int) -> str:
        """Calculate level based on total points."""
        level = "novice"
        for level_name, threshold in LEVELS.items():
            if total_points >= threshold:
                level = level_name
        return level
    
    def _check_badges(
        self,
        profile: UserProfile,
        quality_score: float,
        category: str,
    ) -> list[str]:
        """Check and award new badges."""
        new_badges = []
        
        # First feedback badge
        if "first_feedback" not in profile.badges:
            new_badges.append("first_feedback")
            profile.badges.append("first_feedback")
        
        # Quality star badge
        if "quality_star" not in profile.badges and quality_score > 0.9:
            new_badges.append("quality_star")
            profile.badges.append("quality_star")
        
        # Streak badges
        if "streak_7" not in profile.badges and profile.streak_days >= 7:
            new_badges.append("streak_7")
            profile.badges.append("streak_7")
        
        if "streak_30" not in profile.badges and profile.streak_days >= 30:
            new_badges.append("streak_30")
            profile.badges.append("streak_30")
        
        # Century badge (100 contributions)
        if "century" not in profile.badges and len(profile.contribution_dates) >= 100:
            new_badges.append("century")
            profile.badges.append("century")
        
        # Diversity badge (3+ categories)
        if "diversity" not in profile.badges:
            categories = set()
            # In production, this would query the database
            # For now, track locally
            if len(categories) >= 3:
                new_badges.append("diversity")
                profile.badges.append("diversity")
        
        return new_badges
