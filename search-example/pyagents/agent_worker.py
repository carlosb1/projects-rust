from qdrant_client import QdrantClient
from qdrant_client.models import VectorParams, Distance
from datetime import datetime, timedelta
import dotenv
import os

loaded = dotenv.load_dotenv()

qdrant_host = os.environ.get("QDRANT_HOST","0.0.0.0")
qdrant_port = int(os.environ.get("QDRANT_PORT",6333))

loaded = dotenv.load_dotenv()
print(loaded)



client = QdrantClient(host=qdrant_host, port=qdrant_port)


def str_pregenerate_date(d) -> str:
    return d.strftime("%Y%m%d")
def decrement_days(d, count):
    return d - timedelta(days=count)

def initialize_db():
    database_name = "boe_db"
    if not client.collection_exists(database_name):
        client.create_collection(
            collection_name=database_name,
            vectors_config=VectorParams(size=100, distance=Distance.COSINE),
        )

# initialization
initialize_db()












