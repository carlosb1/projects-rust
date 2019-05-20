cargo install cross
cross build --target=x86_64-unknown-linux-musl --release 
cp ./target/x86_64-unknown-linux-musl/release/bootstrap ./bootstrap && zip lambda.zip bootstrap && rm bootstrap
