//! Core library crate.

pub fn hello() -> &'static str {
	"Hello from core!"
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_hello() {
		assert_eq!(hello(), "Hello from core!");
	}
}
