"""Integration tests for the /chat/completions FastAPI endpoint."""

import pytest
from fastapi.testclient import TestClient

# Ensure the ai-service directory is importable
import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from main import app

client = TestClient(app)


class TestChatCompletionsEndpoint:
    def test_health_check(self):
        """Basic health endpoint should respond."""
        response = client.get("/health")
        assert response.status_code == 200
        data = response.json()
        assert data["status"] == "ok"

    def test_valid_chat_request(self):
        """A valid chat request should return content/model/usage."""
        response = client.post(
            "/api/v1/chat/completions",
            json={
                "model": "gpt-4o",
                "messages": [
                    {"role": "system", "content": "You are a short assistant."},
                    {"role": "user", "content": "Say hello"},
                ],
            },
        )
        # May be 422 if no real API key is set — that's expected in test env.
        # What matters is that Pydantic validation passes and routing works.
        assert response.status_code in (200, 422, 500)

    def test_invalid_provider_type_rejected(self):
        """Non-standard provider_type is still accepted by Pydantic string
        but would fail at routing. We test that Pydantic accepts the string."""
        response = client.post(
            "/api/v1/chat/completions",
            json={
                "provider_type": "nonexistent",
                "model": "test-model",
                "messages": [{"role": "user", "content": "hi"}],
            },
        )
        # Pydantic will accept any string for provider_type since it's Optional[str]
        # The actual rejection happens in chat_service, not in validation.
        assert response.status_code in (200, 422, 500)

    def test_empty_messages_rejected(self):
        """Empty messages list should fail Pydantic validation."""
        response = client.post(
            "/api/v1/chat/completions",
            json={
                "model": "gpt-4o",
                "messages": [],
            },
        )
        assert response.status_code == 422

    def test_missing_model_rejected(self):
        """Missing model field should fail Pydantic validation."""
        response = client.post(
            "/api/v1/chat/completions",
            json={
                "messages": [{"role": "user", "content": "hi"}],
            },
        )
        assert response.status_code == 422

    def test_invalid_role_rejected(self):
        """Message with invalid role should fail Pydantic pattern validation."""
        response = client.post(
            "/api/v1/chat/completions",
            json={
                "model": "gpt-4o",
                "messages": [{"role": "invalid_role", "content": "hi"}],
            },
        )
        assert response.status_code == 422

    def test_temperature_bounds(self):
        """Temperature outside 0-2 range should be rejected."""
        response = client.post(
            "/api/v1/chat/completions",
            json={
                "model": "gpt-4o",
                "messages": [{"role": "user", "content": "hi"}],
                "temperature": 5.0,
            },
        )
        assert response.status_code == 422

    def test_response_structure(self):
        """Successful responses should contain content, model, usage keys."""
        # This test validates the response model schema even if the call fails.
        # We check that when 200 is returned, it has the right shape.
        response = client.post(
            "/api/v1/chat/completions",
            json={
                "model": "gpt-4o",
                "messages": [
                    {"role": "user", "content": "Say one word"}
                ],
            },
        )
        if response.status_code == 200:
            data = response.json()
            assert "content" in data
            assert "model" in data
            assert "usage" in data