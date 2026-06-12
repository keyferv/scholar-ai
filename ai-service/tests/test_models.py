"""Tests for Pydantic models in ai-service.models."""

import pytest
from pydantic import ValidationError

from models import (
    ChatMessage,
    ChatRequest,
    ChatResponse,
    ProviderCreateRequest,
    ProviderTestResult,
    ProviderType,
)


class TestProviderType:
    def test_valid_provider_types(self):
        """All defined enum values should be valid."""
        assert ProviderType.OPENAI == "openai"
        assert ProviderType.ANTHROPIC == "anthropic"
        assert ProviderType.GEMINI == "gemini"
        assert ProviderType.AZURE == "azure"
        assert ProviderType.LOCAL == "local"


class TestProviderCreateRequest:
    def test_valid_request(self):
        req = ProviderCreateRequest(
            provider_type=ProviderType.OPENAI,
            api_key="sk-test1234567890abcdefghij",
            model="gpt-4o",
        )
        assert req.provider_type == ProviderType.OPENAI
        assert req.api_key == "sk-test1234567890abcdefghij"
        assert req.model == "gpt-4o"

    def test_valid_with_base_url(self):
        req = ProviderCreateRequest(
            provider_type=ProviderType.OLLAMA,
            api_key="ollama-local-key",
            model="llama3",
            base_url="http://localhost:11434",
        )
        assert req.base_url == "http://localhost:11434"

    def test_invalid_empty_api_key(self):
        with pytest.raises(ValidationError):
            ProviderCreateRequest(
                provider_type=ProviderType.OPENAI,
                api_key="",
                model="gpt-4o",
            )

    def test_invalid_empty_model(self):
        with pytest.raises(ValidationError):
            ProviderCreateRequest(
                provider_type=ProviderType.OPENAI,
                api_key="sk-test",
                model="",
            )

    def test_invalid_provider_type(self):
        with pytest.raises(ValidationError):
            ProviderCreateRequest(
                provider_type="invalid_type",
                api_key="sk-test",
                model="gpt-4o",
            )


class TestProviderTestResult:
    def test_success_result(self):
        result = ProviderTestResult(success=True)
        assert result.success is True
        assert result.error is None

    def test_failure_result(self):
        result = ProviderTestResult(
            success=False, error="Connection refused"
        )
        assert result.success is False
        assert result.error == "Connection refused"


class TestChatMessage:
    def test_valid_user_message(self):
        msg = ChatMessage(role="user", content="Hello")
        assert msg.role == "user"
        assert msg.content == "Hello"

    def test_valid_system_message(self):
        msg = ChatMessage(role="system", content="You are a helpful assistant.")
        assert msg.role == "system"

    def test_invalid_role(self):
        with pytest.raises(ValidationError):
            ChatMessage(role="invalid_role", content="test")


class TestChatRequest:
    def test_valid_request(self):
        req = ChatRequest(
            model="gpt-4o",
            messages=[
                ChatMessage(role="system", content="You are helpful."),
                ChatMessage(role="user", content="Hello!"),
            ],
        )
        assert req.model == "gpt-4o"
        assert len(req.messages) == 2
        assert req.temperature == 0.7

    def test_valid_with_provider_type(self):
        req = ChatRequest(
            model="llama3",
            messages=[ChatMessage(role="user", content="Hi")],
            provider_type=ProviderType.OLLAMA,
            base_url="http://localhost:11434",
        )
        assert req.provider_type == ProviderType.OLLAMA
        assert req.base_url == "http://localhost:11434"

    def test_invalid_empty_messages(self):
        with pytest.raises(ValidationError):
            ChatRequest(model="gpt-4o", messages=[])

    def test_invalid_temperature_out_of_range(self):
        with pytest.raises(ValidationError):
            ChatRequest(
                model="gpt-4o",
                messages=[ChatMessage(role="user", content="Hi")],
                temperature=5.0,
            )


class TestChatResponse:
    def test_valid_response(self):
        resp = ChatResponse(
            content="Hello, world!",
            model="gpt-4o",
            usage={"prompt_tokens": 10, "completion_tokens": 20, "total_tokens": 30},
        )
        assert resp.content == "Hello, world!"
        assert resp.model == "gpt-4o"
        assert resp.usage["total_tokens"] == 30

    def test_optional_provider_type(self):
        resp = ChatResponse(content="test", model="gpt-4o")
        assert resp.provider_type is None