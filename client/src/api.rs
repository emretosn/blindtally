use reqwest::blocking::{Client, RequestBuilder, Response};
use tfhe::{CompressedServerKey, FheUint32};

use blindtally_core::election::Ballot;
use blindtally_core::io;

use crate::error::ClientError;

/// A thin typed wrapper around the server's HTTP API. Every body is bincode.
pub struct Api {
    http: Client,
    base_url: String,
}

impl Api {
    pub fn new(base_url: &str) -> Result<Self, ClientError> {
        // A tally costs seconds of CPU per ballot, far beyond reqwest's
        // default 30 s timeout, so waiting is left unbounded.
        let http = Client::builder().timeout(None).build()?;
        Ok(Api {
            http,
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }

    pub fn upload_key(&self, key: &CompressedServerKey) -> Result<(), ClientError> {
        let body = io::to_bytes(key)?;
        self.send(self.http.post(self.url("/keys")).body(body))?;
        Ok(())
    }

    pub fn cast_ballot(&self, ballot: &Ballot) -> Result<(), ClientError> {
        let body = io::to_bytes(ballot)?;
        self.send(self.http.post(self.url("/ballots")).body(body))?;
        Ok(())
    }

    /// Returns the server's summary line, e.g. "tallied 3 ballots".
    pub fn tally(&self) -> Result<String, ClientError> {
        Ok(self.send(self.http.post(self.url("/tally")))?.text()?)
    }

    pub fn results(&self) -> Result<Vec<FheUint32>, ClientError> {
        let bytes = self.send(self.http.get(self.url("/results")))?.bytes()?;
        Ok(io::from_bytes(&bytes)?)
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    /// Sends the request and turns non-2xx replies into `ClientError::Server`,
    /// carrying the server's plain-text error message.
    fn send(&self, request: RequestBuilder) -> Result<Response, ClientError> {
        let response = request.send()?;
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }
        let message = response.text().unwrap_or_default();
        Err(ClientError::Server { status, message })
    }
}
