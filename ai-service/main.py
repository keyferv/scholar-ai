from fastapi import FastAPI

app = FastAPI(title="ScholarAI", version="0.1.0")


@app.get("/health")
async def health():
    return {"status": "ok", "service": "scholar-ai"}


if __name__ == "__main__":
    import uvicorn

    uvicorn.run("main:app", host="127.0.0.1", port=8321, log_level="info")