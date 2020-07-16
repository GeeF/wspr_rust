use wspr::WSPRMessage;

#[test]
fn integration() {
    let msg = WSPRMessage::new("DB2LA", "JO43", 30);
    let channel_encoded = msg.encode().unwrap();
    for i in channel_encoded.iter() {
        print!("{}, ", i);
    }
    println!();
}