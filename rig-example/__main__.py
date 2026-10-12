"""An AWS Python Pulumi program."""

import os
import sys

import pulumi
import pulumi_aws as aws
import pulumi_awsx as awsx

import json

from iam import configure_job_roles
from queues import create_job_queues
from remotefs import create_rustfs_credentials, create_rustfs_storage, create_efs_policy, create_rustfs_service, \
    create_rustfs_discovery
from workers import create_worker

CONTAINER_PORT = 3000
QDRANT_PORT = 6333
QDRANT_GRPC_PORT = 6334


def build_and_push_ecr_image(name, repository, context):
    return awsx.ecr.Image(
        name,
        repository_url=repository.url,
        context=context,
        platform="linux/amd64",
    )
def build_worker_image(name, context):
    repository = awsx.ecr.Repository(
        f"{name}-repository",
    )

    return awsx.ecr.Image(
        f"{name}-image",
        repository_url=repository.url,
        context=context,
        platform="linux/amd64",
    )




def up_infra(
    app_image_uri,
    container_port,
    python_image_uri,
    rust_image_uri,
    queues,
    roles,
    policies,
    storage_endpoint,
):

    # ---------------------------------------------------------
    # VPC
    # ---------------------------------------------------------

    vpc = awsx.ec2.Vpc(
        "rig-example-vpc",
        number_of_availability_zones=2,
        subnet_specs=[
            awsx.ec2.SubnetSpecArgs(
                type=awsx.ec2.SubnetType.PUBLIC,
            )
        ],
        nat_gateways=awsx.ec2.NatGatewayConfigurationArgs(
            strategy=awsx.ec2.NatGatewayStrategy.NONE,
        ),
    )

    # ---------------------------------------------------------
    # ECS Cluster
    # ---------------------------------------------------------

    cluster = aws.ecs.Cluster(
        "llm-cluster",
    )


    # ---------------------------------------------------------
    # Security Group: ALB
    #
    # Internet -> ALB :80 axum 6333 qdrant
    # ---------------------------------------------------------

    alb_sg = aws.ec2.SecurityGroup(
        "rig-example-alb-sg",
        vpc_id=vpc.vpc_id,
        description="Allow HTTP traffic to the application load balancer",

        ingress=[
            aws.ec2.SecurityGroupIngressArgs(
                protocol="tcp",
                from_port=80,
                to_port=80,
                cidr_blocks=["0.0.0.0/0"],
            ),
            # Internet -> Qdrant dashboard
            aws.ec2.SecurityGroupIngressArgs(
                protocol="tcp",
                from_port=6333,
                to_port=6333,
                cidr_blocks=["0.0.0.0/0"],
            ),
        ],

        egress=[
            aws.ec2.SecurityGroupEgressArgs(
                protocol="-1",
                from_port=0,
                to_port=0,
                cidr_blocks=["0.0.0.0/0"],
            )
        ],
    )

    # ---------------------------------------------------------
    # Security Group: ECS/Fargate
    #
    # For 3000 Axum and 6333 Qdrant
    # ---------------------------------------------------------

    ecs_sg = aws.ec2.SecurityGroup(
        "rig-example-ecs-sg",
        vpc_id=vpc.vpc_id,
        description="Allow traffic from ALB to ECS tasks",
        ingress=[
            # ALB -> Axum
            aws.ec2.SecurityGroupIngressArgs(
                protocol="tcp",
                from_port=3000,
                to_port=3000,
                security_groups=[alb_sg.id],
            ),

            # ALB -> Qdrant dashboard/API
            aws.ec2.SecurityGroupIngressArgs(
                protocol="tcp",
                from_port=6333,
                to_port=6333,
                security_groups=[alb_sg.id],
            ),
        ],

        egress=[
            aws.ec2.SecurityGroupEgressArgs(
                protocol="-1",
                from_port=0,
                to_port=0,
                cidr_blocks=["0.0.0.0/0"],
            )
        ],
    )

    # Security group for rust fs
    rustfs_sg = aws.ec2.SecurityGroup(
        "rustfs-sg",
        vpc_id=vpc.vpc_id,
        description="RustFS Fargate service",
        ingress=[
            # Dashboard desde el ALB
            {
                "protocol": "tcp",
                "from_port": 9001,
                "to_port": 9001,
                "security_groups": [alb_sg.id],
            },
            # API S3 desde tus aplicaciones y workers
            {
                "protocol": "tcp",
                "from_port": 9000,
                "to_port": 9000,
                "security_groups": [ecs_sg.id],
            },
        ],
        egress=[{
            "protocol": "-1",
            "from_port": 0,
            "to_port": 0,
            "cidr_blocks": ["0.0.0.0/0"],
        }],
    )

    console_ingress = aws.ec2.SecurityGroupRule(
        "rustfs-console-ingress",
        type="ingress",
        security_group_id=alb_sg.id,
        protocol="tcp",
        from_port=9001,
        to_port=9001,
        cidr_blocks=["0.0.0.0/0"],
    )


    # ---------------------------------------------------------
    # Application Load Balancer
    # ---------------------------------------------------------
    # IMPORTANT: HERE  we automatize the creation of the target group, listener etc.. for axum and its ports
    lb = awsx.lb.ApplicationLoadBalancer(
        "rig-example-lb",
        subnet_ids=vpc.public_subnet_ids,
        security_groups=[alb_sg.id],

        default_target_group={
            "port": container_port,
            "protocol": "HTTP",
            "target_type": "ip",
            "health_check": {
                "path": "/api/health",
                "protocol": "HTTP",
                "matcher": "200",
            },
        },
    )

    # ---------------------------------------------------------
    # Qdrant Target Group
    #
    # ALB :6333 -> Qdrant :6333
    # ---------------------------------------------------------

    qdrant_target_group = aws.lb.TargetGroup(
        "qdrant-target-group",

        vpc_id=vpc.vpc_id,

        port=QDRANT_PORT,
        protocol="HTTP",

        # Required for Fargate / awsvpc
        target_type="ip",

        health_check=aws.lb.TargetGroupHealthCheckArgs(
            path="/",
            protocol="HTTP",
            matcher="200",
        ),
    )

    # ---------------------------------------------------------
    # ALB Listener for Qdrant
    #
    # http://<alb>:6333
    # ---------------------------------------------------------

    qdrant_listener = aws.lb.Listener(
        "qdrant-listener",

        load_balancer_arn=lb.load_balancer.arn,

        port=QDRANT_PORT,
        protocol="HTTP",

        default_actions=[
            aws.lb.ListenerDefaultActionArgs(
                type="forward",
                target_group_arn=qdrant_target_group.arn,
            )
        ],
    )


    rustfs_task_role = aws.iam.Role(
        "rustfs-task-role",
        assume_role_policy=json.dumps({
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Principal": {
                    "Service": "ecs-tasks.amazonaws.com"
                },
                "Action": "sts:AssumeRole",
            }],
        }),
    )



    rustfs_tg = aws.lb.TargetGroup(
        "rustfs-console-tg",
        vpc_id=vpc.vpc_id,
        port=9001,
        protocol="HTTP",
        target_type="ip",
        health_check={
            "path": "/rustfs/console/health",
            "protocol": "HTTP",
            "matcher": "200",
        },
    )

    rustfs_listener = aws.lb.Listener(
        "rustfs-console-listener",
        load_balancer_arn=lb.load_balancer.arn,
        port=9001,
        protocol="HTTP",
        default_actions=[{
            "type": "forward",
            "target_group_arn": rustfs_tg.arn,
        }],
    )

    # ---------------------------------------------------------
    # ECS Fargate Service
    #
    # One task containing:
    #
    #   +-----------------------------+
    #   | Axum           Qdrant        |
    #   | :3000          :6333         |
    #   |    |              ^          |
    #   |    +--------------+          |
    #   |       localhost              |
    #   +-----------------------------+
    # ---------------------------------------------------------

    service = awsx.ecs.FargateService(
        "rig-example-service",
        cluster=cluster.arn,
        network_configuration=aws.ecs.ServiceNetworkConfigurationArgs(
            subnets=vpc.public_subnet_ids,

            # No NAT Gateway, therefore task needs a public IP
            assign_public_ip=True,

            security_groups=[
                ecs_sg.id,
            ],
        ),

        health_check_grace_period_seconds=30,

        task_definition_args=awsx.ecs.FargateServiceTaskDefinitionArgs(
            # Total resources for the Fargate task
            cpu="512",
            memory="1024",
            task_role={ "role_arn": roles["api"].arn },
            containers={

                # -------------------------------------------------
                # Axum
                # -------------------------------------------------

                "rig-example": (
                    awsx.ecs.TaskDefinitionContainerDefinitionArgs(
                        name="rig-example",

                        image=app_image_uri,

                        cpu=256,
                        memory=512,

                        essential=True,

                        environment=[
                            awsx.ecs.TaskDefinitionKeyValuePairArgs(
                                name="QDRANT_URL",
                                value="http://127.0.0.1:6333",
                            ),
                            awsx.ecs.TaskDefinitionKeyValuePairArgs(
                                name="PYTHON_QUEUE_URL",
                                value=queues["python"].url,
                            ),
                            awsx.ecs.TaskDefinitionKeyValuePairArgs(
                                name="RUST_QUEUE_URL",
                                value=queues["rust"].url,
                            ),
                            awsx.ecs.TaskDefinitionKeyValuePairArgs(
                                name="S3_ENDPOINT",
                                value=storage_endpoint,
                            ),
                            awsx.ecs.TaskDefinitionKeyValuePairArgs(
                                name="S3_BUCKET",
                                value="documents",
                            ),
                        ],

                        port_mappings=[
                            awsx.ecs.TaskDefinitionPortMappingArgs(
                                container_port=container_port,
                                target_group=lb.default_target_group,
                            )
                        ],
                    )
                ),

                # -------------------------------------------------
                # Qdrant
                # -------------------------------------------------

                "qdrant": (
                    awsx.ecs.TaskDefinitionContainerDefinitionArgs(
                        name="qdrant",

                        image="qdrant/qdrant:dev",

                        cpu=256,
                        memory=512,

                        essential=True,

                        port_mappings=[
                            # REST API + dashboard
                            awsx.ecs.TaskDefinitionPortMappingArgs(
                                container_port=QDRANT_PORT,
                                target_group=qdrant_target_group,
                            ),

                            # gRPC
                            awsx.ecs.TaskDefinitionPortMappingArgs(
                                container_port=QDRANT_GRPC_PORT,
                            ),
                        ],
                    )
                ),
            },
        ),
        opts=pulumi.ResourceOptions(
            depends_on=policies["api"],
        ),
    )


    python_worker = create_worker(
        name="python-worker",
        cluster=cluster,
        vpc=vpc,
        security_group=ecs_sg,
        image_uri=python_image_uri,
        task_role=roles["python"],
        queue_url=queues["python"].url,
        storage_endpoint=storage_endpoint,
        bucket_name="documents",
        depends_on=policies["python"],
    )

    rust_worker = create_worker(
        name="rust-worker",
        cluster=cluster,
        vpc=vpc,
        security_group=ecs_sg,
        image_uri=rust_image_uri,
        task_role=roles["rust"],
        queue_url=queues["rust"].url,
        storage_endpoint=storage_endpoint,
        bucket_name="documents",
        depends_on=policies["rust"],
    )

    #### EFS configuration

    discovery = create_rustfs_discovery(vpc.vpc_id)
    # ---------------------------------------------------------
    # RustFS credentials
    # ---------------------------------------------------------

    credentials = create_rustfs_credentials()

    # ---------------------------------------------------------
    # RustFS Fargate service
    # ---------------------------------------------------------

    rustfs = create_rustfs_service(
        cluster_arn=cluster.arn,
        subnet_ids=vpc.public_subnet_ids,
        task_security_group_id=rustfs_sg.id,

        execution_role_arn=credentials.execution_role.arn,
        task_role_arn=rustfs_task_role.arn,

        access_key_secret_arn=credentials.access_key_secret.arn,
        secret_key_secret_arn=credentials.secret_key_secret.arn,

        target_group_arn=rustfs_tg.arn,
        discovery_service_arn=discovery.arn,
        depends_on=[
            rustfs_listener,
            *credentials.dependencies,
        ],
    )

    pulumi.export(
        "rustfs_dashboard",
        pulumi.Output.concat(
            "http://",
            lb.load_balancer.dns_name,
            ":9001",
        ),
    )


    pulumi.export(
        "python_worker_service",
        python_worker.service.name,
    )

    pulumi.export(
        "rust_worker_service",
        rust_worker.service.name,
    )



    # ---------------------------------------------------------
    # Outputs
    # ---------------------------------------------------------

    pulumi.export(
        "url",
        pulumi.Output.concat(
            "http://",
            lb.load_balancer.dns_name,
        ),
    )

    pulumi.export(
        "qdrant_url",
        pulumi.Output.concat(
            "http://",
            lb.load_balancer.dns_name,
            ":6333",
        ),
    )

    pulumi.export(
        "qdrant_dashboard",
        pulumi.Output.concat(
            "http://",
            lb.load_balancer.dns_name,
            ":6333/dashboard",
        ),
    )

    pulumi.export(
        "ecs_cluster",
        cluster.name,
    )

    pulumi.export(
        "ecs_service",
        service.service.name,
    )


