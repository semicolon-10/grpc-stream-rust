

use std::time::Duration;

use tokio_stream::{Stream, StreamExt};
use tonic::transport::Channel;

use crate::pb::stream_client::StreamClient;
use crate::pb::StreamRequest;

pub mod pb {
    tonic::include_proto!("stream");
}

async fn streaming_echo(client: &mut StreamClient<Channel>, num: usize) {
    let stream = client
        .server_stream(StreamRequest {
            message: "Hi".into(),
        })
        .await
        .unwrap()
        .into_inner();

    let mut stream = stream.take(num);
    while let Some(item) = stream.next().await {
        println!("\treceived: {}", item.unwrap().message);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = StreamClient::connect("http://[::1]:50051").await.unwrap();

    println!("Streaming echo:");
    streaming_echo(&mut client, 5).await;
    tokio::time::sleep(Duration::from_secs(1)).await;


    Ok(())
}
