mod engines;
mod services;
mod processing;
mod context;
mod audioformat;

use context::AudioContext;
use engines::{AudioEngine, speaker::SpeakerEngine};
use getopts::Options;
use services::player::{AudioPlayerServiceRpc, AudioPlayerService};
use std::{thread, env};
use jsonrpc_core::IoHandler;
use jsonrpc_stdio_server::ServerBuilder;

fn print_usage(program: &str, opts: Options) {
	let brief = format!("Usage: {} [options]", program);
	print!("{}", opts.usage(&brief));
}

fn main() {
	let supported_engines = ["speaker"];

	// Parse CLI args
	let args: Vec<String> = env::args().collect();
	let program = args[0].clone();
	
	let mut opts = Options::new();
	opts.optopt("e", "engine", "Specifies which audio output is used", format!("[{}]", supported_engines.join("|")).as_str());
	opts.optopt("t", "token", "Optionally provides an authentication token if required by the engine", "TOKEN");
	
	let parsed_args = opts.parse(&args[1..]).unwrap();
	if !parsed_args.opt_present("e") {
		print_usage(&program, opts);
		return;
	}
	let engine_str = parsed_args.opt_str("e");
	
	// Spawn engine
	let context = AudioContext::new();
	let context_clone = context.clone();

	thread::spawn(move || {
		match engine_str.unwrap().as_str() {
			"speaker" => run_engine(SpeakerEngine, context_clone),
			_ => println!("Unrecognized engine, try one of these: {:?}.", supported_engines)
		};
	});

	// Setup RPC
	let mut io = IoHandler::default();
	io.extend_with(AudioPlayerService::with_context(context).to_delegate());
	
	ServerBuilder::new(io).build();
}

fn run_engine(engine: impl AudioEngine, context: AudioContext) {
	engine.run(context);
}