def main() -> None:

    region = aws.config.region
    home_dir = os.path.expanduser("~")

    pulumi.info(
        f"region={region} home dir={home_dir}"
    )

    aws_credentials_path = os.path.join(
        home_dir,
        ".aws",
    )

    pulumi.info(
        f"aws credentials path={aws_credentials_path}"
    )

    if not os.path.exists(aws_credentials_path):
        pulumi.error(
            f"The .aws directory is missing in "
            f"{aws_credentials_path}."
        )
        sys.exit(1)

    # Create job infrastructure

    # Colas SQS

    queues = create_job_queues()

    # Roles IAM y políticas
    roles, policies = configure_job_roles(queues)

    # ---------------------------------------------------------
    # ECR
    # ---------------------------------------------------------

    app_repository = awsx.ecr.Repository(
        "rig-example-repository",
        force_delete=True,
    )


    # Axum: conserva su nombre lógico actual
    app_image = build_and_push_ecr_image(
        "rig-example",
        app_repository,
        "./app",
    )

    # Nuevas imágenes de los workers
    python_image = build_worker_image(
        "python-worker",
        "./pyagents",
    )

    rust_image = build_worker_image(
        "rust-worker",
        "./rust-worker",
    )


    app_image.image_uri.apply(
        lambda uri: pulumi.info(
            f"Image URI={uri}"
        )
    )

    # ---------------------------------------------------------
    # Infrastructure
    # ---------------------------------------------------------
    config = pulumi.Config()
    #storage_endpoint = config.require("rustfsEndpoint")
    storage_endpoint="http://rustfs.rig.internal:9000"

    # Reutilizar infraestructura actual
    up_infra(
        app_image_uri=app_image.image_uri,
        container_port=CONTAINER_PORT,
        python_image_uri=python_image.image_uri,
        rust_image_uri=rust_image.image_uri,
        queues=queues,
        roles=roles,
        policies=policies,
        storage_endpoint=storage_endpoint,
    )


if __name__ == "__main__":
    main()
