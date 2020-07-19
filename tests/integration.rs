use wspr::WSPRMessage;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::{self, BufRead};

/// test generated test messages from wspr.txt that contain the channel encoded
/// symbols encoded by the reference implementation
#[test]
fn integration() {
    let mut testdata_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    testdata_path.push("tests");
    testdata_path.push("wspr.txt");

    if let Ok(lines) = read_lines(testdata_path) {
        let lines_iter = lines.map(|l| l.unwrap());
        for line in lines_iter {
            let msg: Vec<&str> = line.trim().split(" ").collect();
            let mut chan_symbols: [u8; 162] = [0; 162];
            for (i, &sym) in msg.iter().skip(3).enumerate() {
                chan_symbols[i] = sym.to_string().parse::<u8>().unwrap();
            }
            let wspr_msg = WSPRMessage::new(msg[0], msg[1], msg[2].to_string().parse::<u8>().unwrap());
            println!("{} {} {}", wspr_msg.callsign, wspr_msg.locator, wspr_msg.power);
            let channel_encoded = wspr_msg.encode().unwrap();

            for (sym, reference) in channel_encoded.iter().zip(chan_symbols.iter()) {
                //println!("{} {}", sym, reference);
                
                assert_eq!(sym, reference);
            }
        }
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}