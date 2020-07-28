//! WSPR encoder / decoder
use crate::convcode::convolve;
use crate::interleave::interleave;

/// Used to construct the final channel symbols from the convolved and interleaved data
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
    CallsignEncodeError,    // Callsign does not match WSPR spec
    LocatorEncodeError,     // Locator does not match WSPR spec
    PowerEncodeError,       // Power does not match WSPR spec
}

/// WSPR Message representation holding the unencoded data of the message
/// 
/// May contain invalid data. Checks are only applied on encode
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

    pub fn decode(_symbols: [u8; 162]) -> Self {
        // tbd: fano decoder
        unimplemented!();
    }

    /// Get the channel encoded 4-FSK symbols for a standard message
    /// 
    /// Will throw an error code if the data of the message does not conform to the spec
    pub fn encode(&self) -> Result<[u8; 162], ErrorCode> {
        let mut encoded_frame = [0u8; 162];
        let s = SourceFrame::new(self)?;
        let interleaved_frame = interleave(convolve(s.packed_src_frame())?);
        for (i, elem) in interleaved_frame.iter().enumerate() {
            encoded_frame[i as usize] = SYNC_VECTOR[i] + 2 * elem;
        }

        Ok(encoded_frame)
    }
}

/// Source encoded WSPR frame
/// 
/// Holds the 50 bit source encoded data in two u32s
struct SourceFrame {
    callsign: u32,
    locator_power: u32,
}

impl SourceFrame {
    fn new(msg: &WSPRMessage) -> Result<Self, ErrorCode> {
        Ok(Self {
            callsign: source_encode_callsign(&msg.callsign)?,
            locator_power: source_encode_locator_power(&msg.locator, msg.power)?,
        })
    }

    /// pack into one u64 with only the right most 50 bits used
    fn packed_src_frame(&self) -> [u8; 50] {
        let encoded = (self.callsign as u64) << 22 | self.locator_power as u64;

        let mut packed_src_frame: [u8; 50] = [0; 50];
        for (i, element) in packed_src_frame.iter_mut().enumerate() {
            *element = (encoded >> (50 - i - 1) & 1) as u8;
        }

        packed_src_frame
    }
}

/// Shift all elements in the char array to the right. Will cut the right most element if present
/// 
/// Used for the source encoding
fn prepend_space(arr: &mut [char]) {
    for i in 0..5 {
        arr[5 - i] = arr[4 - i];
    }
    arr[0] = ' ';
}

/// Encode a single character according to the WSPR spec
///
/// Illegal characters should be checked before this
fn encode_char(c: char) -> u8 {
    match c {
        '0'..='9' => c as u8 - 48, // '0'-'9' as 0-9
        'A'..='Z' => c as u8 - 55, // 'A'-'Z' as 10-35
        ' ' => 36,                 // space is 36
        _ => 0,                    // illegal char: 0
    }
}

/// Character only fields are encoded as 0-26
fn encode_alpha_only(c: char) -> u8 {
    match c {
        'A'..='Z' => c as u8 - 65,
        ' ' => 26,
        _ => 0,
    }
}

/// Source encoding of the callsign checking for various constraints
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
        (' ', 'A'..='Z', '0'..='9')
        | ('A'..='Z', 'A'..='Z', '0'..='9')
        | ('0'..='9', 'A'..='Z', '0'..='9') => (),
        _ => return Err(ErrorCode::CallsignEncodeError),
    }

    // last 3 characters have to be letters or space
    match (callsign_arr[3], callsign_arr[4], callsign_arr[5]) {
        ('A'..='Z', 'A'..='Z', 'A'..='Z')
        | ('A'..='Z', 'A'..='Z', ' ')
        | ('A'..='Z', ' ', ' ')
        | (' ', ' ', ' ') => (),
        _ => return Err(ErrorCode::CallsignEncodeError),
    }

    // encode characters, packed to 28 bits maximum
    let mut encoded_callsign = encode_char(callsign_arr[0]) as u32;
    encoded_callsign = encode_char(callsign_arr[1]) as u32 + encoded_callsign * 36;
    encoded_callsign = encode_char(callsign_arr[2]) as u32 + encoded_callsign * 10;
    encoded_callsign = encode_alpha_only(callsign_arr[3]) as u32 + encoded_callsign * 27;
    encoded_callsign = encode_alpha_only(callsign_arr[4]) as u32 + encoded_callsign * 27;
    encoded_callsign = encode_alpha_only(callsign_arr[5]) as u32 + encoded_callsign * 27;

    Ok(encoded_callsign)
}

