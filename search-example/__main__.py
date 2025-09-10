"""An AWS Python Pulumi program"""

import pulumi
import pulumi_aws as aws
import pulumi_docker as docker
from dotenv import dotenv_values
from pulumi_docker import RegistryArgs
import base64
import os

from tools import setting_up_infra, get_image_uri_safe

# Get neccessary settings from the pulumi config
config = pulumi.Config()
availability_zone = aws.config.region

# Define the user's home directory dynamically
home_dir = os.path.expanduser("~")

# Ensure .aws directory exists
aws_credentials_path = os.path.join(home_dir, ".aws")
pulumi.info(f"aws credentials path={aws_credentials_path}")

if not os.path.exists(aws_credentials_path):
    pulumi.error(f"The .aws directory is missing in {aws_credentials_path}. Make sure to create it.")
    import sys
    sys.exit(1)

pulumi.info(f"Running the configuration in this region {availability_zone}")

import os

is_push = os.getenv("PUSH", "false")
if is_push == "true":
    url_app = "886248216134.dkr.ecr.eu-west-1.amazonaws.com/search/app"
    url_pyagents = "886248216134.dkr.ecr.eu-west-1.amazonaws.com/search/pyagents"

    app_ecr_repo_app = aws.ecr.get_repository(name='search/app')
    app_ecr_repo_pyagents = aws.ecr.get_repository(name='search/pyagents')

    urls_with_contexts = [
        (url_app, "./app", "app", "./app/Dockerfile", app_ecr_repo_app),
        (url_pyagents,"./pyagents", "pyagents", "./pyagents/Dockerfile", app_ecr_repo_pyagents),
    ]
    # running infra
    setting_up_infra(urls_with_contexts)

