import os

import redis
import json
import subprocess
import dotenv

def main():
    loaded = dotenv.load_dotenv()
    print(loaded)

    #LOCAL_HOST_NAME="host.docker.internal" #localhost
    LOCAL_HOST_NAME = "0.0.0.0"

    url = os.environ.get("URL_QUEUE", f"redis://{LOCAL_HOST_NAME}:6379/0")
    r = redis.from_url(url)

    while True:
        _, task_json = r.blpop("queue")
        task = json.loads(task_json)
        if task['status'] == 'run':
            pass
        print(f"[Dispatcher] Received task: {task}")

        # Throws the process
        subprocess.Popen(["python", "agent_worker.py", json.dumps(task)])

if __name__ == "__main__":
    main()