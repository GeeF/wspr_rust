use wspr;

#[test]
fn integration() {
    let mut msg = wspr::WSPRMessage::decode([0; 162]);
    let c = msg.callsign;
    msg.callsign = "DB2LA".to_string();
}