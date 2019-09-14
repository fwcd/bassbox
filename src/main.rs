mod engine;
mod services;
mod processing;
mod source;
mod graph;
mod audioformat;
mod util;

use graph::new_shared_graph;
use engine::{AudioEngine, speaker::SpeakerEngine};
use processing::DspNode;
use getopts::Options;
use services::player::{AudioPlayerServiceRpc, AudioPlayerService};
use services::graph::{AudioGraphServiceRpc, AudioGraphService};
use std::env;
use jsonrpc_core::IoHandler;
use jsonrpc_stdio_server::ServerBuilder;

fn print_usage(program: &str, opts: Options) {
	let brief = format!("Usage: {} [--engine ENGINE] [options]", program);
	print!("{}", opts.usage(&brief));
}

fn main() {
	let supported_engines = ["speaker"];

	// Parse CLI args
	let args: Vec<String> = env::args().collect();
	let program = args[0].clone();
	
	let mut opts = Options::new();
	opts.optflag("r", "raw-graph", "If set, an 'empty' audio graph containing only a single master node will be used and the 'audioPlayer' service will not be available");
	opts.optopt("e", "engine", "Specifies which audio output is used", format!("[{}]", supported_engines.join("|")).as_str());
	opts.optopt("t", "token", "Optionally provides an authentication token if required by the engine", "TOKEN");
	
	let parsed_args = opts.parse(&args[1..]).unwrap();
	if !parsed_args.opt_present("engine") {
		println!("Missing engine.");
		print_usage(&program, opts);
		return;
	}
	let engine_str = parsed_args.opt_str("engine");
	
	// Spawn engine
	let shared_graph = new_shared_graph();
	let background_engine = match engine_str.unwrap().as_str() {
		"speaker" => SpeakerEngine.run_async(shared_graph.clone()),
		_ => panic!("Unrecognized engine, try one of these: {:?}.", supported_engines)
	};

	// Setup RPC services
	let mut io = IoHandler::default();
	if parsed_args.opt_present("raw-graph") {
		let mut graph = shared_graph.lock().unwrap();
		let master = graph.add_node(DspNode::Empty);
		graph.set_master(Some(master));
	} else {
		io.extend_with(AudioPlayerService::constructing_graph(shared_graph.clone(), background_engine.clone()).to_delegate());
	}
	io.extend_with(AudioGraphService::using_graph(shared_graph.clone(), background_engine.clone()).to_delegate());
	
	ServerBuilder::new(io).build();
}
