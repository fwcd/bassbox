use super::{AudioEngine, BackgroundEngine};
use crate::audioformat::{StandardFrame, empty_standard_frame, STANDARD_CHANNELS, StandardSample};
use crate::graph::SharedAudioGraph;
use dsp::Node;
use dsp::sample::{Sample, Frame, FromSample, conv::ToFrameSliceMut};
use cpal::{StreamData, UnknownTypeOutputBuffer, OutputBuffer};
use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};

/// An engine that uses CPAL to provide
/// cross-platform audio output.
pub struct SpeakerEngine;

impl AudioEngine for SpeakerEngine {
	fn run_async(self, shared_graph: SharedAudioGraph) -> BackgroundEngine {
		// Setup CPAL
		let host = cpal::default_host();
		let event_loop = host.event_loop();
		let device = host.default_output_device().expect("No output device available.");
		let format = device.default_output_format().expect("No default format.");
		let sample_hz = format.sample_rate.0 as f64;
		let channels = format.channels as usize;
		let stream_id = event_loop.build_output_stream(&device, &format).expect("Could not build output stream");
		
		if channels != STANDARD_CHANNELS {
			// TODO
			println!("CPAL output format has {} channels, but only exactly {} are supported currently.", channels, STANDARD_CHANNELS);
		}
		
		event_loop.play_stream(stream_id.clone()).expect("Could not play stream.");
		std::thread::spawn(move || {
			event_loop.run(move |_, result| {
				let mut data = result.expect("Error while streaming.");
				let sample_count = buffer_sample_count(&data).unwrap_or(0);
				let frame_count = sample_count / STANDARD_CHANNELS;
				
				// Read audio from graph into temporary buffer
				let mut audio: Vec<StandardFrame> = vec![empty_standard_frame(); frame_count];
				shared_graph.lock().unwrap().audio_requested(&mut audio, sample_hz);
				
				match data {
					StreamData::Output { buffer: UnknownTypeOutputBuffer::U16(ref mut buffer) } => write_audio(&audio, buffer),
					StreamData::Output { buffer: UnknownTypeOutputBuffer::I16(ref mut buffer) } => write_audio(&audio, buffer),
					StreamData::Output { buffer: UnknownTypeOutputBuffer::F32(ref mut buffer) } => write_audio(&audio, buffer),
					_ => println!("Audio format was not recognized by CPAL!")
				}
			});
		});

		BackgroundEngine { sample_hz: sample_hz }
	}
}

/// Fetches CPAL's stream data buffer sample count.
fn buffer_sample_count(data: &StreamData) -> Option<usize> {
	match *data {
		StreamData::Output { buffer: UnknownTypeOutputBuffer::U16(ref buffer) } => Some(buffer.len()),
		StreamData::Output { buffer: UnknownTypeOutputBuffer::I16(ref buffer) } => Some(buffer.len()),
		StreamData::Output { buffer: UnknownTypeOutputBuffer::F32(ref buffer) } => Some(buffer.len()),
		_ => None
	}
}

/// Writes the audio to the speaker in a specified format.
fn write_audio<S>(audio: &[StandardFrame], buffer: &mut OutputBuffer<S>) where S: cpal::Sample + Sample + FromSample<StandardSample> {
	let buf_slice: &mut [[S; STANDARD_CHANNELS]] = buffer.to_frame_slice_mut().unwrap();
	for (i, frame) in buf_slice.iter_mut().enumerate() {
		*frame = audio[i].map(Sample::to_sample);
	}
}
