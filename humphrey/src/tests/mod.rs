#[cfg(not(feature = "tokio"))]
pub mod request;

#[cfg(feature = "tokio")]
pub mod request_tokio;

pub mod client;
pub mod date;
pub mod krauss;
pub mod method;
pub mod mock_stream;
pub mod percent;
pub mod response;
pub mod status;
