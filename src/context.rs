use dsp::Graph;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::sink::{Sink, default::DefaultSink};
use crate::processing::dspnode::DspNode;
use crate::audioformat::Frame;

/// A collection of sinks that can be added
/// or removed dynamically.
pub struct SinkCollection {
	pub default: DefaultSink,
	pub additional: Arc<Mutex<HashMap<String, Box<dyn Sink + Send>>>>
}

/// The audio state which is shared between
/// an engine and possibly control threads
/// (such as RPC-mechanisms).
pub struct AudioContext {
	pub graph: Arc<Mutex<Graph<Frame, DspNode>>>,
	pub sinks: SinkCollection
}

impl SinkCollection {
	pub fn new() -> SinkCollection {
		SinkCollection {
			default: DefaultSink::new(),
			additional: Arc::new(Mutex::new(HashMap::new()))
		}
	}
}

impl AudioContext {
	pub fn new() -> AudioContext {
		AudioContext {
			graph: Arc::new(Mutex::new(Graph::new())),
			sinks: SinkCollection::new()
		}
	}
}
