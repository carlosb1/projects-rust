import threading
import traceback

import schedule
import time
import dotenv

from agent_worker import run, run_batch


def main():
    loaded = dotenv.load_dotenv()
    print(f"loaded .env info {loaded}")
    batch_join = threading.Thread(target=run_batch).start()

    print(f"download current data for today")
    run()
    schedule.every().day.at("10:30").do(run)


    try:
        while 1:
            schedule.run_pending()
            time.sleep(10)
    except Exception as e:
        traceback.print_exception(e)
if __name__ == "__main__":
    main()