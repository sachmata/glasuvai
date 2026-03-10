//! Embeds TOML ballot data at compile time and provides typed accessors.
//!
//! The active feature flag determines which election's data is compiled in.
//! Any change to the data files triggers a recompile (enforced by `build.rs`).

use serde::Deserialize;

use super::types::*;

// ── Embedded TOML data, gated by feature flag ─────────────────────────────

#[cfg(feature = "bg-na51-2024")]
mod embedded {
    pub const ELECTION_TOML: &str =
        include_str!("../../../../data/elections/bg-na51-2024/election.toml");
    pub const MIRS_TOML: &str = include_str!("../../../../data/elections/bg-na51-2024/mirs.toml");
    pub const PARTIES_TOML: &str =
        include_str!("../../../../data/elections/bg-na51-2024/parties.toml");
    pub const MIR22_TOML: &str =
        include_str!("../../../../data/elections/bg-na51-2024/candidates/mir-22.toml");
    pub const MIR23_TOML: &str =
        include_str!("../../../../data/elections/bg-na51-2024/candidates/mir-23.toml");
    pub const MIR24_TOML: &str =
        include_str!("../../../../data/elections/bg-na51-2024/candidates/mir-24.toml");
}

#[cfg(feature = "bg-na51-2024")]
use embedded::*;

/// SHA-256 hex digest of all embedded data files, computed at build time.
/// Used for anti-tampering verification — the verifier and bulletin board
/// can confirm that the election data matches the published digest.
pub const DATA_INTEGRITY_DIGEST: &str = env!("GLASUVAI_DATA_SHA256");

// ── Helper wrapper types for TOML deserialization ─────────────────────────

#[derive(Deserialize)]
struct MirFile {
    mir: Vec<Mir>,
}

#[derive(Deserialize)]
struct PartyFile {
    party: Vec<Party>,
}

// ── Public accessors ──────────────────────────────────────────────────────

/// Parse the embedded election configuration.
pub fn election_config() -> ElectionConfig {
    toml::from_str(ELECTION_TOML).expect("embedded election.toml is valid")
}

/// Parse the embedded MIR table.
pub fn mirs() -> Vec<Mir> {
    let f: MirFile = toml::from_str(MIRS_TOML).expect("embedded mirs.toml is valid");
    f.mir
}

/// Parse the embedded party table.
pub fn parties() -> Vec<Party> {
    let f: PartyFile = toml::from_str(PARTIES_TOML).expect("embedded parties.toml is valid");
    f.party
}

/// Parse the embedded candidate data for MIR 22.
pub fn candidates_mir22() -> MirCandidates {
    toml::from_str(MIR22_TOML).expect("embedded mir-22.toml is valid")
}

/// Parse the embedded candidate data for MIR 23.
pub fn candidates_mir23() -> MirCandidates {
    toml::from_str(MIR23_TOML).expect("embedded mir-23.toml is valid")
}

/// Parse the embedded candidate data for MIR 24.
pub fn candidates_mir24() -> MirCandidates {
    toml::from_str(MIR24_TOML).expect("embedded mir-24.toml is valid")
}

/// Return the list of MIR IDs for which candidate data is available.
pub fn available_mir_ids() -> &'static [u32] {
    &[22, 23, 24]
}

/// Build a complete [`BallotSpec`] for a MIR by combining party + candidate data.
///
/// # Panics
///
/// Panics if candidate data is not available for the given `mir_id`.
pub fn ballot_spec(mir_id: u32) -> BallotSpec {
    let all_parties = parties();
    let mir_candidates = match mir_id {
        22 => candidates_mir22(),
        23 => candidates_mir23(),
        24 => candidates_mir24(),
        _ => panic!("candidate data not available for MIR {mir_id}"),
    };

    let registered_party_nums: Vec<u32> = mir_candidates
        .party_lists
        .iter()
        .map(|pl| pl.party_number)
        .collect();

    let mir_parties: Vec<Party> = all_parties
        .into_iter()
        .filter(|p| registered_party_nums.contains(&p.number))
        .collect();

    let max_candidates = mir_candidates
        .party_lists
        .iter()
        .map(|pl| pl.candidates.len() as u32)
        .max()
        .unwrap_or(0);

    BallotSpec {
        mir_id,
        parties: mir_parties,
        candidates: mir_candidates.party_lists,
        max_candidates,
    }
}
