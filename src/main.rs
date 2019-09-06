mod engine;
mod services;
mod processing;
mod source;
mod graph;
mod audioformat;

use graph::new_shared_graph;
use engine::{AudioEngine, speaker::SpeakerEngine};
use getopts::Options;
use services::player::{AudioPlayerServiceRpc, AudioPlayerService};
use services::graph::{AudioGraphServiceRpc, AudioGraphService};
use std::env;
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
	let shared_graph = new_shared_graph();
	let background_engine = match engine_str.unwrap().as_str() {
		"speaker" => SpeakerEngine.run_async(shared_graph.clone()),
		_ => panic!("Unrecognized engine, try one of these: {:?}.", supported_engines)
	};

	// Setup RPC services
	let mut io = IoHandler::default();
	io.extend_with(AudioPlayerService::constructing_graph(shared_graph.clone(), background_engine).to_delegate());
	io.extend_with(AudioGraphService::using_graph(shared_graph.clone()).to_delegate());
	
	ServerBuilder::new(io).build();
}
