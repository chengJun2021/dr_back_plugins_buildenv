use std::fmt::Debug;

/// Suppresses errors by logging the error and then returning a supplied default value
pub fn drop_errors_or<T, E: Debug>(ran: Result<T, E>, default: T) -> T {
    match ran {
        Ok(t) => t,
        Err(e) => {
            error!("Uncaught error: {:?}", e);

            return default;
        }
    }
}

/// Suppresses errors by logging the error then returning the default value of the desired result
///
/// If you'd like to specify the default value, check out [`drop_errors_or`]
pub fn drop_errors_or_default<T: Default, E: Debug>(ran: Result<T, E>) -> T {
    drop_errors_or(ran, T::default())
}