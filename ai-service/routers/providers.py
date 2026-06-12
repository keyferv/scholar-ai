"""Provider router: test AI provider connectivity."""

from __future__ import annotations

from fastapi import APIRouter, HTTPException

from models import ProviderCreateRequest, ProviderTestResult
from provider_service import test_provider

router = APIRouter(prefix="/api/v1/providers", tags=["providers"])


@router.post("/test", response_model=ProviderTestResult)
async def test_provider_endpoint(req: ProviderCreateRequest) -> ProviderTestResult:
    """Test an AI provider's connectivity and credentials.

    Errors never include API key material.
    """
    result = await test_provider(req)
    if not result.success:
        raise HTTPException(status_code=422, detail=result.error)
    return result