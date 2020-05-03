from datetime import datetime
from flask import Flask, render_template, jsonify
from flask_cors import cross_origin, CORS
import requests
from flask import request
from factory_responses import FactoryResponse
from pymongo import MongoClient
from celery import Celery
import logging


def get_collection(host: str, port: int):
    connection = MongoClient(host, port)
    db = connection['db_news']
    news = db['news']
    return news

factory_responses = FactoryResponse()
USED_LANGUAGE = 'es'
CELERY_BROKER_ADDRESS = 'redis://0.0.0.0:6379/0'

news = get_collection('localhost', 27017)
app = Flask(__name__, static_folder="./dist/static", template_folder="./dist")
celery = Celery('tasks', broker=CELERY_BROKER_ADDRESS)

# Configure CORS feature
cors = CORS(app, resources={r"/api/*": {"origins": '*'}})
app.config['CORS_HEADER'] = 'Content-Type'


@app.route('/api/news', methods=['GET'])
@cross_origin(origin='*')
def get_news():
    app.logger.info("Receiving get query")
    return jsonify({'result': 'ok'})


@app.route('/api/news', methods=['POST'])
@cross_origin(origin='*')
def post_news():
    app.logger.info("Receiving post query")
    content = request.json

    if not content or type(content) is not list:
        app.logger.info("Content has not correct format")
        return factory_responses.new400()
    for entry in content:
        if type(entry) is not dict and not (all(elem in ['link', 'title', 'description']) for elem in entry):
            app.logger.info(f'Content doesn include enough info {str(entry)}')
            continue
        app.logger.info("-----------------")
        app.logger.info(f'{str(entry)}')
        app.logger.info("-----------------")
        run_batch.apply_async((entry['link'], entry['title'], entry['description']))

    list_ids = []
    return factory_responses.new201({'ids': list_ids})

@celery.task
def run_batch(link: str, title: str, description: str):
        app.logger.info(f'Executing analysed batch task {str(link)}')


if __name__ == '__main__':
    host = '0.0.0.0'
    port = 5002
    debug = True

    gunicorn_error_logger = logging.getLogger('gunicorn.error')
    app.logger.handlers.extend(gunicorn_error_logger.handlers)
    app.logger.setLevel(logging.DEBUG)

    celery_argvs = ['worker', '--loglevel=DEBUG']
    import threading
    celery_thread = threading.Thread(target=celery.worker_main,
                                       args=[celery_argvs])
    celery_thread.start()
    app.logger.info("Running API REST")
    app.run(host=host, port=port, debug=debug)
