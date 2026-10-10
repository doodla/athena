//! Ids for jobs, interventions, restarts, and resolutions (D30).
//!
//! A [`RecordId`] is `YYYYMMDD-HHMMSS-xxxx`: UTC time, then four Crockford base32 characters from 20 random
//! bits. Ids sort by time, are unique without coordination, and give their month shard for free. The job id
//! `bootstrap` is reserved for the job that built v0.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Crockford's base32 alphabet, lowercase, without i, l, o, and u.
const CROCKFORD: &[u8; 32] = b"0123456789abcdefghjkmnpqrstvwxyz";

/// Why a string is not a valid id.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{input:?} is not a valid Athena id: {reason}")]
pub struct IdError {
    pub input: String,
    pub reason: &'static str,
}

/// A time-sortable record id, such as `20261010-153012-k7q2`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RecordId(String);

impl RecordId {
    /// Mints a new id from the given time and fresh randomness.
    pub fn generate(now: jiff::Timestamp) -> Self {
        let mut bytes = [0u8; 4];
        getrandom::fill(&mut bytes).expect("the operating system provides randomness");
        let bits = u32::from_le_bytes(bytes) & 0x000F_FFFF;
        Self::from_parts(now, bits)
    }

    /// Builds an id from a time and 20 bits of entropy. Deterministic; used by tests.
    pub fn from_parts(now: jiff::Timestamp, bits: u32) -> Self {
        let utc = now.to_zoned(jiff::tz::TimeZone::UTC);
        let mut suffix = String::with_capacity(4);
        for shift in [15, 10, 5, 0] {
            suffix.push(CROCKFORD[((bits >> shift) & 0x1F) as usize] as char);
        }
        RecordId(format!(
            "{:04}{:02}{:02}-{:02}{:02}{:02}-{}",
            utc.year(),
            utc.month(),
            utc.day(),
            utc.hour(),
            utc.minute(),
            utc.second(),
            suffix
        ))
    }

    /// Parses and validates an id.
    pub fn parse(input: &str) -> Result<Self, IdError> {
        let err = |reason| IdError {
            input: input.to_string(),
            reason,
        };
        let bytes = input.as_bytes();
        if bytes.len() != 20 {
            return Err(err("expected YYYYMMDD-HHMMSS-xxxx"));
        }
        if bytes[8] != b'-' || bytes[15] != b'-' {
            return Err(err("expected dashes after the date and the time"));
        }
        if !bytes[..8]
            .iter()
            .chain(&bytes[9..15])
            .all(u8::is_ascii_digit)
        {
            return Err(err("the date and time must be digits"));
        }
        if !bytes[16..].iter().all(|b| CROCKFORD.contains(b)) {
            return Err(err(
                "the suffix must be four lowercase Crockford base32 characters",
            ));
        }
        let month: u32 = input[4..6].parse().map_err(|_| err("bad month"))?;
        let day: u32 = input[6..8].parse().map_err(|_| err("bad day"))?;
        let hour: u32 = input[9..11].parse().map_err(|_| err("bad hour"))?;
        let minute: u32 = input[11..13].parse().map_err(|_| err("bad minute"))?;
        let second: u32 = input[13..15].parse().map_err(|_| err("bad second"))?;
        if !(1..=12).contains(&month)
            || !(1..=31).contains(&day)
            || hour > 23
            || minute > 59
            || second > 60
        {
            return Err(err("the date or time is out of range"));
        }
        Ok(RecordId(input.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The month shard, such as `2026-10`.
    pub fn month(&self) -> String {
        format!("{}-{}", &self.0[0..4], &self.0[4..6])
    }

    /// The last four characters, which users may type instead of the full id.
    pub fn suffix(&self) -> &str {
        &self.0[16..]
    }
}

impl fmt::Display for RecordId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for RecordId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for RecordId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        RecordId::parse(&text).map_err(serde::de::Error::custom)
    }
}

/// A job id: either a [`RecordId`] or the reserved `bootstrap`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JobId {
    Bootstrap,
    Id(RecordId),
}

impl JobId {
    pub const BOOTSTRAP: &'static str = "bootstrap";

    pub fn parse(input: &str) -> Result<Self, IdError> {
        if input == Self::BOOTSTRAP {
            Ok(JobId::Bootstrap)
        } else {
            RecordId::parse(input).map(JobId::Id)
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            JobId::Bootstrap => Self::BOOTSTRAP,
            JobId::Id(id) => id.as_str(),
        }
    }

    /// Whether this is a job made from a request (not the bootstrap).
    pub fn is_request(&self) -> bool {
        matches!(self, JobId::Id(_))
    }

    /// The directory that holds this job's records, relative to the repository root.
    pub fn dir(&self) -> String {
        match self {
            JobId::Bootstrap => "records/jobs/bootstrap".to_string(),
            JobId::Id(id) => format!("records/jobs/{}/{}", id.month(), id),
        }
    }
}

impl fmt::Display for JobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for JobId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for JobId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        JobId::parse(&text).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str) -> jiff::Timestamp {
        text.parse().unwrap()
    }

    #[test]
    fn ids_have_the_documented_shape() {
        let id = RecordId::from_parts(at("2026-10-10T15:30:12Z"), 0b00000_00001_00010_00011);
        assert_eq!(id.as_str(), "20261010-153012-0123");
        assert_eq!(id.month(), "2026-10");
        assert_eq!(id.suffix(), "0123");
        assert_eq!(RecordId::parse(id.as_str()).unwrap(), id);
    }

    #[test]
    fn generated_ids_parse_and_sort_by_time() {
        let early = RecordId::generate(at("2026-10-10T15:30:12Z"));
        let late = RecordId::generate(at("2026-10-10T15:30:13Z"));
        assert!(RecordId::parse(early.as_str()).is_ok());
        assert!(early < late);
    }

    #[test]
    fn malformed_ids_are_refused() {
        for bad in [
            "20261010-153012-k7q",
            "20261010_153012-k7q2",
            "20261310-153012-k7q2",
            "20261010-153012-K7Q2",
            "20261010-153012-k7qi",
            "bootstrap",
        ] {
            assert!(RecordId::parse(bad).is_err(), "{bad} should be refused");
        }
    }

    #[test]
    fn job_ids_include_the_reserved_bootstrap() {
        assert_eq!(JobId::parse("bootstrap").unwrap(), JobId::Bootstrap);
        assert_eq!(JobId::Bootstrap.dir(), "records/jobs/bootstrap");
        let job = JobId::parse("20261010-153012-k7q2").unwrap();
        assert_eq!(job.dir(), "records/jobs/2026-10/20261010-153012-k7q2");
        assert_eq!(
            serde_json::to_string(&job).unwrap(),
            "\"20261010-153012-k7q2\""
        );
    }
}
