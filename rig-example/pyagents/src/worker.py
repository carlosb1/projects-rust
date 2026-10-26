import json
import logging
import os
import time
from pathlib import Path
from tempfile import TemporaryDirectory

import boto3
import psycopg2
from fastembed import TextEmbedding
from psycopg2.extras import RealDictCursor
from qdrant_client import QdrantClient

from src.docs import unzip_report, read_pdf
from src.indexing import ReportIndexer
MODEL_NAME = "sentence-transformers/all-MiniLM-L6-v2"

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
        endpoint_url=os.environ["S3_ENDPOINT"],
        aws_access_key_id=os.environ["RUSTFS_ACCESS_KEY"],
        aws_secret_access_key=os.environ["RUSTFS_SECRET_KEY"],
        config=boto3.session.Config(s3={"addressing_style": "path"}),
    )

def get_report_by_job_id(conn, report_id: str):
    with conn:
        with conn.cursor(cursor_factory=RealDictCursor) as cur:
            cur.execute(
                "SELECT * FROM reports WHERE report_id = %s LIMIT 1",
                (report_id,)
            )
            return cur.fetchone()


def update_report_status(conn, report_id: str, status: str):
    with conn:
        with conn.cursor() as cur:
            cur.execute(
                """
                UPDATE reports
                SET status = %s::report_status
                WHERE report_id = %s
                """,
                (status, report_id),
            )

            if cur.rowcount == 0:
                raise ValueError(f"Informe no encontrado: {report_id}")

def download_from_s3(s3_client, bucket: str, key: str, destination: Path):
    obj = s3_client.get_object(Bucket=bucket, Key=key)
    with open(destination, "wb") as f:
        f.write(obj["Body"].read())


def main():
    sqs = create_sqs_client()
    s3 = create_s3_client()
    bucket = os.getenv("S3_BUCKET", "documents")
    qdrant_url = os.getenv("QDRANT_URL", "http://localhost:6333")
    cache_fast_embed = os.getenv(
            "FASTEMBED_CACHE_PATH",
            "/tmp/fastembed",
        )

    queue_url = os.getenv("QUEUE_URL")
    if not queue_url:
        logger.warning("QUEUE_URL not defined")
        return


    collection = os.getenv(
        "QDRANT_COLLECTION",
        "reports_chunks",
    )

    qdrant_client = QdrantClient(
        url=qdrant_url,
        api_key=None,
    )

    embedding_model = TextEmbedding(
        model_name=MODEL_NAME,
        cache_dir=cache_fast_embed,
    )


    indexer = ReportIndexer(collection, qdrant_client, embedding_model)
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
                    key = f"report_{job_id}.zip"
                    logger.info(f"Downloading report from {bucket}/{key}")
                    try:
                        download_from_s3(s3, bucket, key, local_path)

                        logger.info("Unzipping report...")
                        with TemporaryDirectory(prefix="report_") as temp_dir:
                            pdf_files = unzip_report(local_path, Path(temp_dir))

                            if not pdf_files:
                                raise ValueError("ZIP doesn't contain any files")

                            for pdf_path in pdf_files:
                                whole_text = read_pdf(pdf_path)

                                logger.info(
                                    "PDF=%s chars=%s",
                                    pdf_path.name,
                                    len(whole_text),
                                )
                                pdf_relative_path = pdf_path.relative_to(Path(temp_dir)).as_posix()
                                source = f"s3://{bucket}/{key}#pdf={pdf_relative_path}"
                                logger.info(f"Writing report to {bucket}/{key} source={source}")
                                indexer.index_pdf(job_id, source, whole_text)
                                update_report_status(conn, job_id, "processing")
                    except Exception as e:
                        logger.exception("Error indexing  %s  with  %s", job_id, e)
                        update_report_status(conn, job_id, "failed")
                    finally:
                        logger.info(f"Finishing deleting local path {local_path} for job {job_id}")
                        local_path.unlink(missing_ok=True)

                    sqs.delete_message(
                        QueueUrl=queue_url,
                        ReceiptHandle=message["ReceiptHandle"],
                    )
                except Exception as e:
                    logger.exception("Job processing failed %s", e)

        except Exception as e:
            logger.exception("SQS connection failed %s", e)
            time.sleep(5)

if __name__ == "__main__":
    main()