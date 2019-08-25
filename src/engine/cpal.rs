use std::thread;
use sink::Sink;
use super::AudioEngine;
use crate::audioformat::{Frame, empty_frame};
use crate::context::AudioContext;
use cpal::{StreamData, UnknownTypeOutputBuffer};

/// An engine that uses CPAL to provide
/// cross-platform audio output.
pub struct CpalEngine;

impl AudioEngine for CpalEngine {
	fn run(self, context: AudioContext) {
		// Setup CPAL
		let host = cpal::default_host();
		let event_loop = host.event_loop();
		let device = host.default_output_device().expect("No output device available.");
		let format = host.default_output_format().expect("No default format.");
		let sample_rate = format.sample_rate.0 as f64;
		let stream_id = event_loop.build_output_stream(&device, &format).expect("Could not build output stream");
		
		event_loop.play_stream(stream_id.clone()).expect("Could not play stream.");
		event_loop.run(move |id, result| {
			let data = result.expect("Error while streaming.");
			let mut req_buffer: Vec<Frame> = vec![empty_frame(); buffer_size(&data)];
			
			// Read audio into temporary buffer
			graph.audio_requested(&mut req_buffer, sample_rate);
			
			if context.sinks.default.enabled {
				write_audio(&req_buffer, &mut data);
			}
			
			for sink in context.sinks.additional.lock().unwrap().values() {
				sink.output_audio(&req_buffer);
			}
		});
	}
}

/// Fetches CPAL's stream data buffer size.
fn buffer_size(data: &StreamData) -> Option<usize> {
	match *data {
		StreamData::Output { buffer: UnknownTypeOutputBuffer::U16(ref buffer) } => Some(buffer.len()),
		StreamData::Output { buffer: UnknownTypeOutputBuffer::I16(ref buffer) } => Some(buffer.len()),
		StreamData::Output { buffer: UnknownTypeOutputBuffer::F32(ref buffer) } => Some(buffer.len()),
		() => None
	}
}

/// Writes the audio to the speaker possibly converting
/// to a different format on-the-fly.
fn write_audio(audio: &[Frame], data: StreamData) {
	match data {
		StreamData::Output { buffer: UnknownTypeOutputBuffer::U16(mut buffer) } => {
			for (i, sample) in buffer.chunks_mut(CHANNELS).enumerate() {
				for (j, c) in sample.iter_mut().enumerate() {
					*c = ((audio[i][j] * 0.5 + 0.5) * std::u16::MAX as f32) as u16;
				}
			}
		},
		StreamData::Output { buffer: UnknownTypeOutputBuffer::I16(mut buffer) } => {
			for (i, sample) in buffer.chunks_mut(CHANNELS).enumerate() {
				for (j, c) in sample.iter_mut().enumerate() {
					*c = (audio[i][j] * std::i16::MAX as f32) as i16;
				}
			}
		},
		StreamData::Output { buffer: UnknownTypeOutputBuffer::F32(mut buffer) } => {
			for (i, sample) in buffer.chunks_mut(CHANNELS).enumerate() {
				for (j, c) in sample.iter_mut().enumerate() {
					*c = audio[i][j];
				}
			}
		},
		_ => println!("Format unsupported: {:?}", format)
	}
}
