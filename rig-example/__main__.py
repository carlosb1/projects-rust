"""An AWS Python Pulumi program."""

import json

import pulumi
import pulumi_aws as aws
import pulumi_awsx as awsx

from iam import configure_job_roles
from queues import create_job_queues
from remotefs import (
    create_rustfs_credentials,
    create_rustfs_service,
    create_rustfs_discovery,
)
from workers import create_worker
from database import create_database


CONTAINER_PORT = 3000
QDRANT_PORT = 6333
QDRANT_GRPC_PORT = 6334

# Solo para la API, que comparte tarea con Qdrant.
QDRANT_LOCAL_URL = f"http://127.0.0.1:{QDRANT_PORT}"

# Namespace independiente del utilizado por RustFS.
QDRANT_PRIVATE_URL = f"http://qdrant.vector.internal:{QDRANT_PORT}"

STORAGE_ENDPOINT = "http://rustfs.rig.internal:9000"


def env(name, value):
    return awsx.ecs.TaskDefinitionKeyValuePairArgs(
        name=name,
        value=value,
    )


def secret(name, arn):
    return awsx.ecs.TaskDefinitionSecretArgs(
        name=name,
        value_from=arn,
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


def create_application_execution_role(secret_arns):
    """Rol utilizado por ECS para iniciar API y workers."""

    role = aws.iam.Role(
        "application-execution-role",
        assume_role_policy=json.dumps({
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Principal": {
                    "Service": "ecs-tasks.amazonaws.com",
                },
                "Action": "sts:AssumeRole",
            }],
        }),
    )

    base_policy = aws.iam.RolePolicyAttachment(
        "application-execution-base-policy",
        role=role.name,
        policy_arn=(
            "arn:aws:iam::aws:policy/"
            "service-role/AmazonECSTaskExecutionRolePolicy"
        ),
    )

    secrets_policy = aws.iam.RolePolicy(
        "application-execution-secrets-policy",
        role=role.id,
        policy=pulumi.Output.json_dumps({
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Action": "secretsmanager:GetSecretValue",
                "Resource": secret_arns,
            }],
        }),
    )

    return role, [base_policy, secrets_policy]


