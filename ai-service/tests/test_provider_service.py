"""Tests for provider_service module using mocked litellm."""

import asyncio
import sys
import os

# Ensure the ai-service directory is importable
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from unittest.mock import AsyncMock, patch, MagicMock
import pytest

from provider_service import test_provider
from models import (
    ProviderCreateRequest,
    ProviderTestResult,
    ProviderType,
)


class TestTestProvider:
    @patch("provider_service.acompletion")
    async def test_openai_success(self, mock_acompletion: AsyncMock):
        """OpenAI provider with valid credentials should succeed."""
        mock_response = MagicMock()
        mock_acompletion.return_value = mock_response

        req = ProviderCreateRequest(
            provider_type=ProviderType.OPENAI,
            api_key="sk-test1234567890abcdefghij",
            model="gpt-4o",
        )
        result = await test_provider(req)

        assert isinstance(result, ProviderTestResult)
        assert result.success is True
        assert result.error is None

        mock_acompletion.assert_called_once()
        call_kwargs = mock_acompletion.call_args
        assert call_kwargs["model"] == "gpt-4o"
        assert call_kwargs["api_key"] == "sk-test1234567890abcdefghij"

    @patch("provider_service.acompletion")
    async def test_ollama_success(self, mock_acompletion: AsyncMock):
        """Ollama provider with base_url should route correctly."""
        mock_response = MagicMock()
        mock_acompletion.return_value = mock_response

        req = ProviderCreateRequest(
            provider_type=ProviderType.OLLAMA,
            api_key="n/a",
            model="llama3",
            base_url="http://localhost:11434",
        )
        result = await test_provider(req)

        assert result.success is True
        mock_acompletion.assert_called_once()
        call_kwargs = mock_acompletion.call_args
        assert call_kwargs["custom_provider"] == "ollama"
        assert call_kwargs["api_base"] == "http://localhost:11434"

    @patch("provider_service.acompletion")
    async def test_anthropic_failure(self, mock_acompletion: AsyncMock):
        """Anthropic provider with bad key should return failure without exposing key."""
        mock_acompletion.side_effect = Exception(
            "AuthenticationError: API key is invalid sk-ant-abc123secretkey"
        )

        req = ProviderCreateRequest(
            provider_type=ProviderType.ANTHROPIC,
            api_key="sk-ant-abc123secretkey",
            model="claude-3-5-sonnet-20241022",
        )
        result = await test_provider(req)

        assert result.success is False
        assert result.error is not None
        # Key must never appear in the error message
        assert "sk-ant-abc123secretkey" not in result.error
        # Redacted form should not appear either
        assert "sk-ant-" not in result.error

    @patch("provider_service.acompletion")
    async def test_openrouter_routing(self, mock_acompletion: AsyncMock):
        """OpenRouter provider should set custom_provider correctly."""
        mock_response = MagicMock()
        mock_acompletion.return_value = mock_response

        req = ProviderCreateRequest(
            provider_type=ProviderType.OPENROUTER,
            api_key="sk-or-test",
            model="meta-llama/llama-3.1-8b-instruct",
        )
        result = await test_provider(req)

        assert result.success is True
        call_kwargs = mock_acompletion.call_args
        assert call_kwargs["custom_provider"] == "openrouter"

    @patch("provider_service.acompletion")
    async def test_generic_exception_key_not_leaked(self, mock_acompletion: AsyncMock):
        """Even on unexpected errors, no API key material should leak."""
        mock_acompletion.side_effect = Exception(
            "ConnectionError: timeout connecting to api.openai.com with key sk-proj-XyZ123"
        )

        req = ProviderCreateRequest(
            provider_type=ProviderType.OPENAI,
            api_key="sk-proj-XyZ123",
            model="gpt-4o",
        )
        result = await test_provider(req)

        assert result.success is False
        assert "sk-proj-XyZ123" not in result.error
        assert "sk-proj-XyZ" not in result.error

    async def test_no_litellm_mock(self):
        """Verify the function structure without mocking — import check."""
        from provider_service import _redact_keys

        # Key redaction should mask sk- prefixed tokens
        text = "Error: invalid api key sk-1234567890abcdefghij0123456789"
        redacted = _redact_keys(text)
        assert "sk-1234567890abcdefghij0123456789" not in redacted
        assert "sk-***REDACTED***" in redacted