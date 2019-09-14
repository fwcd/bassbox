/// A general-purpose sum type with two values.
pub enum Either<L, R> {
	Left(L),
	Right(R)
}
