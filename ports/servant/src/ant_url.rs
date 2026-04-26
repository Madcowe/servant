use url::{Url, ParseError};
use hex::FromHexError;

#[derive(Debug)]
pub enum AntUrlError {
    InvalidScheme,
    InvalidAddressLength,
    InvalidHexAddress(FromHexError),
    UrlParseError(ParseError),
}

pub struct AntUrl {
    pub address: [u8; 32],
    pub sub_path: Option<String>,
}

impl AntUrl {
    pub fn parse(url: &Url) -> Result<Self, AntUrlError> {
        if url.scheme() != "ant" {
            return Err(AntUrlError::InvalidScheme);
        }

        let host = url.host_str().ok_or(AntUrlError::InvalidAddressLength)?;
        
        if host.len() != 64 {
            return Err(AntUrlError::InvalidAddressLength);
        }

        let mut address = [0u8; 32];
        hex::decode_to_slice(host, &mut address)
            .map_err(AntUrlError::InvalidHexAddress)?;

        let path = url.path();
        let sub_path = if path.is_empty() || path == "/" {
            None
        } else {
            Some(path.trim_start_matches('/').to_string())
        };

        Ok(Self {
            address,
            sub_path,
        })
    }
}
