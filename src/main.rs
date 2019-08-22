extern crate rodio;

use std::fs::File;
use std::io::BufReader;
use rodio::Source;
use rodio::Sink;

fn main() {
	let device = rodio::default_output_device().unwrap();
	let file = File::open("local/oldtownroad.mp3").unwrap();
	let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
	let sink = Sink::new(&device);
	sink.append(source);
	sink.sleep_until_end();
}
