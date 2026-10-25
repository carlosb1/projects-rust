"""PostgreSQL infrastructure and the API's connection secret."""

import json
from urllib.parse import quote

import pulumi
import pulumi_aws as aws


def create_database(vpc_id, subnet_ids, app_security_group_id):
    config = pulumi.Config()
    password = config.require_secret("dbPassword")
    database_name = config.get("dbName") or "rig"
    username = "rigadmin"

    security_group = aws.ec2.SecurityGroup(
        "postgres-sg",
        vpc_id=vpc_id,
        description="PostgreSQL access from application tasks",
        ingress=[aws.ec2.SecurityGroupIngressArgs(
            protocol="tcp", from_port=5432, to_port=5432,
            security_groups=[app_security_group_id],
        )],
    )
    subnet_group = aws.rds.SubnetGroup(
        "postgres-subnets", subnet_ids=subnet_ids,
    )
    instance = aws.rds.Instance(
        "postgres",
        engine="postgres",
        engine_version="17",
        instance_class=config.get("dbInstanceClass") or "db.t4g.micro",
        allocated_storage=20,
        storage_type="gp3",
        storage_encrypted=True,
        db_name=database_name,
        username=username,
        password=password,
        port=5432,
        db_subnet_group_name=subnet_group.name,
        vpc_security_group_ids=[security_group.id],
        publicly_accessible=False,
        backup_retention_period=7,
        auto_minor_version_upgrade=True,
        deletion_protection=True,
        skip_final_snapshot=False,
        final_snapshot_identifier=f"rig-{pulumi.get_stack()}-postgres-final",
    )
    url = pulumi.Output.all(instance.address, password).apply(
        lambda values: (
            f"postgresql://{username}:{quote(values[1], safe='')}"
            f"@{values[0]}:5432/{quote(database_name, safe='')}?sslmode=require"
        )
    )
    secret = aws.secretsmanager.Secret("database-url")
    secret_version = aws.secretsmanager.SecretVersion(
        "database-url-version", secret_id=secret.id, secret_string=url,
    )

    execution_role = aws.iam.Role(
        "api-execution-role",
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
        "api-execution-policy",
        role=execution_role.name,
        policy_arn="arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy",
    )
    secret_policy = aws.iam.RolePolicy(
        "api-database-secret-policy",
        role=execution_role.id,
        policy=secret.arn.apply(lambda arn: json.dumps({
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Action": "secretsmanager:GetSecretValue",
                "Resource": arn,
            }],
        })),
    )
    pulumi.export("database_endpoint", instance.endpoint)
    pulumi.export("database_name", instance.db_name)
    return secret, execution_role, [secret_version, execution_policy, secret_policy]
