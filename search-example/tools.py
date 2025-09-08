from pathlib import Path
from typing import Dict, Any, Tuple

import pulumi
import pulumi_aws as aws

def env_list(d: Dict[str, Any]) -> list[dict]:
    return [{"name": k, "value": v} for k, v in d.items()]

#setting dns
def make_sd_service(dns_ns, name: str):
    return aws.servicediscovery.Service(
        f"{name}-sd",
        name=name,  # el hostname será "<name>.p2p.local"
        dns_config=aws.servicediscovery.ServiceDnsConfigArgs(
            namespace_id=dns_ns.id,
            dns_records=[
                aws.servicediscovery.ServiceDnsConfigDnsRecordArgs(
                    ttl=5,
                    type="A",  # Fargate -> A records
                )
            ],
            routing_policy="MULTIVALUE",
        ),
        health_check_custom_config=aws.servicediscovery.ServiceHealthCheckCustomConfigArgs(
            failure_threshold=1,
        ),
    )


############################ Tracker task - service ############################
def make_service(
                app_cluster,
                app_exec_role,
                app_task_role,
                name: str,
                id_subnets: list[str],
                id_security_groups: list[str],
                availability_zone: str,
                image_name: str,
                cpu: str,
                memory: int,
                port_mappings: list,
                env_vars: list,
                logs_group: str,
                depends_on: list[Any],
                load_balancers: list[aws.ecs.ServiceLoadBalancerArgs],
                sd_service: aws.servicediscovery.Service | None = None,
                volumes_info: Tuple[list, list[str]] = ([],[])
            ):
    # Creating a task definition for the second Django instance. This instance will
    # act as the server, and will run indefinately
    volumes= volumes_info[0]
    mount_points = volumes_info[1]


    service_registries = None
    if sd_service is not None:
        service_registries = aws.ecs.ServiceServiceRegistriesArgs(
            registry_arn=sd_service.arn
            # opcional pero recomendable si tienes varios puertos:
            # port=port_mappings[0]["containerPort"],
        )

    task = aws.ecs.TaskDefinition(
        f"{name}-task-definition",
        family=f"{name}-task-definition-family",
        cpu=str(cpu),
        memory=str(memory),
        network_mode="awsvpc",
        requires_compatibilities=["FARGATE"],
        execution_role_arn=app_exec_role.arn,
        task_role_arn=app_task_role.arn,
        container_definitions=pulumi.Output.json_dumps(
            [
                {
                    "name": f"{name}-container",
                    "image": image_name,
                    "memory": memory,
                    "essential": True,
                    "portMappings": port_mappings,
                    "environment": env_vars,
                    "logConfiguration": {
                        "logDriver": "awslogs",
                        "options": {
                            "awslogs-group": logs_group,
                            "awslogs-region": availability_zone,
                            "awslogs-stream-prefix": f"trading-infra-{name}", #TODO fixing namespace as param?
                        },
                    },
                    "mountPoints": mount_points,
                    #   "command": [],
                }
            ]
        ),
        volumes=volumes,
    )

    # Launching our tracker server service on Fargate, using our configurations and load balancers
    svc = aws.ecs.Service(
        f"{name}-service",
        force_new_deployment=True,
        cluster=app_cluster.arn,
        desired_count=1,
        launch_type="FARGATE",
        task_definition=task.arn,
        wait_for_steady_state=False,
        network_configuration=aws.ecs.ServiceNetworkConfigurationArgs(
            assign_public_ip=True,
            subnets=id_subnets, # TODO CHECK THIS TO BE INCLUDED IN OUR VPC
            security_groups=id_security_groups,
        ),
        load_balancers=load_balancers,
        service_registries=service_registries,
        opts=pulumi.ResourceOptions(depends_on=depends_on),
    )
    return svc, task

def make_alb(name, app_vpc, subnets, security_groups, port, protocol):
    alb = aws.lb.LoadBalancer(
        f"{name}-alb",
        load_balancer_type="application",
        security_groups=security_groups,  # tu SG único abierto
        subnets=subnets  # 2 AZs mínimo
    )

    backend_tg = aws.lb.TargetGroup(
        f"{name}-tg",
        port=port, protocol=protocol, target_type="ip", vpc_id=app_vpc.id,
        health_check=aws.lb.TargetGroupHealthCheckArgs(protocol="HTTP", path="/", port="traffic-port"),
    )

    listener = aws.lb.Listener(
        f"{name}-listener",
        load_balancer_arn=alb.arn,
        port=port, protocol=protocol,
        default_actions=[aws.lb.ListenerDefaultActionArgs(type="forward", target_group_arn=backend_tg.arn)],
    )

    load_balancers = [aws.ecs.ServiceLoadBalancerArgs(
        target_group_arn=backend_tg.arn,
        container_name=f"{name}-container",
        container_port=port,
    )]
    return (alb, listener, load_balancers)


