/// Layland Lushbaugh generator polynomials
static POLY1: u32 = 0xF2D05351;
static POLY2: u32 = 0xE4613C47;

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

/// bit magic to reverse the bit positions of a byte
fn reverse_byte(b: u8) -> u8 {
    (((b as u64 * 0x0802u64 & 0x22110u64) | (b as u64 * 0x8020u64 & 0x88440u64)) * 0x10101u64 >> 16)
        as u8
}

/// Convolutional coder with contraint lenght K=32, coding rate=1/2, non-systematic, non-recursive
/// Input is the 50 bit source encoded and packed payload
pub fn encode(input: [u8; 50]) -> Result<[u8; 162], ()> {
    let mut encoded: [u8; 162] = [0; 162];
    let mut register: u32 = 0;

    // TODO: check bits are actually only 0 || 1

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
fn test_reverse() {
    assert_eq!(reverse_byte(0b00000001), 0b10000000);
    assert_eq!(reverse_byte(0b11110000), 0b00001111);
    assert_eq!(reverse_byte(0b10101010), 0b01010101);
}

#[test]
fn test_encode() {
    // expected values verified by infallible manual calculation
    let mut d = [0u8; 50];
    d[49] = 0xDB;

    let e = encode(d).unwrap();
    assert_eq!(e[49*2], 1);
    assert_eq!(e[49*2+1], 1);

    let mut d = [0u8; 50];
    d[49] = 0xAF;

    let e = encode(d).unwrap();
    assert_eq!(e[49*2], 1);
    assert_eq!(e[49*2+1], 1);
}