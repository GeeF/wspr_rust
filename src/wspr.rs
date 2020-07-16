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
}

impl WSPRMessage {
    pub fn new(callsign: &str, locator: &str, power: u8) -> Self {
        Self {
            callsign: callsign.to_string(),
            locator: locator.to_string(),
            power,
        }
    }

    pub fn decode(symbols: [u8; 162]) -> Self {
        // decode symbols => fano metric sequential decoder?
        Self {
            callsign: "".to_string(),
            locator: "".to_string(),
            power: 0,
        }
    }

    /// Get the channel encoded 4-FSK symbols for a standard message
    pub fn encode(&self) -> Result<[u8; 162], ErrorCode> {
        let mut encoded_frame = [0u8; 162];
        let s = SourceFrame::new(self)?;
        let interleaved_frame = interleave(convolve(s.packed_src_frame())?);
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
}

/// SourceFrame containing the source encoded frame parameters
impl SourceFrame {
    fn new(msg: &WSPRMessage) -> Result<Self, ErrorCode> {
        Ok(Self {
            callsign: source_encode_callsign(&msg.callsign)?,
            locator: source_encode_locator(&msg.locator)?,
            power: source_encode_power(msg.power)?,
        })
    }

    fn packed_src_frame(&self) -> [u8; 50] {
        // -> struct member? calculate in new? what about extended then?
        let _encoded = self.callsign + self.locator + self.power as u32;
        [0u8; 50]
    }
}

/// shift all elements in the char array to the right. Will cut the right most element if present
fn prepend_space(arr: &mut [char]) {
    for i in 0..5 {
        arr[5 - i] = arr[4 - i];
    }
    arr[0] = ' ';
}

/// Encode a single character according to the WSPR spec
/// Illegal characters should be checked before this
fn encode_char(c: char) -> u8 {
    match c {
        '0'..='9' => c as u8 - 48, // '0'-'9' as 0-9
        'A'..='Z' => c as u8 - 55, // 'A'-'Z' as 10-35
        ' ' => 36,                 // space is 36
        _ => 0                     // illegal char: 0
    }
}

/// tbw: steps
fn source_encode_callsign(callsign: &str) -> Result<u32, ErrorCode> {
    if callsign.len() < 3 || callsign.len() > 6 {
        return Err(ErrorCode::CallsignEncodeError);
    }

    let mut callsign_arr: [char; 6] = [' '; 6];
    for (n, c) in callsign.to_uppercase().chars().enumerate() {
        callsign_arr[n] = c;
    }

    // position 3 needs to be a number, shift shorter callsigns to the right
    match callsign_arr[2] {
        '0'..='9' => (),
        _ => prepend_space(&mut callsign_arr),
    }

    match (callsign_arr[0], callsign_arr[1], callsign_arr[2]) {
        (' ', 'A'..='Z', '0'..='9') | ('A'..='Z', 'A'..='Z', '0'..='9') => (),
        _ => return Err(ErrorCode::CallsignEncodeError),
    }
    
    // encode characters, packed to 28 bits maximum
    let mut encoded_callsign = encode_char(callsign_arr[0]) as u32;
    encoded_callsign = encode_char(callsign_arr[1]) as u32 + encoded_callsign * 36;
    encoded_callsign = encode_char(callsign_arr[2]) as u32 + encoded_callsign * 10;
    encoded_callsign = encode_char(callsign_arr[3]) as u32 + encoded_callsign * 27;
    encoded_callsign = encode_char(callsign_arr[4]) as u32 + encoded_callsign * 27;
    encoded_callsign = encode_char(callsign_arr[5]) as u32 + encoded_callsign * 27;

    Ok(encoded_callsign)
}

fn source_encode_locator(locator: &str) -> Result<u32, ErrorCode> {
    Ok(0u32)
}

fn source_encode_power(power: u8) -> Result<u8, ErrorCode> {
    match power {
        0..=60 => (),
        _ => return Err(ErrorCode::PowerEncodeError),
    }
    Ok(0u8)
}

#[test]
fn test_encode() {
    let msg = WSPRMessage::new("DB2LA", "JO43", 30);
    let channel_encoded = msg.encode().unwrap();
    for i in channel_encoded.iter() {
        print!("{}, ", i);
    }
    println!();
}

#[test]
fn test_decode() {
    let _msg = WSPRMessage::decode([0; 162]);
}

#[test]
fn test_src_encode_error_callsign_too_short() {
    let callsign = "DB"; // too short
    let e = source_encode_callsign(callsign);
    assert_eq!(e.unwrap_err(), ErrorCode::CallsignEncodeError);
}

#[test]
fn test_src_encode_error_callsign_too_long() {
    let callsign = "XXXXXXX"; // longer than 6 chars
    let e = source_encode_callsign(callsign);
    assert_eq!(e.unwrap_err(), ErrorCode::CallsignEncodeError);
}

#[test]
fn test_src_encode_error_callsign_3rd_not_none() {
    let callsign = "DDDDDD"; // 3rd char is NaN
    let e = source_encode_callsign(callsign);
    assert_eq!(e.unwrap_err(), ErrorCode::CallsignEncodeError);
}

#[test]
fn test_src_encode_error_callsign_3rd_nan_after_move() {
    let callsign = "2XYZ"; // 3rd char is NaN even after move
    let e = source_encode_callsign(callsign);
    assert_eq!(e.unwrap_err(), ErrorCode::CallsignEncodeError);
}

#[test]
fn test_src_encode_error_callsign_illegal_chars() {
    let callsign = "   "; // spaces only allowed in first place
    let e = source_encode_callsign(callsign);
    assert_eq!(e.unwrap_err(), ErrorCode::CallsignEncodeError);
}

#[test]
fn test_src_encode_prepend() {
    let mut callsign: [char; 6] = ['K', '1', 'J', 'T', ' ', ' '];
    prepend_space(&mut callsign);
    assert_eq!(callsign[2], '1');
}

#[test]
fn test_src_encode_callsign() {
    let callsign = "DB2LA";
    let src_encoded = source_encode_callsign(callsign).unwrap();
    println!("e: {:x}", src_encoded);
    assert_eq!(src_encoded, 0x59f7627);
}
