use std::error::Error;

pub trait ResultToString<T, E: ToString> {
    fn str_res(self) -> Result<T, String>;
}
impl<T,E: Error> ResultToString<T, E> for Result<T,E> {
    fn str_res(self) -> Result<T, String> {
        match self {
            Ok(val) => Ok(val),
            Err(err) => Err(err.to_string()),
        }
    }
}
