use crate::wspr::ErrorCode;

/// Layland Lushbaugh generator polynomials
static POLY1: u32 = 0xF2D0_5351;
static POLY2: u32 = 0xE461_3C47;

/// compute the even parity of a u32
/// used here as the modulo-2 adder of bits selected by the generator polynomials
/// for the convolutional code
fn parity(val: u32) -> u8 {
    let mut v = val;
    v ^= v >> 16;
    v ^= v >> 8;
    v ^= v >> 4;
    v &= 0xf;
    ((0x6996 >> v) & 1) as u8
}

/// Convolutional coder with constraint lenght K=32, coding rate=1/2, non-systematic, non-recursive
/// Input is the 50 bit source encoded and packed payload
pub fn convolve(input: [u8; 50]) -> Result<[u8; 162], ErrorCode> {
    let mut encoded: [u8; 162] = [0; 162];
    let mut register: u32 = 0;

    // check bits are actually only 0 or 1
    if input.iter().any(|&x| x != 0 && x != 1) {
        return Err(ErrorCode::ConvolutionBitSetError)
    }

    // extend data to 81 bits for the tail shiftout of the codec
    let mut data: [u8; 81] = [0; 81];
    for (i, bit) in input.iter().enumerate() {
        data[i] = *bit;
    }

    for (i, bit) in data.iter().enumerate() {
        register <<= 1;
        register |= *bit as u32;
        encoded[i*2] = parity(register & POLY1);
        encoded[i*2+1] = parity(register & POLY2);
    }

    Ok(encoded)
}

#[test]
fn test_parity() {
    let p = parity(0xFF01);
    assert_eq!(p, 1);
    let p = parity(0xFFFF0000);
    assert_eq!(p, 0);
}

#[test]
fn test_convolve() {
    // expected values verified by infallible manual calculation
    let mut d = [0u8; 50];
    for i in 0..8 {
        d[50-8+i] = (0xDB >> (7-i)) & 0x01;
    }

    let e = convolve(d).unwrap();
    assert_eq!(e[49*2], 1);
    assert_eq!(e[49*2+1], 1);

    let mut d = [0u8; 50];
    for i in 0..8 {
        d[50-8+i] = (0xAF >> (7-i)) & 0x01;
    }

    let e = convolve(d).unwrap();
    assert_eq!(e[49*2], 1);
    assert_eq!(e[49*2+1], 1);
}

#[test]
fn test_convolve_error() {
    let mut d = [0u8; 50];
    d[0] = 0x02;
    let e = convolve(d);
    assert!(e.is_err());
}
