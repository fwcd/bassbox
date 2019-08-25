//! A collection of audio outputs (i.e. those that
//! output the audio to the user).

use dsp::sample::Frame;
use dsp::Graph;
use std::sync::{Arc, Mutex};

/// An audio output capable of playing sound
/// to the user.
pub trait Sink<F, N> where F: Frame {
	/// Requests the sink to begin outputting audio from
	/// the provided DSP graph, usually using
	/// the `audio_requested` method.
	/// 
	/// The provided graph is possibly already being
	/// used by another thread controlling the nodes,
	/// hence an atomic ref-counted pointer is used.
	fn output(&mut self, graph: Arc<Mutex<Graph<F, N>>>);
	
	/// Requests the sink to continue playback, usually
	/// after a pause.
	fn play(&mut self);
	
	/// Requests the sink to pause playing.
	fn pause(&mut self);
}
