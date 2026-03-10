//! Core election data types.
//!
//! All types derive [`Debug`], [`Clone`], and [`serde::Deserialize`] for
//! TOML parsing. Types use owned `String` fields because they are
//! deserialized from embedded TOML at runtime.

use serde::Deserialize;

/// Multi-member constituency (Многомандатен избирателен район — МИР).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "serde-serialize", derive(serde::Serialize))]
pub struct Mir {
    /// 1-31 for domestic constituencies.
    pub id: u32,
    /// Official Bulgarian name, e.g. "Благоевград".
    pub name: String,
    /// Latin transliteration, e.g. "Blagoevgrad".
    pub name_latin: String,
    /// Number of parliamentary seats allocated to this constituency.
    pub seats: u32,
}

/// Registered political party or electoral coalition.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "serde-serialize", derive(serde::Serialize))]
pub struct Party {
    /// Ballot number assigned by ЦИК lottery.
    pub number: u32,
    /// Official name in Bulgarian.
    pub name: String,
    /// Latin transliteration.
    pub name_latin: String,
    /// Short abbreviation used on ballots, e.g. "ГЕРБ-СДС".
    pub short: String,
    /// `true` if this entry is an electoral coalition.
    pub coalition: bool,
}

/// Candidate on a party list for a specific MIR.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "serde-serialize", derive(serde::Serialize))]
pub struct Candidate {
    /// 1-indexed position on the party list.
    pub position: u32,
    /// Given name in Bulgarian.
    pub first_name: String,
    /// Family name in Bulgarian.
    pub last_name: String,
}

/// A party's candidate list within a MIR.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "serde-serialize", derive(serde::Serialize))]
pub struct PartyList {
    /// References [`Party::number`].
    pub party_number: u32,
    /// Ordered list of candidates (position 1..=N).
    #[serde(rename = "candidate")]
    pub candidates: Vec<Candidate>,
}

/// Candidate file for a single MIR (maps to `candidates/mir-XX.toml`).
#[derive(Debug, Clone, Deserialize)]
pub struct MirCandidates {
    /// Which MIR this file describes.
    pub mir_id: u32,
    /// Party lists registered in this MIR.
    #[serde(rename = "party_list")]
    pub party_lists: Vec<PartyList>,
}

/// Complete ballot specification for a specific MIR (assembled at runtime).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialize", derive(serde::Serialize))]
pub struct BallotSpec {
    /// Which MIR this ballot covers.
    pub mir_id: u32,
    /// Parties registered in this MIR, ordered by ballot number.
    pub parties: Vec<Party>,
    /// Candidate lists for every party registered in this MIR.
    pub candidates: Vec<PartyList>,
    /// Maximum candidate list length across all parties in this MIR.
    pub max_candidates: u32,
}

/// Election-wide configuration parameters.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[cfg_attr(feature = "serde-serialize", derive(serde::Serialize))]
pub struct ElectionConfig {
    /// Stable identifier, e.g. "bg-na51-2024".
    pub election_id: String,
    /// Human-readable name in Bulgarian.
    pub name: String,
    /// ISO 8601 date string, e.g. "2024-10-27".
    pub date: String,
    /// Total number of MIRs.
    pub total_mirs: u32,
    /// National electoral threshold (4% -> 0.04).
    pub national_threshold: f64,
    /// Preference threshold within a party list (7% -> 0.07).
    pub preference_threshold: f64,
    /// Total seats in the National Assembly.
    pub total_seats: u32,
    /// Seat-allocation method identifier.
    pub seat_allocation: String,
}
