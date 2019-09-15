pub mod services;

use bassbox_core::graph::new_shared_graph;
use bassbox_core::engine::{AudioEngine, speaker::SpeakerEngine};
use bassbox_core::processing::DspNode;
use getopts::Options;
use services::player::{AudioPlayerServiceRpc, AudioPlayerService};
use bassbox_rpc_api::AudioGraphServiceRpc;
use services::graph::AudioGraphService;
use std::env;
use jsonrpc_core::IoHandler;
use jsonrpc_stdio_server::ServerBuilder;

fn print_usage(program: &str, opts: Options) {
	let brief = format!("Usage: {} [--engine ENGINE] [options]", program);
	print!("{}", opts.usage(&brief));
}

// TODO: Support HTTP/MPSC as an alternative to Stdio

fn main() {
	let supported_engines = ["speaker"];

	// Parse CLI args
	let args: Vec<String> = env::args().collect();
	let program = args[0].clone();
	
	let mut opts = Options::new();
	opts.optopt("e", "engine", "Specifies which audio output is used", format!("[{}]", supported_engines.join("|")).as_str());
	opts.optopt("t", "token", "Optionally provides an authentication token if required by the engine", "TOKEN");
	
	let parsed_args = opts.parse(&args[1..]).unwrap();
	let engine_str = match parsed_args.opt_str("engine") {
		Some(s) => s,
		None => {
			println!("Missing engine.");
			print_usage(&program, opts);
			return;
		}
	};
	
	// Spawn engine
	let shared_graph = new_shared_graph();
	let background_engine = match engine_str.as_str() {
		"speaker" => SpeakerEngine.run_async(shared_graph.clone()),
		_ => panic!("Unrecognized engine, try one of these: {:?}.", supported_engines)
	};

	// Setup RPC server
	let mut io = IoHandler::new();
	io.extend_with(AudioGraphService::using_graph(shared_graph, background_engine.clone()).to_delegate());
	
	ServerBuilder::new(io).build();
}
