use std::iter;

pub trait Shard:
	Clone + AsRef<[u8]> + AsMut<[u8]> + AsMut<[[u8; 2]]> + AsRef<[[u8; 2]]> + iter::FromIterator<[u8; 2]> + From<Vec<u8>>
{
	type Inner;
	fn into_inner(self) -> Self::Inner;
}

impl<T> Shard for T
where
	T: Clone
		+ AsRef<[u8]>
		+ AsMut<[u8]>
		+ AsMut<[[u8; 2]]>
		+ AsRef<[[u8; 2]]>
		+ iter::FromIterator<[u8; 2]>
		+ From<Vec<u8>>,
{
	type Inner = Self;
	fn into_inner(self) -> Self::Inner {
		self
	}
}
