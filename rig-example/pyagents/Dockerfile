
FROM python:3.12-slim

WORKDIR /app

RUN mkdir -p /app/uploads \
    && chown 65532:65532 /app/uploads


ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1

COPY requirements.txt .

RUN pip install \
    --no-cache-dir \
    -r requirements.txt

COPY src ./src

USER 65532:65532

CMD ["python", "-m", "src.worker"]
