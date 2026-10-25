import pulumi
import pulumi_aws as aws
import pulumi_awsx as awsx


def create_worker(
    name,
    cluster,
    vpc,
    security_group,
    image_uri,
    task_role,
    queue_url,
    storage_endpoint,
    bucket_name,
    qdrant_url,
    region,
    database_url_secret_arn,
    access_key_secret_arn,
    secret_key_secret_arn,
    execution_role_arn,
    depends_on=None,
):
    return awsx.ecs.FargateService(
        name,
        cluster=cluster.arn,
        desired_count=1,
        network_configuration=aws.ecs.ServiceNetworkConfigurationArgs(
            subnets=vpc.public_subnet_ids,
            assign_public_ip=True,
            security_groups=[security_group.id],
        ),
        task_definition_args=awsx.ecs.FargateServiceTaskDefinitionArgs(
            cpu="256",
            memory="512",
            execution_role={
                "role_arn": execution_role_arn,
            },
            task_role={
                "role_arn": task_role.arn,
            },
            containers={
                name: awsx.ecs.TaskDefinitionContainerDefinitionArgs(
                    name=name,
                    image=image_uri,
                    essential=True,
                    environment=[
                        awsx.ecs.TaskDefinitionKeyValuePairArgs(
                            name="QUEUE_URL",
                            value=queue_url,
                        ),
                        awsx.ecs.TaskDefinitionKeyValuePairArgs(
                            name="S3_ENDPOINT",
                            value=storage_endpoint,
                        ),
                        awsx.ecs.TaskDefinitionKeyValuePairArgs(
                            name="S3_BUCKET",
                            value=bucket_name,
                        ),
                        awsx.ecs.TaskDefinitionKeyValuePairArgs(
                            name="AWS_REGION",
                            value=region,
                        ),
                        awsx.ecs.TaskDefinitionKeyValuePairArgs(
                            name="AWS_DEFAULT_REGION",
                            value=region,
                        ),
                        awsx.ecs.TaskDefinitionKeyValuePairArgs(
                            name="QDRANT_URL",
                            value=qdrant_url,
                        ),
                    ],
                    secrets=[
                        awsx.ecs.TaskDefinitionSecretArgs(
                            name="DATABASE_URL",
                            value_from=database_url_secret_arn,
                        ),
                        awsx.ecs.TaskDefinitionSecretArgs(
                            name="RUSTFS_ACCESS_KEY",
                            value_from=access_key_secret_arn,
                        ),
                        awsx.ecs.TaskDefinitionSecretArgs(
                            name="RUSTFS_SECRET_KEY",
                            value_from=secret_key_secret_arn,
                        ),
                    ],
                ),
            },
        ),
        opts=pulumi.ResourceOptions(
            depends_on=depends_on or [],
        ),
    )