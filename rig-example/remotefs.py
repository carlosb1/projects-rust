import json
from dataclasses import dataclass

import pulumi
import pulumi_aws as aws


@dataclass
class RustfsCredentials:
    execution_role: aws.iam.Role
    access_key_secret: aws.secretsmanager.Secret
    secret_key_secret: aws.secretsmanager.Secret
    dependencies: list


def create_rustfs_credentials(name="rustfs") -> RustfsCredentials:
    config = pulumi.Config()

    access_key = config.require_secret("rustfsAccessKey")
    secret_key = config.require_secret("rustfsSecretKey")

    access_secret = aws.secretsmanager.Secret(
        f"{name}-access-key",
    )

    secret_secret = aws.secretsmanager.Secret(
        f"{name}-secret-key",
    )

    access_version = aws.secretsmanager.SecretVersion(
        f"{name}-access-version",
        secret_id=access_secret.id,
        secret_string=access_key,
    )

    secret_version = aws.secretsmanager.SecretVersion(
        f"{name}-secret-version",
        secret_id=secret_secret.id,
        secret_string=secret_key,
    )

    execution_role = aws.iam.Role(
        f"{name}-execution-role",
        assume_role_policy=json.dumps({
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Principal": {"Service": "ecs-tasks.amazonaws.com"},
                "Action": "sts:AssumeRole",
            }],
        }),
    )

    execution_policy = aws.iam.RolePolicyAttachment(
        f"{name}-execution-policy",
        role=execution_role.name,
        policy_arn=(
            "arn:aws:iam::aws:policy/service-role/"
            "AmazonECSTaskExecutionRolePolicy"
        ),
    )

    secrets_policy = aws.iam.RolePolicy(
        f"{name}-secrets-policy",
        role=execution_role.id,
        policy=pulumi.Output.all(
            access_secret.arn,
            secret_secret.arn,
        ).apply(lambda arns: json.dumps({
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Action": ["secretsmanager:GetSecretValue"],
                "Resource": arns,
            }],
        })),
    )

    return RustfsCredentials(
        execution_role=execution_role,
        access_key_secret=access_secret,
        secret_key_secret=secret_secret,
        dependencies=[
            access_version,
            secret_version,
            execution_policy,
            secrets_policy,
        ],
    )

@dataclass
class RustfsStorage:
    filesystem: aws.efs.FileSystem
    access_point: aws.efs.AccessPoint
    mount_targets: list
    security_group: aws.ec2.SecurityGroup


def create_rustfs_storage(
    *,
    vpc_id,
    subnet_ids,
    task_security_group_id,
    name="rustfs",
) -> RustfsStorage:

    filesystem = aws.efs.FileSystem(
        f"{name}-efs",
        encrypted=True,
        throughput_mode="bursting",
        tags={"Name": f"{name}-efs"},
        opts=pulumi.ResourceOptions(protect=True),
    )

    security_group = aws.ec2.SecurityGroup(
        f"{name}-efs-sg",
        vpc_id=vpc_id,
        description="NFS access from RustFS ECS tasks",
        ingress=[{
            "protocol": "tcp",
            "from_port": 2049,
            "to_port": 2049,
            "security_groups": [task_security_group_id],
        }],
        egress=[],
    )

    access_point = aws.efs.AccessPoint(
        f"{name}-access-point",
        file_system_id=filesystem.id,
        posix_user={
            "uid": 10001,
            "gid": 10001,
        },
        root_directory={
            "path": "/rustfs",
            "creation_info": {
                "owner_uid": 10001,
                "owner_gid": 10001,
                "permissions": "0750",
            },
        },
    )

    mount_targets = [
        aws.efs.MountTarget(
            f"{name}-mount-{i}",
            file_system_id=filesystem.id,
            subnet_id=subnet_id,
            security_groups=[security_group.id],
        )
        for i, subnet_id in enumerate(subnet_ids)
    ]

    return RustfsStorage(
        filesystem=filesystem,
        access_point=access_point,
        mount_targets=mount_targets,
        security_group=security_group,
    )



import json
from dataclasses import dataclass
from typing import Optional, Sequence

import pulumi
import pulumi_aws as aws


def create_efs_policy(task_role, storage):
    # Permisos IAM para montar EFS
    efs_policy = aws.iam.RolePolicy(
        "rustfs-efs-policy",
        role=task_role.id,
        policy=storage.filesystem.arn.apply(
            lambda arn: json.dumps({
                "Version": "2012-10-17",
                "Statement": [{
                    "Effect": "Allow",
                    "Action": [
                        "elasticfilesystem:ClientMount",
                        "elasticfilesystem:ClientWrite",
                    ],
                    "Resource": arn,
                }],
            })
        ),
    )
    return efs_policy

