rm -rf target
rm -rf lambda.zip
aws lambda delete-function --function-name $1
