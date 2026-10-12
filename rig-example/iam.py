
import pulumi
import pulumi_aws as aws


def create_task_role(name: str) -> aws.iam.Role:
    trust_policy = aws.iam.get_policy_document(
        statements=[
            {
                "actions": ["sts:AssumeRole"],
                "principals": [{
                    "type": "Service",
                    "identifiers": [
                        "ecs-tasks.amazonaws.com"
                    ],
                }],
            }
        ]
    )

    return aws.iam.Role(
        name,
        assume_role_policy=trust_policy.json,
    )


def grant_queue_access(
    name: str,
    role: aws.iam.Role,
    queue: aws.sqs.Queue,
    actions: list[str],
) -> aws.iam.RolePolicy:
    return aws.iam.RolePolicy(
        name,
        role=role.id,
        policy=pulumi.Output.json_dumps({
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Action": actions,
                "Resource": queue.arn,
            }],
        }),
    )


def configure_job_roles(queues: dict):
    # 1. Create ECS task roles
    api_role = create_task_role("api-job-role")
    python_role = create_task_role("python-job-role")
    rust_role = create_task_role("rust-job-role")

    # 2. Axum publishes jobs to both queues
    api_python_policy = grant_queue_access(
        "api-send-python",
        api_role,
        queues["python"],
        ["sqs:SendMessage"],
    )

    api_rust_policy = grant_queue_access(
        "api-send-rust",
        api_role,
        queues["rust"],
        ["sqs:SendMessage"],
    )

    # 3. Workers consume their respective queues
    consumer_actions = [
        "sqs:ReceiveMessage",
        "sqs:DeleteMessage",
        "sqs:ChangeMessageVisibility",
    ]

    python_policy = grant_queue_access(
        "python-consume",
        python_role,
        queues["python"],
        consumer_actions,
    )

    rust_policy = grant_queue_access(
        "rust-consume",
        rust_role,
        queues["rust"],
        consumer_actions,
    )

    # 4. Return roles and policies
    roles = {
        "api": api_role,
        "python": python_role,
        "rust": rust_role,
    }

    policies = {
        "api": [api_python_policy, api_rust_policy],
        "python": [python_policy],
        "rust": [rust_policy],
    }

    return roles, policies
