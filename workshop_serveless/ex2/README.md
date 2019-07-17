bash -x ./run.sh
bash -x ./deploy.sh "arn:aws:iam::886248216134:role/lamba_role_default" test1
bash -x ./setup.sh test1
-> Add trigger, API Gateway to link test1 api, in test devel.
