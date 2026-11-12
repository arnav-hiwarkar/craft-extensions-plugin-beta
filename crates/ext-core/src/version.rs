//! Versions: plain `major.minor.patch` semver for packages (no pre-release tags in the beta), and
//! simple requirements (`1.2.3`, `^1.2`, `=1.2.3`, `>=1.2.0`, `*`) for dependencies and pins.

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionError;

impl fmt::Display for VersionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a version looks like 1.2.3")
    }
}

impl std::error::Error for VersionError {}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    pub fn parse(text: &str) -> Result<Self, VersionError> {
        let mut parts = text.split('.');
        let mut next = || -> Result<u32, VersionError> {
            let part = parts.next().ok_or(VersionError)?;
            let plain = !part.is_empty() && part.len() <= 10 && part.bytes().all(|b| b.is_ascii_digit());
            if !plain || (part.len() > 1 && part.starts_with('0')) {
                return Err(VersionError);
            }
            part.parse::<u32>().map_err(|_| VersionError)
        };
        let version = Self::new(next()?, next()?, next()?);
        if parts.next().is_some() {
            return Err(VersionError);
        }
        Ok(version)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Serialize for Version {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let text = String::deserialize(d)?;
        Version::parse(&text).map_err(serde::de::Error::custom)
    }
}

/// A version requirement. `^1.2` (and a bare `1.2.3`) means "same major (or same minor while the
/// major is 0), at least this", which is what semver promises is compatible.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VersionReq {
    Any,
    Exact(Version),
    AtLeast(Version),
    Caret(Version),
}

impl VersionReq {
    pub fn parse(text: &str) -> Result<Self, VersionError> {
        let text = text.trim();
        if text == "*" {
            return Ok(Self::Any);
        }
        let padded = |t: &str| -> Result<Version, VersionError> {
            match t.split('.').count() {
                1 => Version::parse(&format!("{t}.0.0")),
                2 => Version::parse(&format!("{t}.0")),
                _ => Version::parse(t),
            }
        };
        if let Some(rest) = text.strip_prefix('=') {
            return Ok(Self::Exact(Version::parse(rest.trim())?));
        }
        if let Some(rest) = text.strip_prefix(">=") {
            return Ok(Self::AtLeast(padded(rest.trim())?));
        }
        let rest = text.strip_prefix('^').unwrap_or(text);
        Ok(Self::Caret(padded(rest.trim())?))
    }

    pub fn matches(&self, v: Version) -> bool {
        match *self {
            Self::Any => true,
            Self::Exact(e) => v == e,
            Self::AtLeast(m) => v >= m,
            Self::Caret(m) => v >= m && if m.major > 0 { v.major == m.major } else { v.major == 0 && v.minor == m.minor },
        }
    }
}

impl fmt::Display for VersionReq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Any => f.write_str("*"),
            Self::Exact(v) => write!(f, "={v}"),
            Self::AtLeast(v) => write!(f, ">={v}"),
            Self::Caret(v) => write!(f, "^{v}"),
        }
    }
}

impl Serialize for VersionReq {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for VersionReq {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let text = String::deserialize(d)?;
        VersionReq::parse(&text).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_orders() {
        assert_eq!(Version::parse("1.2.3"), Ok(Version::new(1, 2, 3)));
        assert!(Version::parse("1.10.0").unwrap() > Version::parse("1.9.9").unwrap());
        assert!(Version::parse("2.0.0").unwrap() > Version::parse("1.99.99").unwrap());
        assert_eq!(Version::new(1, 2, 3).to_string(), "1.2.3");
    }

    #[test]
    fn refuses_everything_else() {
        for bad in [
            "",
            "1",
            "1.2",
            "1.2.3.4",
            "01.2.3",
            "1.02.3",
            "1.2.03",
            "v1.2.3",
            "1.2.3-beta",
            "1.2.3+x",
            "-1.2.3",
            "1. 2.3",
            " 1.2.3",
            "1.2.x",
            "4294967296.0.0",
            "1.2.3\n",
        ] {
            assert_eq!(Version::parse(bad), Err(VersionError), "{bad:?}");
        }
        assert_eq!(Version::parse("0.0.0"), Ok(Version::new(0, 0, 0)));
        assert_eq!(Version::parse("4294967295.0.0"), Ok(Version::new(u32::MAX, 0, 0)));
    }

    #[test]
    fn requirements() {
        let v = |t| Version::parse(t).unwrap();
        let r = |t| VersionReq::parse(t).unwrap();
        assert!(r("^1.2").matches(v("1.9.0")) && !r("^1.2").matches(v("2.0.0")) && !r("^1.2").matches(v("1.1.9")));
        assert!(r("1.2.3").matches(v("1.2.4")));
        assert!(r("^0.3").matches(v("0.3.9")) && !r("^0.3").matches(v("0.4.0")));
        assert!(r("=1.0.0").matches(v("1.0.0")) && !r("=1.0.0").matches(v("1.0.1")));
        assert!(r(">=1.0").matches(v("7.0.0")) && r("*").matches(v("0.0.1")));
        assert!(VersionReq::parse("^x").is_err());
    }

    #[test]
    fn serde_round_trip() {
        let v: Version = serde_json::from_str("\"3.1.4\"").unwrap();
        assert_eq!(v, Version::new(3, 1, 4));
        assert_eq!(serde_json::to_string(&v).unwrap(), "\"3.1.4\"");
        assert!(serde_json::from_str::<Version>("\"3.1\"").is_err());
        assert!(serde_json::from_str::<Version>("3").is_err());
    }
}
