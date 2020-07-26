FROM python:3.8.2-buster
ADD . /code
WORKDIR /code
RUN apt-get install -y gcc libc-dev zlib1g-dev libjpeg-dev libxml2-dev
RUN pip install -r requirements.txt
RUN python -c "from ml import MyBertTransformerSentimentAnalysis; ml = MyBertTransformerSentimentAnalysis();"
# CMD ["python", "app.py"]
CMD ["gunicorn","-b","0.0.0.0:5002", "app:app"]
