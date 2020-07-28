//! This executable will produce a .wav file with an encoded WSPR frame in 4-FSK modulation.
//! The resulting file could be used to feed the audio input of a TX to modulate it onto
//! one of the frequencies designated for WSPR transmissions in the ham bands

use hound::WavWriter;
use std::f32::consts::PI;
use std::{env, fs, io};
use wspr::WSPRMessage;

static SAMPLE_RATE: u32 = 44100;
static BAUD_RATE: f32 = 1.4648;
static SYMBOL_DURATION: u32 = (SAMPLE_RATE as f32 / BAUD_RATE) as u32;
static TONE_SEPARATION: f32 = 1.4648;
static AMPLITUDE: f32 = i16::MAX as f32 - 5000.0; // don't use max amplitude

struct SampleEmitter {
    symbol_count: u32,
    base_freq: f32,
    writer: WavWriter<io::BufWriter<fs::File>>,
}

impl SampleEmitter {
    fn new(writer: WavWriter<io::BufWriter<fs::File>>, base_freq: f32) -> Self {
        if !(1400.0..=1600.0).contains(&base_freq) {
            panic!("Base frequency not in range of 1400..1600 Hz");
        }
        Self {
            symbol_count: 0,
            base_freq,
            writer,
        }
    }

    fn calculate_sample(&self, t: f32, symbol: u8) -> f32 {
        (t * (self.base_freq + (TONE_SEPARATION * symbol as f32)) * 2.0 * PI).cos()
    }

    fn write_symbol(&mut self, symbol: u8) {
        let mut samples = Vec::new();

        for _ in 0..SYMBOL_DURATION {
            let t = self.symbol_count as f32 / SAMPLE_RATE as f32;
            self.symbol_count += 1;
            let sample = self.calculate_sample(t, symbol);
            samples.push(sample);
        }

        for sample in samples.iter() {
            self.writer
                .write_sample((sample * AMPLITUDE) as i16)
                .unwrap();
        }
    }
}

fn usage_and_exit() {
    println!("Usage: baseband_audio \"<wspr message>\" <base_frequency>");
    println!(" e.g.: baseband_audio \"DB2LA JO43 30\" 1500");
    std::process::exit(1);
}

fn main() {
    if env::args().len() != 3 {
        usage_and_exit();
    }

    let msg_arg = env::args().nth(1).unwrap();
    let freq_arg = env::args().nth(2).unwrap();
    let base_freq = freq_arg.parse::<f32>().unwrap();
    let msg = msg_arg.split(' ').collect::<Vec<&str>>();
    if msg.len() != 3 {
        usage_and_exit();
    }

    println!("Encoding message: {} {} {}", msg[0], msg[1], msg[2]);
    println!("Symbol Duration: {} samples", SYMBOL_DURATION);
    let wspr_msg = WSPRMessage::new(msg[0], msg[1], msg[2].parse::<u8>().unwrap());
    let encoded_msg = wspr_msg.encode().unwrap_or_else(|e| {
        panic!("error encoding wspr message: {:?}", e);
    });

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let writer = hound::WavWriter::create("wspr.wav", spec).unwrap();
    let mut symbol_writer = SampleEmitter::new(writer, base_freq);
    for &symbol in encoded_msg.iter() {
        symbol_writer.write_symbol(symbol);
    }
}
