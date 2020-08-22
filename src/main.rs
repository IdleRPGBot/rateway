use async_std::task::spawn;
use async_tungstenite::async_std::connect_async;
use futures::StreamExt;
pub mod models;

#[async_std::main]
async fn main() {
    let (ws_stream, _) = connect_async(models::GATEWAY_URL)
        .await
        .expect("Failed to connect to the Discord gateway");

    let (_write, read) = ws_stream.split();

    let ws_to_stdout = {
        read.for_each(|message| async {
            let message = message.unwrap().into_data();
            let decoded: models::GatewayPayload =
                serde_json::from_slice(&message).expect("Cannot decode JSON");
            println!("{:?}", decoded);
        })
    };

    spawn(ws_to_stdout).await;
}
