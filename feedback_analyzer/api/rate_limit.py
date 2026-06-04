"""Rate limiting middleware."""

import time
from collections import defaultdict
from typing import Optional
from fastapi import Request, HTTPException, status
from fastapi.responses import JSONResponse


class RateLimiter:
    """Simple in-memory rate limiter."""
    
    def __init__(
        self,
        requests_per_minute: int = 60,
        requests_per_hour: int = 1000,
    ):
        self.requests_per_minute = requests_per_minute
        self.requests_per_hour = requests_per_hour
        self.minute_windows: dict[str, list[float]] = defaultdict(list)
        self.hour_windows: dict[str, list[float]] = defaultdict(list)
    
    def _get_client_id(self, request: Request) -> str:
        """Get client identifier from request."""
        # Use API key if present, otherwise use IP
        api_key = request.headers.get("X-API-Key")
        if api_key:
            return f"api:{api_key}"
        
        # Use IP address
        forwarded = request.headers.get("X-Forwarded-For")
        if forwarded:
            ip = forwarded.split(",")[0].strip()
        else:
            ip = request.client.host if request.client else "unknown"
        
        return f"ip:{ip}"
    
    def _clean_window(self, window: list[float], duration: float) -> list[float]:
        """Remove timestamps outside the window."""
        cutoff = time.time() - duration
        return [t for t in window if t > cutoff]
    
    def check_rate_limit(self, client_id: str) -> Optional[str]:
        """Check if client has exceeded rate limit. Returns error message if exceeded."""
        now = time.time()
        
        # Check per-minute limit
        self.minute_windows[client_id] = self._clean_window(
            self.minute_windows[client_id], 60
        )
        if len(self.minute_windows[client_id]) >= self.requests_per_minute:
            return f"Rate limit exceeded: {self.requests_per_minute} requests per minute"
        
        # Check per-hour limit
        self.hour_windows[client_id] = self._clean_window(
            self.hour_windows[client_id], 3600
        )
        if len(self.hour_windows[client_id]) >= self.requests_per_hour:
            return f"Rate limit exceeded: {self.requests_per_hour} requests per hour"
        
        # Record this request
        self.minute_windows[client_id].append(now)
        self.hour_windows[client_id].append(now)
        
        return None
    
    def get_remaining(self, client_id: str) -> dict:
        """Get remaining requests for client."""
        self.minute_windows[client_id] = self._clean_window(
            self.minute_windows[client_id], 60
        )
        self.hour_windows[client_id] = self._clean_window(
            self.hour_windows[client_id], 3600
        )
        
        return {
            "minute_remaining": max(0, self.requests_per_minute - len(self.minute_windows[client_id])),
            "hour_remaining": max(0, self.requests_per_hour - len(self.hour_windows[client_id])),
            "minute_limit": self.requests_per_minute,
            "hour_limit": self.requests_per_hour,
        }


# Global rate limiter instance
rate_limiter = RateLimiter()


async def rate_limit_middleware(request: Request, call_next):
    """FastAPI middleware for rate limiting."""
    client_id = rate_limiter._get_client_id(request)
    
    error = rate_limiter.check_rate_limit(client_id)
    if error:
        return JSONResponse(
            status_code=status.HTTP_429_TOO_MANY_REQUESTS,
            content={"detail": error},
            headers={"Retry-After": "60"},
        )
    
    response = await call_next(request)
    
    # Add rate limit headers
    remaining = rate_limiter.get_remaining(client_id)
    response.headers["X-RateLimit-Minute-Remaining"] = str(remaining["minute_remaining"])
    response.headers["X-RateLimit-Hour-Remaining"] = str(remaining["hour_remaining"])
    
    return response
