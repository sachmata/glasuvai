//! Admin CLI for validating and exporting election data.
//!
//! Usage:
//!   cargo run -p glasuvai-admin                # print election summary
//!   cargo run -p glasuvai-admin -- --mir 23    # export MIR 23 ballot spec as JSON

use glasuvai_election::election::{data, validate};

fn main() {
    let config = data::election_config();
    let mirs = data::mirs();
    let parties = data::parties();

    // Validate MIR seat totals
    validate::validate_mir_seats(&mirs, config.total_seats).expect("MIR seat validation failed");

    println!("Election: {} ({})", config.name, config.date);
    println!("Data integrity: {}", data::DATA_INTEGRITY_DIGEST);
    println!("MIRs: {}, Parties: {}", mirs.len(), parties.len());
    println!(
        "Available candidate data: MIRs {:?}",
        data::available_mir_ids()
    );

    // --mir N → export ballot spec as JSON
    let args: Vec<String> = std::env::args().collect();
    if let Some(mir_arg) = args.iter().position(|a| a == "--mir") {
        let mir_id: u32 = args
            .get(mir_arg + 1)
            .expect("--mir requires a MIR number")
            .parse()
            .expect("invalid MIR number");

        let spec = data::ballot_spec(mir_id);
        validate::validate_ballot_spec(&spec).expect("ballot spec validation failed");

        let json = serde_json::to_string_pretty(&spec).expect("JSON serialisation failed");
        println!("{json}");
    }
}
