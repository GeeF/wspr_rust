//! WSPR encoder / decoder
use crate::convcode::convolve;
use crate::interleave::interleave;

static SYNC_VECTOR: [u8; 162] = [
    1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 0, 0, 1, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0,
    0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 1, 0,
    0, 0, 0, 1, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1, 1, 0, 0, 0, 1, 1, 0, 1, 0, 1, 0,
    0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 1, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 0, 1, 1, 1,
    0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 0, 1, 1, 0, 0, 0, 1, 1, 0,
    0, 0,
];

#[derive(Copy, Clone)]
pub enum FrameType {
    Standard,
    Extended,
}

#[derive(Debug, PartialEq)]
pub enum ErrorCode {
    ConvolutionBitSetError, // A "bit" in one of the representational u8s was not 0 or 1
    CallsignEncodeError,    // Callsign does not match format for the selected FrameType
    LocatorEncodeError,     // Locator does not match format for the selected FrameType
    PowerEncodeError,       // Power does not match format for the selected FrameType
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
        // decode symbols => fano metric sequential decoder?
        Self {
            callsign: "".to_string(),
            locator: "".to_string(),
            power: 0,
            frametype: FrameType::Standard,
        }
    }

    /// Get the channel encoded 4-FSK symbols for a standard message
    pub fn encode(&self) -> Result<[u8; 162], ErrorCode> {
        let mut encoded_frame = [0u8; 162];
        // TODO:
        // source encode parameters -> src_frame (pack)
        // src frame for testing...
        let mut src_frame: [u8; 50] = [0; 50];
        src_frame[1] = 1;
        src_frame[2] = 1;
        src_frame[10] = 1;
        src_frame[30] = 1;

        let interleaved_frame = interleave(convolve(src_frame)?);
        for (i, elem) in interleaved_frame.iter().enumerate() {
            encoded_frame[i as usize] = SYNC_VECTOR[i] + 2 * elem;
        }
        Ok(encoded_frame)
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

/// SourceFrame containing the source encoded frame parameters
impl SourceFrame {
    fn new(msg: WSPRMessage, frametype: FrameType) -> Result<Self, ErrorCode> {
        Ok(Self {
            callsign: source_encode_callsign(&msg.callsign, frametype)?,
            locator: source_encode_locator(&msg.locator, frametype)?,
            power: source_encode_power(msg.power, frametype)?,
            frametype: frametype,
        })
    }

    fn packed_src_frame(&self) -> [u8; 50] {
        [0u8; 50]
    }
}

/// shift all elements in the char array to the right. Will cut the right most element if present
fn prepend_space(arr: &mut [char]) {
    for i in 0..5 {
        arr[5-i] = arr[4-i];
    }
    arr[0] = ' ';
}

fn source_encode_callsign(callsign: &str, frametype: FrameType) -> Result<u32, ErrorCode> {
    // callsign regex R"(^[A-Za-z0-9/]+$)" : R"(^[A-Za-z0-9]+$)"
    if callsign.len() < 3 || callsign.len() > 6 {
        return Err(ErrorCode::CallsignEncodeError);
    }
    let mut callsign_arr: [char; 6] = [' '; 6];
    for (n, c) in callsign.chars().enumerate() {
        callsign_arr[n] = c;
    }
    println!("a: {:?}", callsign_arr);
    prepend_space(&mut callsign_arr);
    println!("a shifted: {:?}", callsign_arr);
    // match frametype {
    //     FrameType::Standard => match callsign {
    //         (_, _, _, _, _, _) => println!("std"),
    //     },
    //     FrameType::Extended => println!("ext1"),
    // }
    Ok(0u32)
}

fn source_encode_locator(locator: &str, frametype: FrameType) -> Result<u32, ErrorCode> {
    // validate: https://github.com/roelandjansen/wsjt-x/blob/master/validators/MaidenheadLocatorValidator.cpp
    Ok(0u32)
}

fn source_encode_power(power: u8, frametype: FrameType) -> Result<u8, ErrorCode> {
    Ok(0u8)
}

#[test]
fn test_encode() {
    let msg = WSPRMessage::new("DB2LA", "JO43", 30, FrameType::Standard);
    let channel_encoded = msg.encode().unwrap();
    for i in channel_encoded.iter() {
        print!("{}, ", i);
    }
    println!();
}

#[test]
fn test_decode() {
    let msg = WSPRMessage::decode([0; 162]);
}
