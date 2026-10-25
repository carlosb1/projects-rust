import json
import logging
import os
import time
from pathlib import Path

import boto3
import psycopg2
from psycopg2.extras import RealDictCursor

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

UPLOADS_DIR = Path("./uploads")
UPLOADS_DIR.mkdir(exist_ok=True)

def get_db_connection():
    db_url = os.getenv("DATABASE_URL")
    if not db_url:
        raise RuntimeError("DATABASE_URL not defined")
    return psycopg2.connect(db_url)

def create_sqs_client():
    return boto3.client(
        "sqs",
        region_name=os.getenv("AWS_REGION", "eu-west-1"),
        endpoint_url=os.getenv("SQS_ENDPOINT") or None,
        aws_access_key_id=os.getenv("AWS_ACCESS_KEY_ID"),
        aws_secret_access_key=os.getenv("AWS_SECRET_ACCESS_KEY"),
    )

def create_s3_client():
    return boto3.client(
        "s3",
        region_name=os.getenv("AWS_REGION", "eu-west-1"),
        endpoint_url=os.getenv("S3_ENDPOINT") or None,
        aws_access_key_id=os.getenv("AWS_ACCESS_KEY_ID"),
        aws_secret_access_key=os.getenv("AWS_SECRET_ACCESS_KEY"),
        config=boto3.session.Config(s3={"addressing_style": "path"}),
    )

def get_report_by_job_id(conn, report_id: str):
    with conn.cursor(cursor_factory=RealDictCursor) as cur:
        cur.execute(
            "SELECT * FROM reports WHERE report_id = %s LIMIT 1",
            (report_id,)
        )
        return cur.fetchone()

def download_from_s3(s3_client, bucket: str, key: str, destination: Path):
    obj = s3_client.get_object(Bucket=bucket, Key=key)
    with open(destination, "wb") as f:
        f.write(obj["Body"].read())

def process_report(report: dict, local_file: Path):
    logger.info("Procesando report: %s", report)
    # Aquí haces tu lógica real
    # por ejemplo: leer PDF/ZIP, extraer texto, indexar en Qdrant, etc.
    logger.info("Archivo local: %s", local_file)

def main():
    sqs = create_sqs_client()
    s3 = create_s3_client()
    bucket = os.getenv("S3_BUCKET", "documents")

    queue_url = os.getenv("QUEUE_URL")
    if not queue_url:
        logger.warning("QUEUE_URL not defined")
        return

    conn = get_db_connection()

    while True:
        try:
            response = sqs.receive_message(
                QueueUrl=queue_url,
                MaxNumberOfMessages=1,
                WaitTimeSeconds=20,
            )

            for message in response.get("Messages", []):
                try:
                    body = message["Body"]
                    job = json.loads(body)

                    job_id = str(job["job_id"])
                    logger.info("Processing job: %s", job_id)

                    report = get_report_by_job_id(conn, job_id)
                    if not report:
                        logger.warning("Report not found for job_id=%s", job_id)
                        continue

                    local_path = UPLOADS_DIR / job_id

                    key = f"report_{UPLOADS_DIR}/{job_id}.zip"
                    download_from_s3(s3, bucket, key, local_path)

                    try:
                        process_report(report, local_path)
                    finally:
                        if local_path.exists():
                            local_path.unlink()

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