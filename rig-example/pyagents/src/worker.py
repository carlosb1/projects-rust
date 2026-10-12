
import json
import logging
import os
import time

import boto3

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


def create_sqs_client():
    return boto3.client(
        "sqs",
        region_name=os.getenv("AWS_REGION", "eu-west-1"),
        endpoint_url=os.getenv("SQS_ENDPOINT") or None,
    )


def process_job(job: dict):
    logger.info("Processing job: %s", job["job_id"])

    # Aquí llamarías a tu lógica de procesamiento.
    # Por ejemplo:
    # processor = PDFProcessor(...)
    # processor.process(job["key"])


def main():
    sqs = create_sqs_client()
    if "QUEUE_URL" not in os.environ:
        logger.warning("Environment variable 'QUEUE_URL' not set")
        return

    queue_url = os.environ["QUEUE_URL"]
    logger.info("Python worker started")

    while True:
        try:
            response = sqs.receive_message(
                QueueUrl=queue_url,
                MaxNumberOfMessages=1,
                WaitTimeSeconds=20,
            )

            for message in response.get("Messages", []):
                try:
                    job = json.loads(message["Body"])
                    process_job(job)

                    sqs.delete_message(
                        QueueUrl=queue_url,
                        ReceiptHandle=message["ReceiptHandle"],
                    )
                except Exception:
                    logger.exception("Job processing failed")

        except Exception:
            logger.exception("SQS connection failed")
            time.sleep(5)


if __name__ == "__main__":
    main()