else :
    ####################################################################
    app_ecr_repo_app = aws.ecr.get_repository(name='search/app')
    app_ecr_repo_pyagents = aws.ecr.get_repository(name='search/pyagents')

    tag_deploy = 'dev'

    app_image_name = get_image_uri_safe(app_ecr_repo_app, tag_deploy)
    pyagents_image_name = get_image_uri_safe(app_ecr_repo_pyagents, tag_deploy)

    ###########################################

    app_cluster = aws.ecs.Cluster("app-cluster")


    ###########################################
    ############################
    # Network vpc, subnet

    # Creating a VPC and a public subnet
    app_vpc = aws.ec2.Vpc("app-vpc", cidr_block="172.31.0.0/16", enable_dns_hostnames=True)

    # Private dns
    dns_ns = aws.servicediscovery.PrivateDnsNamespace(
        "p2p-ns",
        name="p2p.local",
        vpc=app_vpc.id,
        description="Private DNS for ECS services",
    )

    # Mandatory vpc subnets
    app_vpc_subnet = aws.ec2.Subnet(
        "app-vpc-subnet",
        cidr_block="172.31.0.0/20",
        availability_zone="eu-west-1a",
        vpc_id=app_vpc.id,
    )

    app_vpc_subnet_b = aws.ec2.Subnet(
        "app-vpc-subnet-b",
        cidr_block="172.31.16.0/20",
        availability_zone="eu-west-1b",
        vpc_id=app_vpc.id,
    )

    # Creating a gateway to the web for the VPC
    app_gateway = aws.ec2.InternetGateway("app-gateway", vpc_id=app_vpc.id)

    app_routetable = aws.ec2.RouteTable(
        "app-routetable",
        routes=[
            aws.ec2.RouteTableRouteArgs(
                cidr_block="0.0.0.0/0",
                gateway_id=app_gateway.id,
            )
        ],
        vpc_id=app_vpc.id,
    )
    subnets: list = [app_vpc_subnet.id, app_vpc_subnet_b.id]


    ############################
    ## Security configuration

    # Associating our gateway with our VPC, to allow our app to communicate with the greater internet
    app_routetable_association = aws.ec2.MainRouteTableAssociation(
        "app_routetable_association", route_table_id=app_routetable.id, vpc_id=app_vpc.id
    )

    # Creating a Security Group that restricts incoming traffic to HTTP
    sg_all_open = aws.ec2.SecurityGroup(
        "security-group",
        vpc_id=app_vpc.id,
        description="Enables all access",
        ingress=[
            aws.ec2.SecurityGroupIngressArgs(
                protocol="tcp",
                from_port=0,
                to_port=65535,
                cidr_blocks=["0.0.0.0/0"],
            )
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

    # Creating an IAM role used by Fargate to execute all our services
    app_exec_role = aws.iam.Role(
        "app-exec-role",
        assume_role_policy="""{
            "Version": "2012-10-17",
            "Statement": [
            {
                "Action": "sts:AssumeRole",
                "Principal": {
                    "Service": "ecs-tasks.amazonaws.com"
                },
                "Effect": "Allow",
                "Sid": ""
            }]
        }""",
    )

    security_groups: list = [sg_all_open.id]

    ############################

    ############################

    logs_group_name = "trading-infra-log-group"

    # Creating a Cloudwatch instance to store the logs that the ECS services produce
    log_group = aws.cloudwatch.LogGroup(
        logs_group_name, retention_in_days=1, name=logs_group_name
    )


    ############################

    # Policies for running roles

    # Attaching execution permissions to the exec role
    exec_policy_attachment = aws.iam.RolePolicyAttachment(
        "app-exec-policy",
        role=app_exec_role.name,
        policy_arn=aws.iam.ManagedPolicy.AMAZON_ECS_TASK_EXECUTION_ROLE_POLICY,
    )

    # Creating an IAM role used by Fargate to manage tasks
    app_task_role = aws.iam.Role(
        "app-task-role",
        assume_role_policy="""{
            "Version": "2012-10-17",
            "Statement": [
            {
                "Action": "sts:AssumeRole",
                "Principal": {
                    "Service": "ecs-tasks.amazonaws.com"
                },
                "Effect": "Allow",
                "Sid": ""
            }]
        }""",
    )

    # Attaching execution permissions to the task role
    task_policy_attachment = aws.iam.RolePolicyAttachment(
        "app-access-policy",
        role=app_task_role.name,
        policy_arn=aws.iam.ManagedPolicy.AMAZON_ECS_FULL_ACCESS,
    )

    ############################

    # Setting up private dns service names
    from tools import make_service, make_sd_service, make_alb, make_alb_questdb


    app_sd = make_sd_service(dns_ns, "app")
    pyagents_sd = make_sd_service(dns_ns, "pyagents")
    qdrant_sd = make_sd_service(dns_ns, "qdrant")

    app_hostname = pulumi.Output.concat("app.", dns_ns.name)
    qdrant_hostname = pulumi.Output.concat("qdrant.", dns_ns.name)
    pyagents_hostname = pulumi.Output.concat("pyagents.", dns_ns.name)

    env = dotenv_values(".env")  # dict[str,str|None]

    # WS_SERVER_HOST must stay as an Output
    env['QDRANT_HOST'] = qdrant_hostname.apply(lambda h: str(h))
    env['QDRANT_PORT'] = '6334'
    #ENDPOINT_ZMQ="tcp://ml:5555"

    ecs_env = [
        {"name": k, "value": v}
        for k, v in env.items()
    ]

    name = "qdrant"
    image_name = "qdrant/qdrant:dev-3be4ca880519be040c45baafacd06f4dd4aee080"
    cpu = '1024'
    memory = 2048
    port = 6333

    port_2 = 6334
    port_mappings=[{"containerPort": port, "protocol": "tcp"},{"containerPort": port_2, "protocol": "tcp"} ]

    (alb_questdb, _, load_balancers) = make_alb(name, app_vpc, subnets, security_groups, port, "HTTP")
    (service, definition) = make_service(app_cluster, app_exec_role, app_task_role,
                    name,
                    subnets,
                    security_groups,
                    availability_zone,
                    image_name,
                    cpu,
                    memory,
                    port_mappings,
                    ecs_env,
                    logs_group_name,
                    [],
                    load_balancers,
                    qdrant_sd)

    # setting up backend
    name = "pyagents"
    image_name = pyagents_image_name
    cpu = '256'
    memory = 1024
    port_mappings=[]
    make_service(app_cluster, app_exec_role, app_task_role,
                    name,
                    subnets,
                    security_groups,
                    availability_zone,
                    image_name,
                    cpu,
                    memory,
                    port_mappings,
                    ecs_env,
                    logs_group_name,
                    [],
                    [],
                    sd_service=None)

    name = "app"
    image_name = app_image_name
    cpu = '256'
    memory = 1024
    port = 3000

    port_mappings=[{"containerPort": port, "protocol": "tcp"}]
    (alb, _, load_balancers) = make_alb(name, app_vpc, subnets, security_groups, port, "HTTP")
    make_service(app_cluster, app_exec_role, app_task_role,
                    name,
                    subnets,
                    security_groups,
                    availability_zone,
                    image_name,
                    cpu,
                    memory,
                    port_mappings,
                    ecs_env,
                    logs_group_name,
                    [],
                    load_balancers,
                    app_sd)

    frontend_public_url = pulumi.Output.concat("http://", alb.dns_name, ":3000")
    pulumi.export("frontend_public_url", frontend_public_url)