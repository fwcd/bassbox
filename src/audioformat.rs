//! Defines a default output format that is used
//! internally by the DSP processing.

use dsp::Frame;

pub type StandardSample = f32;
pub const STANDARD_CHANNELS: usize = 2;
pub type StandardFrame = [StandardSample; STANDARD_CHANNELS];

/// Basic arithmetic operating on the signal's amplitude
/// for standard frames. More flexible than add_amp and scale_amp
/// since it provides subtraction and division operations too.
pub trait OpsExt {
	fn add(self, rhs:StandardFrame) -> StandardFrame;
	
	fn sub(self, rhs:StandardFrame) -> StandardFrame;
	
	fn mul(self, rhs:StandardFrame) -> StandardFrame;
	
	fn div(self, rhs:StandardFrame) -> StandardFrame;
	
	fn scale(self, rhs:f32) -> StandardFrame;
}

impl OpsExt for StandardFrame {
	fn add(self, rhs:StandardFrame) -> StandardFrame { self.zip_map(rhs, |a, b| a + b) }
	
	fn sub(self, rhs:StandardFrame) -> StandardFrame { self.zip_map(rhs, |a, b| a - b) }
	
	fn mul(self, rhs:StandardFrame) -> StandardFrame { self.zip_map(rhs, |a, b| a * b) }
	
	fn div(self, rhs:StandardFrame) -> StandardFrame { self.zip_map(rhs, |a, b| a / b) }
	
	fn scale(self, rhs:f32) -> StandardFrame { self.map(|x| x * rhs) }

}
