use std::fmt::Debug;

pub fn drop_errors_or<T, E: Debug>(ran: Result<T, E>, default: T) -> T {
	match ran {
		Ok(t) => t,
		Err(e) => {
			eprintln!("Uncaught error: {:?}", e);

			return default;
		}
	}
}

pub fn drop_errors_or_default<T: Default, E: Debug>(ran: Result<T, E>) -> T {
	drop_errors_or(ran, T::default())
}