#!/bin/sh

API_NAME=$1
REGION=us-east-1
STAGE=test

function fail() {
    echo $2
    exit $1
}

#awslocal lambda create-function \
#    --region ${REGION} \
#    --function-name ${API_NAME} \
#    --runtime nodejs8.10 \
#    --handler lambda.apiHandler \
#    --memory-size 128 \
#    --zip-file fileb://api-handler.zip \
#    --role arn:aws:iam::123456:role/irrelevant
#
#[ $? == 0 ] || fail 1 "Failed: AWS / lambda / create-function"

LAMBDA_ARN=$(aws lambda list-functions --query "Functions[?FunctionName==\`${API_NAME}\`].FunctionArn" --output text --region ${REGION})

aws apigateway create-rest-api \
    --region ${REGION} \
    --name ${API_NAME}

[ $? == 0 ] || fail 2 "Failed: AWS / apigateway / create-rest-api"

API_ID=$(aws apigateway get-rest-apis --query "items[?name==\`${API_NAME}\`].id" --output text --region ${REGION})
PARENT_RESOURCE_ID=$(aws apigateway get-resources --rest-api-id ${API_ID} --query 'items[?path==`/`].id' --output text --region ${REGION})

aws apigateway create-resource \
    --region ${REGION} \
    --rest-api-id ${API_ID} \
    --parent-id ${PARENT_RESOURCE_ID} \
    --path-part "{somethingId}"

[ $? == 0 ] || fail 3 "Failed: AWS / apigateway / create-resource"

RESOURCE_ID=$(aws apigateway get-resources --rest-api-id ${API_ID} --query 'items[?path==`/{somethingId}`].id' --output text --region ${REGION})

aws apigateway put-method \
    --region ${REGION} \
    --rest-api-id ${API_ID} \
    --resource-id ${RESOURCE_ID} \
    --http-method ANY \
    --request-parameters "method.request.path.somethingId=true" \
    --authorization-type "NONE" \

[ $? == 0 ] || fail 4 "Failed: AWS / apigateway / put-method"

aws apigateway put-integration \
    --region ${REGION} \
    --rest-api-id ${API_ID} \
    --resource-id ${RESOURCE_ID} \
    --http-method ANY \
    --type AWS_PROXY \
    --integration-http-method POST \
    --uri arn:aws:apigateway:${REGION}:lambda:path/2015-03-31/functions/${LAMBDA_ARN}/invocations \
    --passthrough-behavior WHEN_NO_MATCH \

[ $? == 0 ] || fail 5 "Failed: AWS / apigateway / put-integration"


aws apigateway put-method-response \
--rest-api-id ${API_ID} \
--resource-id ${RESOURCE_ID} \
--http-method ANY \
--status-code 200 \
--region ${REGION} \

aws apigateway put-integration-response \
--rest-api-id ${API_ID} \
--resource-id ${RESOURCE_ID} \
--http-method ANY \
--status-code 200 \
--selection-pattern ".*" --region ${REGION} \


aws apigateway create-deployment \
    --region ${REGION} \
    --rest-api-id ${API_ID} \
    --stage-name ${STAGE} \

[ $? == 0 ] || fail 6 "Failed: AWS / apigateway / create-deployment"


#APIARN=$(echo ${LAMBDA_ARN} | sed -e 's/lambda/execute-api/' -e "s/function:${NAME}/${API_ID}/")
#echo "Updating Lambda to be executable by API..."
#aws lambda add-permission \
#	--function-name "$NAME" \
#	--statement-id "api-$REST_API_ID-$RESOURCE_ID" \
#	--action lambda:InvokeFunction \
#	--principal apigateway.amazonaws.com \
#    --source-arn "${APIARN}/*/*/${API_NAME}" \
#    --region ${REGION} \


ENDPOINT=https://${API_ID}.execute-api.${REGION}.amazonaws.com/${STAGE}/${API_NAME}

echo "API available at: ${ENDPOINT}"

echo "Testing GET:"
curl -i -H "Accept: application/json" -H "Content-Type: application/json" -X GET $ENDPOINT

echo "Testing POST:"
curl -i -H "Accept: application/json" -H "Content-Type: application/json" -X POST $ENDPOINT -d '{"username": "new-user", "email": "new@mail.com"}'