/// Source encoding for locator and power
/// 
/// Both are encoded into a combined 22 bit representation
fn source_encode_locator_power(locator: &str, power: u8) -> Result<u32, ErrorCode> {
    if locator.len() != 4 {
        return Err(ErrorCode::LocatorEncodeError);
    }

    let mut locator_arr: [char; 4] = [' '; 4];
    for (n, c) in locator.to_uppercase().chars().enumerate() {
        locator_arr[n] = c;
    }

    // check locator format
    match (
        locator_arr[0],
        locator_arr[1],
        locator_arr[2],
        locator_arr[3],
    ) {
        ('A'..='R', 'A'..='R', '0'..='9', '0'..='9') => (),
        _ => return Err(ErrorCode::LocatorEncodeError),
    }

    let encoded_chars: [u32; 4] = [
        encode_alpha_only(locator_arr[0]) as u32,
        encode_alpha_only(locator_arr[1]) as u32,
        encode_char(locator_arr[2]) as u32,
        encode_char(locator_arr[3]) as u32,
    ];

    let encoded_locator = (179 - 10 * encoded_chars[0] as i32 - encoded_chars[2] as i32) * 180
        + 10 * encoded_chars[1] as i32
        + encoded_chars[3] as i32;

    // check power
    if power > 60 {
        return Err(ErrorCode::PowerEncodeError);
    }

    // cobine locator and power
    Ok(encoded_locator as u32 * 128 + 64 + power as u32)
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
    assert_eq!(src_encoded, 0x59f5895);
}

#[test]
fn test_src_encode_mixed_case_callsign() {
    let callsign = "dB2La";
    let src_encoded = source_encode_callsign(callsign).unwrap();
    assert_eq!(src_encoded, 0x59f5895);
}

#[test]
fn test_src_encode_locator_power() {
    let src_encoded = source_encode_locator_power("JO43", 30).unwrap();
    assert_eq!(src_encoded, 0x1e29de);
}

#[test]
fn test_src_encode_error_locator() {
    let src_encoded = source_encode_locator_power("ZZ43", 30); // Illegal chars in locator (only A-R)
    assert_eq!(src_encoded.unwrap_err(), ErrorCode::LocatorEncodeError);
    let locator = "JO3Z"; // Illegal chars in locator (only A-R)
    let src_encoded = source_encode_locator_power(locator, 30);
    assert_eq!(src_encoded.unwrap_err(), ErrorCode::LocatorEncodeError);
}

#[test]
fn test_src_encode_error_power() {
    let src_encoded = source_encode_locator_power("JO43", 61); // power value >60
    assert_eq!(src_encoded.unwrap_err(), ErrorCode::PowerEncodeError);
}

#[test]
fn test_packed_src_frame() {
    let src_frame = SourceFrame::new(&WSPRMessage::new("DB2LA", "JO43", 30)).unwrap();
    let packed_src_frame = src_frame.packed_src_frame();
    let reference_src_frame: [u8; 50] = [
        0, 1, 0, 1, 1, 0, 0, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 0, 0, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0, 1,
        1, 1, 1, 0, 0, 0, 1, 0, 1, 0, 0, 1, 1, 1, 0, 1, 1, 1, 1, 0,
    ];
    for (left, right) in packed_src_frame.iter().zip(reference_src_frame.iter()) {
        assert_eq!(left, right);
    }
}

#[test]
fn test_data_symbols() {
    let src_frame = SourceFrame::new(&WSPRMessage::new("DB2LA", "JO43", 30)).unwrap();
    let convolved = convolve(src_frame.packed_src_frame()).unwrap();
    let interleaved = interleave(convolved);
    let reference_data_symbols = [
        0, 1, 0, 1, 0, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 0, 1, 0, 1, 1, 0, 0, 0, 0, 1, 1,
        1, 0, 1, 1, 0, 1, 1, 1, 1, 0, 1, 0, 0, 1, 1, 1, 0, 1, 0, 1, 1, 0, 1, 0, 0, 1, 0, 1, 0, 1,
        1, 1, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 0,
        1, 0, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 1, 0, 1, 1, 1, 0, 0, 1, 1, 0,
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1, 0, 1, 1, 1,
        1, 1, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1,
    ];
    for (left, right) in interleaved.iter().zip(reference_data_symbols.iter()) {
        assert_eq!(left, right);
    }
}

#[test]
fn test_channel_symbols() {
    let encoded_wspr_msg = WSPRMessage::new("DB2LA", "JO43", 30).encode().unwrap();
    let reference_channel_symbols = [
        1, 3, 0, 2, 0, 0, 2, 2, 1, 2, 2, 2, 3, 3, 1, 0, 2, 0, 1, 0, 2, 1, 2, 3, 1, 1, 1, 0, 2, 2,
        2, 0, 2, 2, 1, 2, 2, 3, 2, 1, 2, 0, 0, 2, 2, 2, 1, 2, 1, 3, 2, 0, 3, 1, 0, 3, 0, 2, 0, 3,
        3, 2, 1, 0, 2, 2, 0, 1, 1, 2, 3, 0, 3, 2, 3, 2, 3, 0, 0, 3, 2, 0, 1, 0, 1, 1, 2, 2, 2, 1,
        3, 0, 3, 2, 3, 2, 2, 2, 1, 0, 2, 2, 2, 0, 1, 0, 0, 3, 0, 0, 3, 1, 3, 2, 3, 1, 0, 2, 3, 1,
        2, 1, 0, 0, 0, 1, 1, 1, 0, 0, 2, 2, 2, 3, 2, 3, 0, 0, 1, 1, 2, 0, 2, 2, 0, 2, 0, 3, 3, 2,
        3, 2, 1, 1, 0, 0, 2, 1, 3, 2, 0, 2,
    ];
    for (left, right) in encoded_wspr_msg
        .iter()
        .zip(reference_channel_symbols.iter())
    {
        assert_eq!(left, right);
    }
}
