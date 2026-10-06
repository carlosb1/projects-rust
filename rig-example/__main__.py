"""An AWS Python Pulumi program."""

import os
import sys

import pulumi
import pulumi_aws as aws
import pulumi_awsx as awsx


CONTAINER_PORT = 3000
QDRANT_PORT = 6333
QDRANT_GRPC_PORT = 6334


def build_and_push_ecr_image(repository):
    return awsx.ecr.Image(
        "rig-example",
        repository_url=repository.url,
        context="./app",
        platform="linux/amd64",
    )


def up_infra(app_image_uri, container_port: int):

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
    # Internet -> ALB :80
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
    # ALB -> Fargate :3000
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

    # ---------------------------------------------------------
    # Application Load Balancer
    # ---------------------------------------------------------

    lb = awsx.lb.ApplicationLoadBalancer(
        "rig-example-lb",
        subnet_ids=vpc.public_subnet_ids,
        security_groups=[alb_sg.id],

        default_target_group={
            "port": container_port,
            "protocol": "HTTP",
            "target_type": "ip",
            "health_check": {
                "path": "/health",
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
                            )
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

    # ---------------------------------------------------------
    # ECR
    # ---------------------------------------------------------

    repository = awsx.ecr.Repository(
        "rig-example-repository",
        force_delete=True,
    )

    # Builds ./app Dockerfile and pushes the image to ECR.
    app_image = build_and_push_ecr_image(repository)

    app_image.image_uri.apply(
        lambda uri: pulumi.info(
            f"Image URI={uri}"
        )
    )

    # ---------------------------------------------------------
    # Infrastructure
    # ---------------------------------------------------------

    up_infra(
        app_image.image_uri,
        CONTAINER_PORT,
    )


if __name__ == "__main__":
    main()
