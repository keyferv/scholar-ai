"""Chat service: wrap LiteLLM acompletion for streaming-free chat."""

from __future__ import annotations

import logging
import re
from typing import Any, Optional

from litellm import acompletion

from models import ChatRequest, ChatResponse, ChatMessage, ProviderType

logger = logging.getLogger(__name__)

# Patterns that look like API keys
_KEY_REDACT_RE = re.compile(r"(sk-[A-Za-z0-9_-]{20,})")


def _redact_keys(text: str) -> str:
    """Redact API keys from log messages."""
    return _KEY_REDACT_RE.sub("sk-***REDACTED***", text)


def _build_litellm_model_name(model: str, provider_type: Optional[ProviderType]) -> str:
    """Build a LiteLLM-compatible model identifier.

    For non-OpenAI providers, use the provider/model format.
    """
    if provider_type and provider_type != ProviderType.OPENAI:
        return f"{provider_type.value}/{model}"
    return model


async def chat_completion(req: ChatRequest) -> ChatResponse:
    """Execute a non-streaming chat completion via LiteLLM.

    Returns a ChatResponse with content, model name, and usage stats.
    Never includes API keys in error responses.
    """
    model_name = _build_litellm_model_name(req.model, req.provider_type)
    api_key = req.api_key
    base_url = req.base_url

    logger.info("Chat completion: model=%s provider=%s", model_name, req.provider_type)

    kwargs: dict[str, Any] = {
        "model": model_name,
        "messages": [m.model_dump() for m in req.messages],
        "max_tokens": req.max_tokens,
        "temperature": req.temperature,
        "stream": False,
    }

    if api_key:
        kwargs["api_key"] = api_key
    if base_url:
        kwargs["api_base"] = base_url

    try:
        response = await acompletion(**kwargs)

        # Extract usage and content from the LiteLLM response
        usage: dict[str, Any] = {}
        content = ""

        if hasattr(response, "choices") and response.choices:
            choice = response.choices[0]
            if hasattr(choice, "message") and choice.message:
                content = choice.message.content or ""
            elif hasattr(choice, "text"):
                content = choice.text or ""

        if hasattr(response, "usage"):
            u = response.usage
            usage = {
                "prompt_tokens": u.prompt_tokens if hasattr(u, "prompt_tokens") else 0,
                "completion_tokens": u.completion_tokens if hasattr(u, "completion_tokens") else 0,
                "total_tokens": u.total_tokens if hasattr(u, "total_tokens") else 0,
            }

        return ChatResponse(
            content=content,
            model=response.model or model_name,
            usage=usage,
            provider_type=req.provider_type.value if req.provider_type else None,
        )
    except Exception as exc:
        safe_msg = _redact_keys(str(exc))
        logger.error("Chat completion failed: model=%s error=%s", model_name, safe_msg)
        raise