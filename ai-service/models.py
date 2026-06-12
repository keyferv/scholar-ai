"""Pydantic models for ScholarAI AI Providers."""

from __future__ import annotations

from enum import Enum
from pydantic import BaseModel, Field, field_validator
from typing import List, Optional


class ProviderType(str, Enum):
    OPENAI = "openai"
    ANTHROPIC = "anthropic"
    GEMINI = "gemini"
    AZURE = "azure"
    LOCAL = "local"


class ProviderMeta(BaseModel):
    """Metadata identifying an AI provider."""

    provider_type: ProviderType
    model: str
    base_url: Optional[str] = None


class ProviderCreateRequest(BaseModel):
    """Request to register/test a new AI provider."""

    provider_type: ProviderType = Field(..., description="The AI provider type")
    api_key: str = Field(..., min_length=1, description="API key for the provider")
    model: str = Field(..., min_length=1, description="Model name to use")
    base_url: Optional[str] = Field(None, description="Custom base URL for the provider")


class ProviderTestResult(BaseModel):
    """Result of testing a provider connection."""

    success: bool
    error: Optional[str] = None


class ChatMessage(BaseModel):
    """A single message in a chat conversation."""

    role: str = Field(..., pattern="^(system|user|assistant)$")
    content: str


class ChatRequest(BaseModel):
    """Request for a chat completion."""

    model: str = Field(..., min_length=1, description="Model name to use")
    messages: List[ChatMessage] = Field(..., min_length=1, max_length=100)
    provider_type: Optional[ProviderType] = None
    base_url: Optional[str] = None
    api_key: Optional[str] = None
    temperature: Optional[float] = Field(default=0.7, ge=0.0, le=2.0)
    max_tokens: Optional[int] = Field(default=1024, gt=0)


class ChatResponse(BaseModel):
    """Response from a chat completion."""

    content: str
    model: str
    usage: dict = Field(default_factory=dict)
    provider_type: Optional[str] = None