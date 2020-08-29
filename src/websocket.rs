use crate::models::{GatewayPayload, OpCode, GATEWAY_URL};
use async_native_tls::TlsStream;
use async_std::{
    future::timeout,
    net::TcpStream,
    task::{sleep, spawn, JoinHandle},
};
use async_tungstenite::{
    async_std::connect_async, stream::Stream, tungstenite::Message, WebSocketStream,
};
use futures::{SinkExt, StreamExt};
use serde_json::{json, to_vec};
use std::time::Duration;

type WsStream = WebSocketStream<Stream<TcpStream, TlsStream<TcpStream>>>;

pub struct WebsocketClient {
    heartbeat_interval: u64,
    sequence: u64,
    stream: WsStream,
    dead: bool,
    heartbeat_task: Option<JoinHandle<()>>,
}

impl WebsocketClient {
    /// Connects to the gateway
    pub async fn connect() -> WebsocketClient {
        let (ws_stream, _) = connect_async(GATEWAY_URL)
            .await
            .expect("Failed to connect to the Discord gateway");
        WebsocketClient {
            heartbeat_interval: 0,
            sequence: 0,
            stream: ws_stream,
            dead: false,
            heartbeat_task: None,
        }
    }

    async fn recv_json(&mut self) -> Option<Message> {
        match timeout(std::time::Duration::from_millis(500), self.stream.next()).await {
            Ok(v) => v.map(|v| v.ok()).flatten(),
            Err(_) => None,
        }
    }

    /// Sends a heartbeat package
    async fn send_heartbeat(&mut self) {
        while !self.dead {
            let payload = GatewayPayload {
                op: OpCode::Heartbeat,
                data: Some(json!(self.sequence)),
                event_type: None,
                sequence: None,
            };
            let encoded = to_vec(&payload).unwrap();
            self.stream
                .send(Message::binary(encoded))
                .await
                .expect("Cannot send message");
            sleep(Duration::from_millis(self.heartbeat_interval)).await;
        }
    }

    fn dispatch(&mut self, payload: GatewayPayload) {
        match payload.op {
            OpCode::Hello => {
                // The gateway welcomes us for the first time
                // Now parse the heartbeat interval
                let heartbeat_interval = payload
                    .data
                    .unwrap()
                    .get("heartbeat_interval")
                    .expect("No heartbeat interval found")
                    .as_u64()
                    .unwrap();
                self.heartbeat_interval = heartbeat_interval;

                if let Some(old_task) = self.heartbeat_task {
                    old_task.cancel();
                }

                self.heartbeat_task = Some(spawn(self.send_heartbeat()));
            }
        }
    }

    pub async fn read_task(&mut self) -> bool {
        while !self.dead {
            let message = self.recv_json().await;
            if let Some(msg) = message {
                match msg {
                    Message::Binary(data) => {
                        let decoded: GatewayPayload =
                            serde_json::from_slice(&data).expect("Invalid JSON");
                        self.dispatch(decoded);
                    }
                    Message::Text(text) => {
                        let decoded: GatewayPayload =
                            serde_json::from_str(&text).expect("Invalid JSON");
                        self.dispatch(decoded);
                    }
                    Message::Close(opt_frame) => {
                        if let Some(frame) = opt_frame {
                            let allowed = frame.code.is_allowed();
                            return allowed;
                        }
                    }
                }
            }
        }
        false
    }
}
