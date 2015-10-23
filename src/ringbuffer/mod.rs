use std::error::Error;
use std::fmt;
use std::fmt::{ Debug, Display, Formatter };

#[derive(Debug, Clone, PartialEq)]
/// RingBuffer to store elements in a fixed size list, overwriting
/// the oldest elements when its capacity is full.
pub struct RingBuffer<T: Copy + Debug> {
	buf: Vec<T>,
	pos: usize,
	cap: usize,
}

impl<T: Copy + Debug> RingBuffer<T> {
	/// Creates a new RingBuffer populated with the elements in v,
	/// with capacity == v.len().
	/// Takes ownership of v.
	pub fn from_vec(v: Vec<T>) -> RingBuffer<T> {
		let c = v.len();
		RingBuffer {
			buf: v.iter().map(|&b| b).rev().collect::<Vec<_>>(),
			pos: c - 1,
			cap: c,
		}
	}

	/// Creates a new RingBuffer with a max capacity of c.
	pub fn with_capacity(c: usize) -> RingBuffer<T> {
		RingBuffer {
			buf: Vec::with_capacity(c),
			pos: 0,
			cap: c,
		}
	}

	/// Returns a result containing the nth element from the back,
	/// i.e. the 0th element is the last element that has been pushed.
	/// Returns RingBufferError::ParameterExceededSize, if n exceeds
	/// the buffers length or number of stored items.
	pub fn nth(&self, n: usize) -> Result<&T, RingBufferError> {
		let len = self.buf.len();

		// @Note: Uncommenting this line eats performance, even if Debugging is set to None
		//        because the format string is being non-lazily evaluated, potentially
		//        iterating over a huge buffer.
		// debug(&format!("RingBuffer::nth(): {:?}", (self.clone(), self.buf.len(), n)));

		if n >= len {
			Err(RingBufferError::ParameterExceededSize)
		} else {
			Ok(&self.buf[(self.pos + len - n) % len])
		}
	}

	pub fn slice_distance_length(&self, n: usize, _l: usize, buf: &mut [T]) -> Result<(), RingBufferError> {
		let len = self.buf.len();

		if n >= len {
			Err(RingBufferError::ParameterExceededSize)
		} else {
			// @Note: Uncommenting this line eats performance, even if Debugging is set to None
			//        because the format string is being non-lazily evaluated, potentially
			//        iterating over a huge buffer.
			// debug(&format!("RingBuffer::slice_distance_length(): {:?}", (self.clone(), self.buf.len(), n, len)));

			for (i, mut item) in buf.iter_mut().enumerate() {
				*item = self.buf[(self.pos + len - n + i) % len];
			}
			Ok(())
		}
	}

	/// Pushes an item to the end of the ring buffer.
	pub fn push(&mut self, item: T) {
		let len = self.buf.len();
		if len < self.cap {
			self.buf.push(item);
			self.pos = len;
		} else {
			self.pos = (self.pos + 1) % len;
			self.buf[self.pos] = item;
		}
	}
}

#[test]
fn should_retrieve_last_item() {
	let mut buf = RingBuffer::with_capacity(2);
	let item = 15;
	buf.push(item);

	assert_eq!(item, *buf.nth(0).unwrap());;
}

#[derive(Debug, Clone, PartialEq)]
enum RingBufferError {
	ParameterExceededSize,
}

impl Display for RingBufferError {
	fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {

		fmt.write_str(self.description())
	}
}

impl Error for RingBufferError {
	fn description(&self) -> &str {
		match *self {
			RingBufferError::ParameterExceededSize => "Index parameter exceeded ring buffer size",
		}
	}
}