def create_rustfs_discovery(vpc_id):
    namespace = aws.servicediscovery.PrivateDnsNamespace(
        "rig-private-namespace",
        name="rig.internal",
        vpc=vpc_id,
    )

    discovery_service = aws.servicediscovery.Service(
        "rustfs-discovery",
        name="rustfs",
        dns_config={
            "namespace_id": namespace.id,
            "dns_records": [{
                "type": "A",
                "ttl": 10,
            }],
            "routing_policy": "MULTIVALUE",
        },
    )

    return discovery_service

import json
from dataclasses import dataclass
from typing import Optional, Sequence

import pulumi
import pulumi_aws as aws



@dataclass
class RustfsService:
    service: aws.ecs.Service
    task: aws.ecs.TaskDefinition
    log_group: aws.cloudwatch.LogGroup


def create_rustfs_service(
    *,
    cluster_arn: pulumi.Input[str],
    subnet_ids: pulumi.Input[Sequence[str]],
    task_security_group_id: pulumi.Input[str],

    execution_role_arn: pulumi.Input[str],
    task_role_arn: pulumi.Input[str],

    access_key_secret_arn: pulumi.Input[str],
    secret_key_secret_arn: pulumi.Input[str],

    target_group_arn: pulumi.Input[str],
    discovery_service_arn: pulumi.Input[str],
    name: str = "rustfs",
    image: str = "rustfs/rustfs:latest",
    region: str = "eu-west-1",
    depends_on: Optional[Sequence[pulumi.Resource]] = None,
) -> RustfsService:

    # --------------------------------------------------
    # CloudWatch Logs
    # --------------------------------------------------

    log_group = aws.cloudwatch.LogGroup(
        f"{name}-logs",
        name=f"/ecs/{name}",
        retention_in_days=7,
    )

    # --------------------------------------------------
    # Container Definition
    # --------------------------------------------------

    container_definitions = pulumi.Output.all(
        access_key_secret_arn,
        secret_key_secret_arn,
    ).apply(lambda secrets: json.dumps([{
        "name": name,
        "image": image,
        "essential": True,

        "portMappings": [
            {
                "containerPort": 9000,
                "protocol": "tcp",
            },
            {
                "containerPort": 9001,
                "protocol": "tcp",
            },
        ],

        "environment": [
            {
                "name": "RUSTFS_VOLUMES",
                "value": "/data",
            },
            {
                "name": "RUSTFS_ADDRESS",
                "value": "0.0.0.0:9000",
            },
            {
                "name": "RUSTFS_CONSOLE_ADDRESS",
                "value": "0.0.0.0:9001",
            },
            {
                "name": "RUSTFS_CONSOLE_ENABLE",
                "value": "true",
            },
        ],

        # Injected by ECS at container startup
        "secrets": [
            {
                "name": "RUSTFS_ACCESS_KEY",
                "valueFrom": secrets[0],
            },
            {
                "name": "RUSTFS_SECRET_KEY",
                "valueFrom": secrets[1],
            },
        ],

        # Local ephemeral task volume
        "mountPoints": [{
            "sourceVolume": "rustfs-data",
            "containerPath": "/data",
            "readOnly": False,
        }],

        "logConfiguration": {
            "logDriver": "awslogs",
            "options": {
                "awslogs-group": f"/ecs/{name}",
                "awslogs-region": region,
                "awslogs-stream-prefix": name,
            },
        },
    }]))

    # --------------------------------------------------
    # ECS Fargate Task Definition
    # --------------------------------------------------

    task = aws.ecs.TaskDefinition(
        f"{name}-task",
        family=name,

        requires_compatibilities=["FARGATE"],
        network_mode="awsvpc",

        cpu="1024",
        memory="2048",

        execution_role_arn=execution_role_arn,
        task_role_arn=task_role_arn,

        container_definitions=container_definitions,

        # Total ephemeral storage available to the task
        ephemeral_storage={
            "size_in_gib": 30,
        },

        # No EFS: task-local storage
        volumes=[{
            "name": "rustfs-data",
        }],

        opts=pulumi.ResourceOptions(
            depends_on=[
                log_group,
                *(depends_on or []),
            ],
        ),
    )

    # --------------------------------------------------
    # ECS Fargate Service
    # --------------------------------------------------

    service = aws.ecs.Service(
        f"{name}-service",

        cluster=cluster_arn,
        task_definition=task.arn,

        desired_count=1,
        launch_type="FARGATE",

        network_configuration={
            "subnets": subnet_ids,
            "security_groups": [task_security_group_id],

            # Your VPC has public subnets and no NAT gateway
            "assign_public_ip": True,
        },

        load_balancers=[{
            "target_group_arn": target_group_arn,
            "container_name": name,
            "container_port": 9001,
        }],

        health_check_grace_period_seconds=120,

        service_registries=aws.ecs.ServiceServiceRegistriesArgs(
            registry_arn=discovery_service_arn,
        ),

        opts=pulumi.ResourceOptions(
            depends_on=list(depends_on or []),
        ),
    )

    return RustfsService(
        service=service,
        task=task,
        log_group=log_group,
    )
