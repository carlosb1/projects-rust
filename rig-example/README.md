## Application database (PostgreSQL)

Compose runs PostgreSQL 17 with a persistent `postgres-data` volume. Add a
URL-safe password (letters and digits are sufficient) to your local `.env`:

```dotenv
POSTGRES_PASSWORD=replace_with_a_long_random_alphanumeric_password
```

Start it with `docker compose up -d postgres`, or start the API and its
dependencies with `docker compose up -d app`. Compose waits for PostgreSQL's
health check before starting the API and supplies `DATABASE_URL` using the
`postgres` hostname. For an API running on your host, set:

```bash
export DATABASE_URL='postgresql://rigadmin:YOUR_PASSWORD@localhost:5432/rig?sslmode=disable'
```

Changing `POSTGRES_PASSWORD` after the volume has been initialized does not
change the database user's password; update it inside PostgreSQL as well.

For AWS, Pulumi provisions PostgreSQL 17 on RDS in two isolated subnets,
with encrypted storage, seven days of backups, and access on port 5432 from
the ECS task security group. The API receives `DATABASE_URL` through an ECS
Secrets Manager reference, with `sslmode=require`.

```bash
pulumi config set --secret dbPassword
# Optional settings (defaults shown):
pulumi config set dbName rig
pulumi config set dbInstanceClass db.t4g.micro
pulumi preview
pulumi up
```

RDS incurs AWS charges. Deletion protection is enabled; removing the database
requires explicitly disabling it in the code and applying that change first.
Deletion creates a final snapshot; its configured name must be unique if a
snapshot from a previous deletion still exists. Pulumi exports
`database_endpoint` and `database_name`, without exporting the credentials.
After changing the secret, redeploy the ECS tasks so they receive its new value.

This provisions the database and connection configuration. The Rust API does
not yet read `DATABASE_URL` or persist records. The next application step is to
add a PostgreSQL client such as SQLx, create a connection pool in `AppState`,
and define migrations for the records the app needs. Qdrant remains the vector
database; PostgreSQL can hold users, report metadata, and durable job status.

RDS configuration reference: [Pulumi RDS Instance](https://www.pulumi.com/registry/packages/aws/api-docs/rds/instance/).

 # AWS Python S3 Bucket Pulumi Template

 A minimal Pulumi template for provisioning a single AWS S3 bucket using Python.

 ## Overview

 This template provisions an S3 bucket (`pulumi_aws.s3.BucketV2`) in your AWS account and exports its ID as an output. It’s an ideal starting point when:
  - You want to learn Pulumi with AWS in Python.
  - You need a barebones S3 bucket deployment to build upon.
  - You prefer a minimal template without extra dependencies.

 ## Prerequisites

 - An AWS account with permissions to create S3 buckets.
 - AWS credentials configured in your environment (for example via AWS CLI or environment variables).
 - Python 3.6 or later installed.
 - Pulumi CLI already installed and logged in.

 ## Getting Started

 1. Generate a new project from this template:
    ```bash
    pulumi new aws-python
    ```
 2. Follow the prompts to set your project name and AWS region (default: `us-east-1`).
 3. Change into your project directory:
    ```bash
    cd <project-name>
    ```
 4. Preview the planned changes:
    ```bash
    pulumi preview
    ```
 5. Deploy the stack:
    ```bash
    pulumi up
    ```
 6. Tear down when finished:
    ```bash
    pulumi destroy
    ```

 ## Project Layout

 After running `pulumi new`, your directory will look like:
 ```
 ├── __main__.py         # Entry point of the Pulumi program
 ├── Pulumi.yaml         # Project metadata and template configuration
 ├── requirements.txt    # Python dependencies
 └── Pulumi.<stack>.yaml # Stack-specific configuration (e.g., Pulumi.dev.yaml)
 ```

 ## Configuration

 This template defines the following config value:

 - `aws:region` (string)
   The AWS region to deploy resources into.
   Default: `us-east-1`

 View or update configuration with:
 ```bash
 pulumi config get aws:region
 pulumi config set aws:region us-west-2
 ```

 ## Outputs

 Once deployed, the stack exports:

 - `bucket_name` — the ID of the created S3 bucket.

 Retrieve outputs with:
 ```bash
 pulumi stack output bucket_name
 ```

 ## Next Steps

 - Customize `__main__.py` to add or configure additional resources.
 - Explore the Pulumi AWS SDK: https://www.pulumi.com/registry/packages/aws/
 - Break your infrastructure into modules for better organization.
 - Integrate into CI/CD pipelines for automated deployments.

 ## Help and Community

 If you have questions or need assistance:
 - Pulumi Documentation: https://www.pulumi.com/docs/
 - Community Slack: https://slack.pulumi.com/
 - GitHub Issues: https://github.com/pulumi/pulumi/issues

 Contributions and feedback are always welcome!

## Create my subnets 
aws ec2 create-default-subnet \
  --availability-zone eu-west-1b \
  --region eu-west-1

aws ec2 create-default-subnet \
  --availability-zone eu-west-1c \
  --region eu-west-1    

## Create my bucket

aws s3api create-bucket \
  --bucket rig-example-pulumi-state \
  --region eu-west-1 \
  --create-bucket-configuration LocationConstraint=eu-west-1

❯ aws s3api put-bucket-versioning \
  --bucket rig-example-pulumi-state \
  --versioning-configuration Status=Enabled

pulumi logout
pulumi login s3://rig-example-pulumi-state
