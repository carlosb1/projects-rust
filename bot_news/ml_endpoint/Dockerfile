FROM python:3.8.2-buster
ADD . /code
WORKDIR /code
RUN apt-get install -y gcc libc-dev zlib1g-dev libjpeg-dev libxml2-dev
RUN pip install -r requirements.txt
CMD ["python", "app.py"]
