from datetime import datetime
from flask import Flask, render_template, jsonify
from flask_cors import cross_origin, CORS
import requests
from flask import request
from factory_responses import FactoryResponse
from pymongo import MongoClient
# from celery import Celery
import logging


factory_responses = FactoryResponse()
USED_LANGUAGE = 'es'
CELERY_BROKER_ADDRESS = 'redis://redis:6379/0'

# news = get_collection('localhost', 27017)
app = Flask(__name__, static_folder="./dist/static", template_folder="./dist")
# celery = Celery('tasks', broker=CELERY_BROKER_ADDRESS)

# Configure CORS feature
cors = CORS(app, resources={r"/api/*": {"origins": '*'}})
app.config['CORS_HEADER'] = 'Content-Type'
logger = logging.getLogger(__name__)


@app.route('/api/news', methods=['GET'])
@cross_origin(origin='*')
def get_news():
    return jsonify({'result': 'ok'})


@app.route('/api/news', methods=['POST'])
@cross_origin(origin='*')
def post_news():
    content = request.json
    list_ids = []
    if 'urls' not in content or len(content['urls']) == 0:
        return factory_responses.new400()
    return factory_responses.new201({'ids': list_ids})


@app.route('/', defaults={'path': ''})
@app.route('/<path:path>')
def catch_all(path):
    if app.debug:
        return requests.get('http://0.0.0.0:8080/{}'.format(path)).text
    return render_template("index.html")


# @celery.task
# def run_batch(database_id, url):
#        logger.info("Executing analysed batch task")


if __name__ == '__main__':
    host = '0.0.0.0'
    port = 5002
    debug = True
#    celery_argvs = ['worker', '--loglevel=DEBUG']
#    import threading
#    celery_thread = threading.Thread(target=celery.worker_main,
#                                      args=[celery_argvs])
#    celery_thread.start()
    print("Running API REST")
    app.run(host=host, port=port, debug=debug)
