use super::AudioEngine;
use crate::audioformat::{StandardFrame, empty_standard_frame, STANDARD_CHANNELS};
use crate::graph::SharedAudioGraph;
use dsp::{Node, sample::Sample};
use cpal::{StreamData, UnknownTypeOutputBuffer};
use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};

/// An engine that uses CPAL to provide
/// cross-platform audio output.
pub struct SpeakerEngine;

impl AudioEngine for SpeakerEngine {
	fn run(self, shared_graph: SharedAudioGraph) {
		// Setup CPAL
		let host = cpal::default_host();
		let event_loop = host.event_loop();
		let device = host.default_output_device().expect("No output device available.");
		let format = device.default_output_format().expect("No default format.");
		let sample_rate = format.sample_rate.0 as f64;
		let stream_id = event_loop.build_output_stream(&device, &format).expect("Could not build output stream");
		
		event_loop.play_stream(stream_id.clone()).expect("Could not play stream.");
		event_loop.run(move |_, result| {
			let data = result.expect("Error while streaming.");
			let mut req_buffer: Vec<StandardFrame> = vec![empty_standard_frame(); buffer_size(&data).expect("Not output data.")];
			
			// Read audio into temporary buffer
			shared_graph.graph.lock().unwrap().audio_requested(&mut req_buffer, sample_rate);
			write_audio(&req_buffer, data);
		});
	}
}

/// Fetches CPAL's stream data buffer size.
fn buffer_size(data: &StreamData) -> Option<usize> {
	match *data {
		StreamData::Output { buffer: UnknownTypeOutputBuffer::U16(ref buffer) } => Some(buffer.len()),
		StreamData::Output { buffer: UnknownTypeOutputBuffer::I16(ref buffer) } => Some(buffer.len()),
		StreamData::Output { buffer: UnknownTypeOutputBuffer::F32(ref buffer) } => Some(buffer.len()),
		_ => None
	}
}

/// Writes the audio to the speaker possibly converting
/// to a different format on-the-fly.
fn write_audio(audio: &[StandardFrame], data: StreamData) {
	match data {
		StreamData::Output { buffer: UnknownTypeOutputBuffer::U16(mut buffer) } => {
			for (i, frame) in buffer.chunks_mut(STANDARD_CHANNELS).enumerate() {
				for (j, c) in frame.iter_mut().enumerate() {
					*c = audio[i][j].to_sample();
				}
			}
		},
		StreamData::Output { buffer: UnknownTypeOutputBuffer::I16(mut buffer) } => {
			for (i, frame) in buffer.chunks_mut(STANDARD_CHANNELS).enumerate() {
				for (j, c) in frame.iter_mut().enumerate() {
					*c = audio[i][j].to_sample();
				}
			}
		},
		StreamData::Output { buffer: UnknownTypeOutputBuffer::F32(mut buffer) } => {
			for (i, frame) in buffer.chunks_mut(STANDARD_CHANNELS).enumerate() {
				for (j, c) in frame.iter_mut().enumerate() {
					*c = audio[i][j];
				}
			}
		},
		_ => println!("CPAL stream format unsupported.")
	}
}