def up_infra(
    app_image_uri,
    python_image_uri,
    rust_image_uri,
    queues,
    roles,
    policies,
):
    region = "eu-west-1"

    # ---------------------------------------------------------
    # VPC y cluster
    # ---------------------------------------------------------

    vpc = awsx.ec2.Vpc(
        "rig-example-vpc",
        number_of_availability_zones=2,
        enable_dns_hostnames=True,
        enable_dns_support=True,
        subnet_specs=[
            awsx.ec2.SubnetSpecArgs(
                type=awsx.ec2.SubnetType.PUBLIC,
            ),
            awsx.ec2.SubnetSpecArgs(
                type=awsx.ec2.SubnetType.ISOLATED,
            ),
        ],
        nat_gateways=awsx.ec2.NatGatewayConfigurationArgs(
            strategy=awsx.ec2.NatGatewayStrategy.NONE,
        ),
    )

    cluster = aws.ecs.Cluster("llm-cluster")

    # ---------------------------------------------------------
    # Grupos de seguridad
    # ---------------------------------------------------------

    alb_sg = aws.ec2.SecurityGroup(
        "rig-example-alb-sg",
        vpc_id=vpc.vpc_id,
        description="HTTP access to application",
        ingress=[{
            "protocol": "tcp",
            "from_port": 80,
            "to_port": 80,
            "cidr_blocks": ["0.0.0.0/0"],
        }],
        egress=[{
            "protocol": "-1",
            "from_port": 0,
            "to_port": 0,
            "cidr_blocks": ["0.0.0.0/0"],
        }],
    )
    aws.ec2.SecurityGroupRule(
        "qdrant-public-ingress",
        type="ingress",
        security_group_id=alb_sg.id,
        protocol="tcp",
        from_port=QDRANT_PORT,
        to_port=QDRANT_PORT,
        cidr_blocks=["0.0.0.0/0"],
    )

    ecs_sg = aws.ec2.SecurityGroup(
        "rig-example-ecs-sg",
        vpc_id=vpc.vpc_id,
        description="Application and workers",
        # Las reglas de entrada se crean por separado.
        egress=[{
            "protocol": "-1",
            "from_port": 0,
            "to_port": 0,
            "cidr_blocks": ["0.0.0.0/0"],
        }],
    )

    aws.ec2.SecurityGroupRule(
        "qdrant-from-alb",
        type="ingress",
        security_group_id=ecs_sg.id,
        source_security_group_id=alb_sg.id,
        protocol="tcp",
        from_port=QDRANT_PORT,
        to_port=QDRANT_PORT,
    )

    api_ingress = aws.ec2.SecurityGroupRule(
        "api-from-alb",
        type="ingress",
        security_group_id=ecs_sg.id,
        source_security_group_id=alb_sg.id,
        protocol="tcp",
        from_port=CONTAINER_PORT,
        to_port=CONTAINER_PORT,
    )

    qdrant_rest_ingress = aws.ec2.SecurityGroupRule(
        "qdrant-rest-from-workers",
        type="ingress",
        security_group_id=ecs_sg.id,
        source_security_group_id=ecs_sg.id,
        protocol="tcp",
        from_port=QDRANT_PORT,
        to_port=QDRANT_PORT,
    )

    qdrant_grpc_ingress = aws.ec2.SecurityGroupRule(
        "qdrant-grpc-from-workers",
        type="ingress",
        security_group_id=ecs_sg.id,
        source_security_group_id=ecs_sg.id,
        protocol="tcp",
        from_port=QDRANT_GRPC_PORT,
        to_port=QDRANT_GRPC_PORT,
    )

    rustfs_sg = aws.ec2.SecurityGroup(
        "rustfs-sg",
        vpc_id=vpc.vpc_id,
        description="RustFS service",
        ingress=[
            {
                "protocol": "tcp",
                "from_port": 9001,
                "to_port": 9001,
                "security_groups": [alb_sg.id],
            },
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

    # ---------------------------------------------------------
    # Base de datos y secretos
    # ---------------------------------------------------------

    credentials = create_rustfs_credentials()

    database_secret, _, database_dependencies = create_database(
        vpc.vpc_id,
        vpc.isolated_subnet_ids,
        ecs_sg.id,
    )

    # Un execution role compartido por API y workers.
    execution_role, execution_dependencies = (
        create_application_execution_role([
            database_secret.arn,
            credentials.access_key_secret.arn,
            credentials.secret_key_secret.arn,
        ])
    )

    # ---------------------------------------------------------
    # Service Discovery
    # ---------------------------------------------------------

    rustfs_discovery = create_rustfs_discovery(vpc.vpc_id)

    qdrant_namespace = aws.servicediscovery.PrivateDnsNamespace(
        "qdrant-private-namespace",
        name="vector.internal",
        vpc=vpc.vpc_id,
    )

    qdrant_discovery = aws.servicediscovery.Service(
        "qdrant-discovery",
        name="qdrant",
        dns_config={
            "namespace_id": qdrant_namespace.id,
            "dns_records": [{
                "type": "A",
                "ttl": 10,
            }],
            "routing_policy": "MULTIVALUE",
        },
        health_check_custom_config={
            "failure_threshold": 1,
        },
    )

    # ---------------------------------------------------------
    # ALB para la API
    # ---------------------------------------------------------

    lb = awsx.lb.ApplicationLoadBalancer(
        "rig-example-lb",
        subnet_ids=vpc.public_subnet_ids,
        security_groups=[alb_sg.id],
        default_target_group={
            "port": CONTAINER_PORT,
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
    # Consola RustFS
    # ---------------------------------------------------------

    # Limita el acceso a tu IP:
    # pulumi config set adminCidr TU_IP_PUBLICA/32
    config = pulumi.Config()
    admin_cidr = config.get("adminCidr") or "0.0.0.0/0"

    console_ingress = aws.ec2.SecurityGroupRule(
        "rustfs-console-ingress",
        type="ingress",
        security_group_id=alb_sg.id,
        protocol="tcp",
        from_port=9001,
        to_port=9001,
        cidr_blocks=[admin_cidr],
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

#### target groups for qdrant

    qdrant_target_group = aws.lb.TargetGroup(
        "qdrant-target-group",
        vpc_id=vpc.vpc_id,
        port=QDRANT_PORT,
        protocol="HTTP",
        target_type="ip",
        health_check={
            "path": "/",
            "protocol": "HTTP",
            "matcher": "200",
        },
    )

    qdrant_listener = aws.lb.Listener(
        "qdrant-listener",
        load_balancer_arn=lb.load_balancer.arn,
        port=QDRANT_PORT,
        protocol="HTTP",
        default_actions=[
            aws.lb.ListenerDefaultActionArgs(
                type="forward",
                target_group_arn=qdrant_target_group.arn,
            ),
        ],
    )

    # ---------------------------------------------------------
    # API + Qdrant en la misma tarea
    # ---------------------------------------------------------

    shared_dependencies = [
        *database_dependencies,
        *credentials.dependencies,
        *execution_dependencies,
    ]

    service = awsx.ecs.FargateService(
        "rig-example-service",
        cluster=cluster.arn,
        desired_count=1,
        network_configuration=aws.ecs.ServiceNetworkConfigurationArgs(
            subnets=vpc.public_subnet_ids,
            assign_public_ip=True,
            security_groups=[ecs_sg.id],
        ),
        # Registra la IP privada de esta tarea en Cloud Map.
        service_registries=aws.ecs.ServiceServiceRegistriesArgs(
            registry_arn=qdrant_discovery.arn,
        ),
        health_check_grace_period_seconds=60,
        task_definition_args=awsx.ecs.FargateServiceTaskDefinitionArgs(
            cpu="512",
            memory="1024",
            task_role={
                "role_arn": roles["api"].arn,
            },
            execution_role={
                "role_arn": execution_role.arn,
            },
            containers={
                "rig-example": (
                    awsx.ecs.TaskDefinitionContainerDefinitionArgs(
                        name="rig-example",
                        image=app_image_uri,
                        cpu=256,
                        memory=512,
                        essential=True,
                        secrets=[
                            secret(
                                "DATABASE_URL",
                                database_secret.arn,
                            ),
                            secret(
                                "RUSTFS_ACCESS_KEY",
                                credentials.access_key_secret.arn,
                            ),
                            secret(
                                "RUSTFS_SECRET_KEY",
                                credentials.secret_key_secret.arn,
                            ),
                        ],
                        environment=[
                            env("QDRANT_URL", QDRANT_LOCAL_URL),
                            env("PYTHON_QUEUE_URL", queues["python"].url),
                            env("RUST_QUEUE_URL", queues["rust"].url),
                            env("S3_ENDPOINT", STORAGE_ENDPOINT),
                            env("S3_BUCKET", "documents"),
                            env("AWS_REGION", region),
                            env("AWS_DEFAULT_REGION", region),
                        ],
                        port_mappings=[
                            awsx.ecs.TaskDefinitionPortMappingArgs(
                                container_port=CONTAINER_PORT,
                                target_group=lb.default_target_group,
                            ),
                        ],
                    )
                ),
                "qdrant": (
                    awsx.ecs.TaskDefinitionContainerDefinitionArgs(
                        name="qdrant",
                        image=config.get("qdrantImage") or "qdrant/qdrant:dev",
                        cpu=256,
                        memory=512,
                        essential=True,
                        port_mappings=[
                            awsx.ecs.TaskDefinitionPortMappingArgs(
                                container_port=QDRANT_PORT,
                                target_group=qdrant_target_group,
                            ),
                            awsx.ecs.TaskDefinitionPortMappingArgs(
                                container_port=QDRANT_GRPC_PORT,
                            ),
                        ],
                    )
                ),
            },
        ),
        opts=pulumi.ResourceOptions(
            depends_on=[
                *policies["api"],
                *shared_dependencies,
                api_ingress,
                qdrant_listener,
            ],
        ),
    )

    # ---------------------------------------------------------
    # RustFS
    # ---------------------------------------------------------

    rustfs_task_role = aws.iam.Role(
        "rustfs-task-role",
        assume_role_policy=json.dumps({
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Principal": {
                    "Service": "ecs-tasks.amazonaws.com",
                },
                "Action": "sts:AssumeRole",
            }],
        }),
    )

    rustfs = create_rustfs_service(
        cluster_arn=cluster.arn,
        subnet_ids=vpc.public_subnet_ids,
        task_security_group_id=rustfs_sg.id,
        execution_role_arn=credentials.execution_role.arn,
        task_role_arn=rustfs_task_role.arn,
        access_key_secret_arn=credentials.access_key_secret.arn,
        secret_key_secret_arn=credentials.secret_key_secret.arn,
        target_group_arn=rustfs_tg.arn,
        discovery_service_arn=rustfs_discovery.arn,
        depends_on=[
            rustfs_listener,
            console_ingress,
            *credentials.dependencies,
        ],
    )

    # ---------------------------------------------------------
    # Workers
    # ---------------------------------------------------------

    workers = {}

    for worker_type, image_uri in [
        ("python", python_image_uri),
        ("rust", rust_image_uri),
    ]:
        workers[worker_type] = create_worker(
            name=f"{worker_type}-worker",
            cluster=cluster,
            vpc=vpc,
            security_group=ecs_sg,
            image_uri=image_uri,
            task_role=roles[worker_type],
            queue_url=queues[worker_type].url,
            storage_endpoint=STORAGE_ENDPOINT,
            bucket_name="documents",
            qdrant_url=QDRANT_PRIVATE_URL,
            region=region,
            database_url_secret_arn=database_secret.arn,
            access_key_secret_arn=credentials.access_key_secret.arn,
            secret_key_secret_arn=credentials.secret_key_secret.arn,
            execution_role_arn=execution_role.arn,
            depends_on=[
                *policies[worker_type],
                *shared_dependencies,
                qdrant_discovery,
                qdrant_rest_ingress,
                qdrant_grpc_ingress,
            ],
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
        "rustfs_dashboard",
        pulumi.Output.concat(
            "http://",
            lb.load_balancer.dns_name,
            ":9001",
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

    pulumi.export("qdrant_private_url", QDRANT_PRIVATE_URL)
    pulumi.export("ecs_cluster", cluster.name)
    pulumi.export("ecs_service", service.service.name)

    for worker_type, worker in workers.items():
        pulumi.export(
            f"{worker_type}_worker_service",
            worker.service.name,
        )


def main():
    queues = create_job_queues()
    roles, policies = configure_job_roles(queues)

    app_repository = awsx.ecr.Repository(
        "rig-example-repository",
        force_delete=True,
    )

    app_image = awsx.ecr.Image(
        "rig-example",
        repository_url=app_repository.url,
        context="./app",
        platform="linux/amd64",
    )

    python_image = build_worker_image(
        "python-worker",
        "./pyagents",
    )

    rust_image = build_worker_image(
        "rust-worker",
        "./rust-worker",
    )

    up_infra(
        app_image_uri=app_image.image_uri,
        python_image_uri=python_image.image_uri,
        rust_image_uri=rust_image.image_uri,
        queues=queues,
        roles=roles,
        policies=policies,
    )


if __name__ == "__main__":
    main()