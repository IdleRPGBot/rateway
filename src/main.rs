mod models;
mod websocket;

#[async_std::main]
async fn main() {
    let mut can_connect = true;
    while can_connect {
        println!("Connecting!");
        let mut ws = websocket::WebsocketClient::connect().await;
        can_connect = ws.read_task().await;
    }
    println!("Cannot reconnect");
}
