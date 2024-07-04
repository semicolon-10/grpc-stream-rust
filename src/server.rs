use std::{error::Error, net::ToSocketAddrs, pin::Pin, time::Duration};

use tokio::sync::mpsc;
use tokio_stream::{Stream, StreamExt, wrappers::ReceiverStream};
use tonic::{Request, Response, Status, transport::Server};

use pb::{StreamRequest, StreamResponse};

pub mod pb {
    tonic::include_proto!("stream");
}

type StreamResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item=Result<StreamResponse, Status>> + Send>>;

#[derive(Debug)]
pub struct EchoServer {}

#[tonic::async_trait]
impl pb::stream_server::Stream for EchoServer {
    type ServerStreamStream = ResponseStream;

    async fn server_stream(
        &self,
        req: Request<StreamRequest>,
    ) -> StreamResult<Self::ServerStreamStream> {
        println!("EchoServer::server_streaming_echo");
        println!("\tclient connected from: {:?}", req.remote_addr());

        let repeat = std::iter::repeat(StreamResponse {
            message: "Hello from server".parse().unwrap(),
        });
        let mut stream = Box::pin(tokio_stream::iter(repeat).throttle(Duration::from_millis(200)));

        let (tx, rx) = mpsc::channel(128);
        tokio::spawn(async move {
            while let Some(item) = stream.next().await {
                match tx.send(Result::<_, Status>::Ok(item)).await {
                    Ok(_) => {
                    }
                    Err(_item) => {
                        break;
                    }
                }
            }
            println!("Client disconnected");
        });

        let output_stream = ReceiverStream::new(rx);
        Ok(Response::new(
            Box::pin(output_stream) as Self::ServerStreamStream
        ))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server = EchoServer {};
    println!("Listening...!");
    Server::builder()
        .add_service(pb::stream_server::StreamServer::new(server))
        .serve("[::1]:50051".to_socket_addrs().unwrap().next().unwrap())
        .await
        .unwrap();

    Ok(())
}
