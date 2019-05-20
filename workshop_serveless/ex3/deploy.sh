aws lambda create-function --function-name "$2" \
  --handler doesnt.matter \
  --zip-file fileb://./lambda.zip \
  --runtime provided \
  --role "$1" --environment Variables={RUST_BACKTRACE=1} --tracing-config Mode=Active

