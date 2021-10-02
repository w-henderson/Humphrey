use crate::fcgi::params::Params;
use crate::fcgi::record::FcgiRecord;
use crate::fcgi::types::FcgiType;

use std::collections::HashMap;

/// Represents an FCGI request.
pub struct FcgiRequest<'a> {
    pub params: HashMap<String, String>,
    pub content: &'a [u8],
    pub keep_alive: bool,
}

impl<'a> FcgiRequest<'a> {
    /// Create a new FCGI request with the given parameters.
    pub fn new(params: HashMap<String, String>, content: &'a [u8], keep_alive: bool) -> Self {
        Self {
            params,
            content,
            keep_alive,
        }
    }

    /// Encode an FCGI request into bytes to send.
    pub fn encode(&self) -> Vec<u8> {
        let request_id: u16 = 0;
        let mut request: Vec<u8> = Vec::new();

        // Add the begin request record
        let begin_record: Vec<u8> = FcgiRecord::begin_record(request_id, self.keep_alive).into();
        request.extend(begin_record);

        // Add the params record if it exists, then add an empty one
        let params: Vec<u8> = self.params.encode();
        if !params.is_empty() {
            let params_record: Vec<u8> =
                FcgiRecord::new(FcgiType::Params, &params, request_id).into();
            request.extend(params_record);
        }
        let blank_params_record: Vec<u8> =
            FcgiRecord::new(FcgiType::Params, &[], request_id).into();
        request.extend(blank_params_record);

        // Add the content record if it exists, then add an empty one
        if !self.content.is_empty() {
            let content_record: Vec<u8> =
                FcgiRecord::new(FcgiType::Stdin, self.content, request_id).into();
            request.extend(content_record);
        }
        let blank_content_record: Vec<u8> =
            FcgiRecord::new(FcgiType::Stdin, &[], request_id).into();
        request.extend(blank_content_record);

        request
    }
}
