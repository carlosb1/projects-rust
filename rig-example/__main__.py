"""An AWS Python Pulumi program"""
import pulumi
import pulumi_aws as aws
import pulumi_awsx as awsx
import os
from enum import Enum

from pulumi_aws.ecr import Repository


class Operation(Enum):
    Push = "Push"
    Up = "Up"

def build_and_push_ecr_image(repository: Repository):
    app_image = awsx.ecr.Image(
        "rig-example",
        repository_url=repository.url,
        context="./app",
        platform="linux/amd64",
    )

    return app_image


def up_infra(app_image_uri, container_port: int):

    # ---------------------------------------------------------
    # ECS Cluster
    # ---------------------------------------------------------
    vpc = awsx.ec2.Vpc(
        "rig-example-vpc",
        number_of_availability_zones=2,
        subnet_specs=[
            awsx.ec2.SubnetSpecArgs(
                type=awsx.ec2.SubnetType.PUBLIC,
            )
        ],
    )

    cluster = aws.ecs.Cluster(
        "llm-cluster",
    )

    lb = awsx.lb.ApplicationLoadBalancer(
        "rig-example-lb",
        subnet_ids=vpc.public_subnet_ids,
        default_target_group_port=container_port,
    )

    service = awsx.ecs.FargateService(
        "rig-example-service",
        cluster=cluster.arn,

        network_configuration=aws.ecs.ServiceNetworkConfigurationArgs(
            subnets=vpc.public_subnet_ids,
        ),

        assign_public_ip=True,

        task_definition_args=awsx.ecs.FargateServiceTaskDefinitionArgs(
            container=awsx.ecs.TaskDefinitionContainerDefinitionArgs(
                name="rig-example",
                image=app_image_uri,
                cpu=256,
                memory=512,

                port_mappings=[
                    awsx.ecs.TaskDefinitionPortMappingArgs(
                        container_port=container_port,
                        target_group=lb.default_target_group,
                    )
                ],
            )
        ),
    )


def main(container_port: int, operation: Operation) -> None:
    # Create an AWS resource (S3 Bucket)
    #bucket = s3.Bucket('my-bucket')
    # Export the name of the bucket
    #pulumi.export('bucket_name', bucket.id)

    # Get neccessary settings from the pulumi config

    availability_zone = aws.config.region
    # Define the user's home directory dynamically
    home_dir = os.path.expanduser("~")
    pulumi.info(f"availability zone={availability_zone} home dir={home_dir}")


    # Ensure .aws directory exists
    aws_credentials_path = os.path.join(home_dir, ".aws")
    pulumi.info(f"aws credentials path={aws_credentials_path}")

    if not os.path.exists(aws_credentials_path):
        pulumi.error(f"The .aws directory is missing in {aws_credentials_path}. Make sure to create it.")
        import sys
        sys.exit(1)

    # ---------------------------------------------------------
    # ECR
    # ---------------------------------------------------------

    repository = awsx.ecr.Repository(
        "rig-example-repository",
        force_delete=True,
    )
    image_uri = pulumi.Output.concat(
        repository.url,
        ":latest",
    )
#    if operation == Operation.Push:
#        pulumi.info(f"Push operation on {repository}")
    app_image = build_and_push_ecr_image(repository)
    image_uri = app_image.image_uri

    image_uri.apply(
        lambda uri: pulumi.info(f"Image URI={uri}")
    )

    up_infra(image_uri, container_port)

if __name__ == "__main__":
    config = pulumi.Config()
    operation = Operation(config.get("operation") or "Up")

    container_port = 3000
    main(container_port, Operation.Push)