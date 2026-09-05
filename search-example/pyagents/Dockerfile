FROM python:3.11-slim

ENV PIP_NO_CACHE_DIR=1
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY requirements.txt .
RUN pip install --upgrade pip \
    && pip install torch==2.8.0 \
        --index-url https://download.pytorch.org/whl/cpu \
    && pip install -r requirements.txt

COPY . .

CMD ["python", "dispatcher.py"]