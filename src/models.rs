use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};

// https://discord.com/developers/docs/topics/gateway#gateways-gateway-versions
// We disable compression
pub const GATEWAY_URL: &str = "wss://gateway.discord.gg/?v=7&encoding=json";

// https://discord.com/developers/docs/topics/gateway#payloads-gateway-payload-structure
#[derive(Serialize, Deserialize, Debug)]
pub struct GatewayPayload {
    op: OpCode,
    d: Option<Value>,
    s: Option<i32>,
    t: Option<String>,
}

// https://discord.com/developers/docs/topics/opcodes-and-status-codes#gateway-opcodes
#[derive(Deserialize_repr, Serialize_repr, Debug)]
#[repr(u8)]
enum OpCode {
    Dispatch = 0,
    Heartbeat = 1,
    Identify = 2,
    PresenceUpdate = 3,
    VoiceStateUpdate = 4,
    Resume = 6,
    Reconnect = 7,
    RequestGuildMembers = 8,
    InvalidSession = 9,
    Hello = 10,
    HeartbeatAck = 11,
}

// https://discord.com/developers/docs/topics/opcodes-and-status-codes#gateway-gateway-close-event-codes
#[derive(Deserialize_repr, Serialize_repr)]
#[repr(u16)]
enum CloseCode {
    UnknownError = 4000,
    UnknownOpcode = 4001,
    DecodeError = 4002,
    NotAuthenticated = 4003,
    AuthenticationFailed = 4004,
    AlreadyAuthenticated = 4005,
    InvalidSeq = 4007,
    RateLimited = 4008,
    SessionTimedOut = 4009,
    InvalidShard = 4010,
    ShardingRequired = 4011,
    InvalidApiVersion = 4012,
    InvalidIntents = 4013,
    DisallowedIntents = 4014,
}
