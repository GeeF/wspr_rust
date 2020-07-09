//! WSPR encoder / decoder

static SYNC_VECTOR: [u8; 162] = [
    1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 0, 0, 1, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0,
    0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 1, 0,
    0, 0, 0, 1, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1, 1, 0, 0, 0, 1, 1, 0, 1, 0, 1, 0,
    0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 1, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 0, 1, 1, 1,
    0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 0, 1, 1, 0, 0, 0, 1, 1, 0,
    0, 0,
];

pub enum FrameType {
    Standard,
    Extended,
}

pub struct WSPRMessage {
    pub callsign: String,
    pub locator: String,
    pub power: u8,
    pub frametype: FrameType,
}

impl WSPRMessage {
    pub fn new(callsign: &str, locator: &str, power: u8, frametype: FrameType) -> Self {
        Self {
            callsign: callsign.to_string(),
            locator: locator.to_string(),
            power: power,
            frametype: frametype,
        }
    }

    pub fn decode(symbols: [u8; 162]) -> Self {
        // decode symbols
        Self {
            callsign: "".to_string(),
            locator: "".to_string(),
            power: 0,
            frametype: FrameType::Standard,
        }
    }

    /// Get the channel encoded 4-FSK symbols for a standard message
    pub fn encode(&self) -> [u8; 162] {
        // TODO:
        // source encode parameters -> src_frame
        // convolve(src_frame)
        // interleave(src_frame) -> symbols
        [0; 162]
    }

    /// Get the channel encoded 4-FSK symbols for an extended message
    pub fn encode_extended(&self) -> ([u8; 162], [u8; 162]) {
        ([0; 162], [0; 162])
    }
}

/// Source encoded WSPR frame
struct SourceFrame {
    callsign: u32,
    locator: u32,
    power: u8,
    frametype: FrameType,
}

fn source_encode_callsign(callsign: &str) -> u32 {
    0
}

fn source_encode_locator(locator: &str) -> u32 {
    0
}

fn source_encode_power(power: u8) -> u8 {
    power
}

/// SourceFrame containing the source encoded frame parameters
impl SourceFrame {
    fn new(msg: WSPRMessage, frametype: FrameType) -> Self {
        Self {
            frametype: frametype,
            callsign: source_encode_callsign(&msg.callsign),
            locator: source_encode_locator(&msg.locator),
            power: source_encode_power(msg.power),
        }
    }
}

#[test]
fn test_encode() {
    let msg = WSPRMessage::new("DB2LA", "JO43", 30, FrameType::Standard);
    let channel_encoded = msg.encode();
}

#[test]
fn test_decode() {
    let msg = WSPRMessage::decode([0; 162]);
}
