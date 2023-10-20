use std::error::Error;

use poem_openapi_derive::Object;

#[derive(Object, Debug)]
pub struct GenericResponseError {
    pub error: String,
}

impl<T> From<T> for GenericResponseError
where
    T: Error,
{
    fn from(err: T) -> Self {
        Self {
            error: err.to_string(),
        }
    }
}
