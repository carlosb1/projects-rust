
import pulumi
import pulumi_aws as aws


def create_queue(name):
    dlq = aws.sqs.Queue(
        f"{name}-dlq",
        message_retention_seconds=1209600,
        sqs_managed_sse_enabled=True,
    )

    queue = aws.sqs.Queue(
        name,
        visibility_timeout_seconds=900,
        receive_wait_time_seconds=20,
        sqs_managed_sse_enabled=True,
        redrive_policy=pulumi.Output.json_dumps({
            "deadLetterTargetArn": dlq.arn,
            "maxReceiveCount": 5,
        }),
    )
    return queue

def create_job_queues():
    return {
        "python": create_queue("python-jobs"),
        "rust": create_queue("rust-jobs"),
    }
