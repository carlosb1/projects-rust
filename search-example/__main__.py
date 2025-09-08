"""An AWS Python Pulumi program"""
import base64

import os
import pulumi
import pulumi_aws as aws
import pulumi_docker as docker
from pulumi_docker import RegistryArgs


# Repo functions

def set_up_policy(name, app_ecr_repo):
    return aws.ecr.LifecyclePolicy(
        f"app-lifecycle-policy-{name}",
        repository=app_ecr_repo.name,
        policy="""{
            "rules": [
                {
                    "rulePriority": 10,
                    "description": "Remove untagged images",
                    "selection": {
                        "tagStatus": "untagged",
                        "countType": "imageCountMoreThan",
                        "countNumber": 1
                    },
                    "action": {
                        "type": "expire"
                    }
                }
            ]
        }""",
    )


def get_registry_info(rid):
    creds = aws.ecr.get_credentials(registry_id=rid)
    decoded = base64.b64decode(creds.authorization_token).decode()
    parts = decoded.split(':')
    if len(parts) != 2:
        raise Exception("Invalid credentials")
    return RegistryArgs(
        server=creds.proxy_endpoint,
        username=parts[0],
        password=parts[1],
    )


def new_docker_image(url, tag, context, name, dockerfile, app_registry):
    url_version = f"{url}:{tag}"
    app_image = docker.Image(
        f"{name}-{tag}",
        image_name=url_version,
        build=docker.DockerBuildArgs(
            context=context,
            platform='linux/amd64',
            dockerfile=dockerfile
        ),
        skip_push=False,
        registry=app_registry
    )
    return app_image



def setting_up_infra(urls_with_contexts):
    for (_, _, name, _, app_ecr_repo) in urls_with_contexts:
        print(f"Creating policy for {name}")
        set_up_policy(name, app_ecr_repo)

    for (_, _, name, _, app_ecr_repo) in urls_with_contexts:
        print(f"Creating {app_ecr_repo.registry_id}")

    app_images = []
    for (url, context, name, dockerfile, app_ecr_repo) in urls_with_contexts:
        app_registry = get_registry_info(app_ecr_repo.registry_id)
        for tag in tags:
            print(f"{url}:{tag} with context={context}")
            app_image = new_docker_image(url, tag, context, name, dockerfile, app_registry)
            app_images.append(app_image.repo_digest)

    for app_image in app_images:
        pulumi.export("app image:", app_image)


# setting up configuration

# Get neccessary settings from the pulumi config
config = pulumi.Config()
availability_zone = aws.config.region

hash_tag = os.getenv('GITHUB_SHA', 'unknown')
tags = ['latest', 'dev']

if hash_tag != 'unknown':
    tags.append(hash_tag)
# TODO Add 80 port mapping

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
pulumi.info(f'Possible tags = {tags}')


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
