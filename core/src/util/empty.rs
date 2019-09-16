/// Indicates that there is some 'empty'
/// variant of this type which can be
/// constructed.
pub trait Empty {
	/// Contructs an 'empty' variant of
	/// this type.
	fn empty() -> Self;
}
