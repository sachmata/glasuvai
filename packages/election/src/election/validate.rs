//! Validation functions for election data integrity.
//!
//! These functions check that embedded ballot data is internally consistent
//! (seat totals, candidate ordering, party references, etc.).

use super::types::{BallotSpec, Mir};

/// Checks that total seats across all MIRs equals the expected total.
pub fn validate_mir_seats(mirs: &[Mir], expected: u32) -> Result<(), String> {
    let total: u32 = mirs.iter().map(|m| m.seats).sum();
    if total != expected {
        return Err(format!("expected {expected} total seats, got {total}"));
    }
    Ok(())
}

/// Checks that a [`BallotSpec`] is internally consistent:
///
/// - At least one party is registered.
/// - Every party list references a party present in `spec.parties`.
/// - Candidate positions are sequential 1..=N within each list.
/// - No duplicate party numbers.
pub fn validate_ballot_spec(spec: &BallotSpec) -> Result<(), String> {
    if spec.parties.is_empty() {
        return Err("no parties registered".into());
    }

    // Every party list must reference a party present in spec.parties
    for pl in &spec.candidates {
        if !spec.parties.iter().any(|p| p.number == pl.party_number) {
            return Err(format!(
                "party list references unknown party {}",
                pl.party_number
            ));
        }

        // Positions must be sequential 1..=N
        for (i, c) in pl.candidates.iter().enumerate() {
            if c.position != (i as u32 + 1) {
                return Err(format!(
                    "party {} candidate at index {} has position {} (expected {})",
                    pl.party_number,
                    i,
                    c.position,
                    i + 1
                ));
            }
        }
    }

    // No duplicate party numbers
    let mut seen = std::collections::HashSet::new();
    for pl in &spec.candidates {
        if !seen.insert(pl.party_number) {
            return Err(format!(
                "duplicate party list for party {}",
                pl.party_number
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::election::data;

    #[test]
    fn mir_seats_sum_to_240() {
        let mirs = data::mirs();
        validate_mir_seats(&mirs, 240).unwrap();
    }

    #[test]
    fn mir22_ballot_spec_is_valid() {
        let spec = data::ballot_spec(22);
        validate_ballot_spec(&spec).unwrap();
    }

    #[test]
    fn mir23_ballot_spec_is_valid() {
        let spec = data::ballot_spec(23);
        validate_ballot_spec(&spec).unwrap();
    }

    #[test]
    fn mir24_ballot_spec_is_valid() {
        let spec = data::ballot_spec(24);
        validate_ballot_spec(&spec).unwrap();
    }

    #[test]
    fn mir_count_is_31() {
        let config = data::election_config();
        let mirs = data::mirs();
        assert_eq!(config.total_mirs, mirs.len() as u32);
        assert_eq!(mirs.len(), 31);
    }

    #[test]
    fn parties_have_unique_numbers() {
        let parties = data::parties();
        let mut seen = std::collections::HashSet::new();
        for p in &parties {
            assert!(seen.insert(p.number), "duplicate party number {}", p.number);
        }
    }

    #[test]
    fn data_integrity_digest_is_valid_hex() {
        let digest = data::DATA_INTEGRITY_DIGEST;
        assert_eq!(digest.len(), 64, "SHA-256 digest should be 64 hex chars");
        assert!(
            digest.chars().all(|c| c.is_ascii_hexdigit()),
            "digest should contain only hex characters"
        );
    }
}
