FROM python:3.11-slim

ENV PIP_NO_CACHE_DIR=1
WORKDIR /app

# System deps needed by Chromium/Playwright
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates wget fonts-liberation \
    libasound2 libatk1.0-0 libatk-bridge2.0-0 libatspi2.0-0 \
    libdbus-1-3 libdrm2 libexpat1 libgbm1 libglib2.0-0 \
    libgtk-3-0 libnspr4 libnss3 libnss3-tools \
    libpango-1.0-0 libpangocairo-1.0-0 \
    libx11-6 libx11-xcb1 libxcb1 libxcomposite1 libxdamage1 libxext6 \
    libxfixes3 libxkbcommon0 libxrandr2 libxrender1 libxshmfence1 libxss1 libxtst6 && rm -rf /var/lib/apt/lists/*

COPY . .
RUN pip install -r requirements.txt
#RUN python -m playwright install chromium
CMD ["python", "dispatcher.py"]
