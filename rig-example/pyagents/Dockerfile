
FROM python:3.12-slim

WORKDIR /app

RUN mkdir -p /app/uploads \
    && chown 65532:65532 /app/uploads

ENV HF_HOME=/app/cache/huggingface \
  FASTEMBED_CACHE_PATH=/app/cache/fastembed

RUN mkdir -p /app/cache/huggingface /app/cache/fastembed \
  && chown -R 65532:65532 /app/cache

ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1

COPY requirements.txt .

RUN pip install --no-cache-dir -r requirements.txt

COPY src ./src
COPY pint_es_aliases.txt ./pint_es_aliases.txt

USER 65532:65532

CMD ["python", "-m", "src.worker"]
