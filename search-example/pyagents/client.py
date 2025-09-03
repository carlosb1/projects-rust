import redis
import os
import dotenv

loaded = dotenv.load_dotenv()
print(loaded)

#LOCAL_HOST_NAME="host.docker.internal" #localhost
LOCAL_HOST_NAME = "0.0.0.0"


url = os.environ.get("URL_QUEUE", f"redis://{LOCAL_HOST_NAME}:6379/0")
r = redis.from_url(url)

entry = {"status" : "run" }
r.lpush("queue", entry)
