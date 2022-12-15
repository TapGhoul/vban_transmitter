use deku::prelude::*;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, DekuRead, DekuWrite)]
#[repr(transparent)]
pub struct StreamName([u8; 16]);

impl TryFrom<&str> for StreamName {
    type Error = DekuError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}

impl TryFrom<String> for StreamName {
    type Error = DekuError;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        if !name.is_ascii() {
            return Err(DekuError::InvalidParam(String::from("name is not ASCII")));
        }
        if name.len() > 16 {
            return Err(DekuError::InvalidParam(String::from(
                "name is longer than 16",
            )));
        }

        let mut buf = name.into_bytes();
        buf.resize(16, 0);

        let v = buf
            .try_into()
            .map_err(|_| DekuError::Unexpected(String::from("vec len != 16")))?;

        Ok(Self(v))
    }
}

impl TryInto<String> for StreamName {
    type Error = DekuError;

    fn try_into(self) -> Result<String, Self::Error> {
        String::from_utf8(self.0.into())
            .map_err(|_| DekuError::Parse(String::from("invalid utf-8 string")))
    }
}

impl Display for StreamName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = if let Some(idx) = self.0.iter().position(|e| *e == 0) {
            String::from_utf8_lossy(&self.0[0..idx])
        } else {
            String::from_utf8_lossy(&self.0)
        };

        f.write_str(s.as_ref())
    }
}