def make_alb_questdb(name, app_vpc, subnets, security_groups, port, protocol):
    alb = aws.lb.LoadBalancer(
        f"{name}-alb",
        load_balancer_type="application",
        security_groups=security_groups,  # tu SG único abierto
        subnets=subnets  # 2 AZs mínimo
    )

    backend_tg = aws.lb.TargetGroup(
        f"{name}-tg",
        port=port, protocol=protocol, target_type="ip", vpc_id=app_vpc.id,
        health_check=aws.lb.TargetGroupHealthCheckArgs(
            enabled=True,
            protocol="HTTP",
            path="/",
            matcher="200-399",
            interval=15,
            timeout=5,
            healthy_threshold=3,
            unhealthy_threshold=3,
            port="9003",              # <<< override: healthcheck va al 9003
        ),
    )

    listener = aws.lb.Listener(
        f"{name}-listener",
        load_balancer_arn=alb.arn,
        port=port, protocol=protocol,
        default_actions=[aws.lb.ListenerDefaultActionArgs(type="forward", target_group_arn=backend_tg.arn)],
    )

    load_balancers = [aws.ecs.ServiceLoadBalancerArgs(
        target_group_arn=backend_tg.arn,
        container_name=f"{name}-container",
        container_port=port,
    )]
    return (alb, listener, load_balancers)


############################ Tracker task - service ############################
def make_service_questdb(
                app_cluster,
                app_exec_role,
                app_task_role,
                name: str,
                id_subnets: list[str],
                id_security_groups: list[str],
                availability_zone: str,
                image_name: str,
                cpu: str,
                memory: int,
                port_mappings: list,
                env_vars: list,
                logs_group: str,
                depends_on: list[Any],
                load_balancers: list[aws.ecs.ServiceLoadBalancerArgs],
                sd_service: aws.servicediscovery.Service | None = None,
                volumes_info: Tuple[list, list[str]] = ([],[])
            ):
    # Creating a task definition for the second Django instance. This instance will
    # act as the server, and will run indefinately
    volumes= volumes_info[0]
    mount_points = volumes_info[1]


    service_registries = None
    if sd_service is not None:
        service_registries = aws.ecs.ServiceServiceRegistriesArgs(
            registry_arn=sd_service.arn
            # opcional pero recomendable si tienes varios puertos:
            # port=port_mappings[0]["containerPort"],
        )

    task = aws.ecs.TaskDefinition(
        f"{name}-task-definition",
        family=f"{name}-task-definition-family",
        cpu=str(cpu),
        memory=str(memory),
        network_mode="awsvpc",
        requires_compatibilities=["FARGATE"],
        execution_role_arn=app_exec_role.arn,
        task_role_arn=app_task_role.arn,
        container_definitions=pulumi.Output.json_dumps(
            [
                {
                    "name": f"{name}-container",
                    "image": image_name,
                    "memory": memory,
                    "essential": True,
                    "portMappings": port_mappings,
                    "environment": env_vars,
                    "logConfiguration": {
                        "logDriver": "awslogs",
                        "options": {
                            "awslogs-group": logs_group,
                            "awslogs-region": availability_zone,
                            "awslogs-stream-prefix": f"trading-infra-{name}", #TODO fixing namespace as param?
                        },
                    },
                    "mountPoints": mount_points,
                    "ulimits": [{"name": "nofile", "softLimit": 1048576, "hardLimit": 1048576}],
                    #   "command": [],
                }
            ]
        ),
        volumes=volumes,
    )

    # Launching our tracker server service on Fargate, using our configurations and load balancers
    svc = aws.ecs.Service(
        f"{name}-service",
        force_new_deployment=True,
        cluster=app_cluster.arn,
        desired_count=1,
        launch_type="FARGATE",
        task_definition=task.arn,
        wait_for_steady_state=False,
        network_configuration=aws.ecs.ServiceNetworkConfigurationArgs(
            assign_public_ip=True,
            subnets=id_subnets, # TODO CHECK THIS TO BE INCLUDED IN OUR VPC
            security_groups=id_security_groups,
        ),
        load_balancers=load_balancers,
        health_check_grace_period_seconds=300,
        service_registries=service_registries,
        opts=pulumi.ResourceOptions(depends_on=depends_on),
    )
    return svc, task