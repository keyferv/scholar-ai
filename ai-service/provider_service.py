"""Provider service: test connectivity to AI providers via LiteLLM."""

from __future__ import annotations

import logging
import re
from typing import Any, Optional

from litellm import acompletion

from models import ProviderCreateRequest, ProviderTestResult, ProviderType

logger = logging.getLogger(__name__)

# Patterns that look like API keys (start with common prefixes)
_KEY_REDACT_RE = re.compile(r"(sk-[A-Za-z0-9_-]{20,})")


def _redact_keys(text: str) -> str:
    """Redact API keys from log messages."""
    return _KEY_REDACT_RE.sub("sk-***REDACTED***", text)


async def test_provider(req: ProviderCreateRequest) -> ProviderTestResult:
    """Test connectivity to an AI provider using LiteLLM.

    Returns a ProviderTestResult indicating success or failure.
    Never includes API keys in error messages.
    """
    provider_type = req.provider_type
    model = req.model
    api_key = req.api_key
    base_url = req.base_url

    logger.info(
        "Testing provider: type=%s model=%s",
        provider_type.value,
        model,
    )

    # Build the keyword arguments for litellm.acompletion
    kwargs: dict[str, Any] = {
        "model": model,
        "api_key": api_key,
        "messages": [{"role": "user", "content": "Say 'ok'"}],
        "max_tokens": 5,
    }

    # Only set provider if it's not the default (openai)
    if provider_type != ProviderType.OPENAI:
        kwargs["custom_provider"] = provider_type.value

    if base_url:
        kwargs["api_base"] = base_url

    try:
        response = await acompletion(**kwargs)
        logger.info("Provider test succeeded: type=%s model=%s", provider_type.value, model)
        return ProviderTestResult(success=True)
    except Exception as exc:
        # Log the error internally but never expose key material
        safe_msg = _redact_keys(str(exc))
        logger.error("Provider test failed: type=%s model=%s error=%s", provider_type.value, model, safe_msg)
        return ProviderTestResult(success=False, error="Provider test failed — check configuration")