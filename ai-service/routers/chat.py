"""Chat router: non-streaming chat completions via LiteLLM."""

from __future__ import annotations

from fastapi import APIRouter, HTTPException

from models import ChatRequest, ChatResponse
from chat_service import chat_completion

router = APIRouter(prefix="/api/v1/chat", tags=["chat"])


@router.post("/completions", response_model=ChatResponse)
async def chat_completions_endpoint(req: ChatRequest) -> ChatResponse:
    """Non-streaming chat completion endpoint.

    Returns {content, model, usage, provider_type}.

    Invalid provider_type is rejected by Pydantic with HTTP 422.
    """
    try:
        return await chat_completion(req)
    except HTTPException:
        raise
    except Exception as exc:
        raise HTTPException(status_code=422, detail="Chat completion failed")