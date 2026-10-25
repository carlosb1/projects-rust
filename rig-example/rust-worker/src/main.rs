mod ai_agent;

use aws_config::{BehaviorVersion, Region};
use aws_sdk_sqs::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let endpoint = std::env::var("SQS_ENDPOINT")?;
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new("eu-west-1"))
        .endpoint_url(&endpoint)
        .load()
        .await;

    let client = Client::new(&config);

    let queue_url = std::env::var("QUEUE_URL")?;

    println!("Waiting for messages... for queue: {queue_url}");

    loop {
        let response = client
            .receive_message()
            .queue_url(&queue_url)
            .max_number_of_messages(10)
            .wait_time_seconds(20)
            .send()
            .await;

        match response {
            Ok(response) => {
                for message in response.messages() {
                    if let Some(body) = message.body() {
                        println!("Received: {body}");

                        // Aquí procesas el mensaje
                        // process_message(body).await;

                        if let Some(receipt_handle) = message.receipt_handle() {
                            client
                                .delete_message()
                                .queue_url(&queue_url)
                                .receipt_handle(receipt_handle)
                                .send()
                                .await?;
                        }
                    }
                }
            }
            Err(error) => {
                eprintln!("SQS error: {error}");
                tokio::time::sleep(
                    std::time::Duration::from_secs(5)
                ).await;
            }
        }
    }
}