use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};

// https://discord.com/developers/docs/topics/gateway#gateways-gateway-versions
// We disable compression
pub const GATEWAY_URL: &str = "wss://gateway.discord.gg/?v=7&encoding=json";

// https://discord.com/developers/docs/topics/gateway#payloads-gateway-payload-structure
#[derive(Serialize, Deserialize, Debug)]
pub struct GatewayPayload {
    pub op: OpCode,
    #[serde(rename = "d")]
    pub data: Option<Value>,
    #[serde(rename = "s")]
    pub sequence: Option<i32>,
    #[serde(rename = "t")]
    pub event_type: Option<String>,
}

// https://discord.com/developers/docs/topics/opcodes-and-status-codes#voice-voice-opcodes
#[derive(Deserialize_repr, Serialize_repr, Debug)]
#[repr(u8)]
enum VoiceOpCode {
    Identify = 0,
    SelectProtocol = 1,
    Ready = 2,
    Heartbeat = 3,
    SessionDescription = 4,
    Speaking = 5,
    HeartbeatAck = 6,
    Resume = 7,
    Hello = 8,
    Resumed = 9,
    ClientDisconnect = 13,
}

// https://discord.com/developers/docs/topics/opcodes-and-status-codes#gateway-opcodes
#[derive(Deserialize_repr, Serialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum OpCode {
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

// https://discord.com/developers/docs/topics/opcodes-and-status-codes#voice-voice-close-event-codes
#[derive(Deserialize_repr, Serialize_repr)]
#[repr(u16)]
enum VoiceCloseCode {
    UnknownOpcode = 4001,
    NotAuthenticated = 4003,
    AuthenticationFailed = 4004,
    AlreadyAuthenticated = 4005,
    SessionNoLongerValid = 4006,
    SessionTimeout = 4009,
    ServerNotFound = 4011,
    UnknownProtocol = 4012,
    Disconnected = 4014,
    VoiceServerCrashed = 4015,
    UnknownEncryptionMode = 4016,
}
