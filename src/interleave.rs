/// bit magic to reverse the bit positions of a byte
fn reverse_byte(b: u8) -> u8 {
    (((((b as u64 * 0x0802u64) & 0x22110u64) | ((b as u64 * 0x8020u64) & 0x88440u64)) * 0x10101u64) >> 16)
        as u8
}

/// Interleaver for 162 convoluted WSPR data bits
pub fn interleave(data: [u8; 162]) -> [u8; 162] {
    let mut interleaved: [u8; 162] = [0; 162];

    let rmap = (0..255).map(reverse_byte).filter(|&x| x < 162);
    
    for (n, r) in rmap.enumerate() {
        interleaved[r as usize] = data[n];
    }

    interleaved
}

#[test]
fn test_reverse() {
    assert_eq!(reverse_byte(0b00000001), 0b10000000);
    assert_eq!(reverse_byte(0b11110000), 0b00001111);
    assert_eq!(reverse_byte(0b10101010), 0b01010101);
}

#[test]
fn test_interleaver() {
    let mut data: [u8; 162] = [0; 162];
    data[0] = 1;
    data[1] = 1;
    data[2] = 1;
    data[3] = 1;
    data[4] = 1;
    data[5] = 1;

    let interleaved_data = interleave(data);
    
    assert_eq!(interleaved_data[0], 1);
    assert_eq!(interleaved_data[32], 1);
    assert_eq!(interleaved_data[64], 1);
    assert_eq!(interleaved_data[160], 1);
    assert_eq!(interleaved_data[96], 1);
}