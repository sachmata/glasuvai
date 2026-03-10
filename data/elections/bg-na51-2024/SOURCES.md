# Data Sources for bg-na51-2024

All election data in this directory is sourced from official registers and
verifiable public records for the 51st National Assembly elections held on
27 October 2024.

## Primary Sources

### ЦИК Official Results Portal
- **National results (opendata)**:
  https://results.cik.bg/pe202410_ks/opendata/export.zip
- **Hare-Niemeyer seat distribution methodology (per-MIR mandate counts)**:
  https://results.cik.bg/pe202410_ks/hnm.64.html

### ЦИК Open Data Export (raw data files)
The `export/` subdirectory contains raw semicolon-delimited files from the
CIK open data export archive. These are the authoritative source for:
- **`cik_parties.txt`** — National party/coalition list (28 entries)
- **`local_parties.txt`** — Parties registered per MIR
- **`local_candidates.txt`** — Candidate lists per MIR (names, positions, party numbers)
- **`readme.txt`** — CIK-provided format documentation

Candidate TOML files (`candidates/mir-*.toml`) are generated from
`local_candidates.txt`. Names are title-cased from the CIK ALL-CAPS format;
positions are renumbered sequentially where the CIK data has gaps from
deregistered candidates.

### Party Ballot Numbers
- **DW article (ballot number lottery results)**:
  https://www.dw.com/bg/eto-nomerata-na-partiite-i-koaliciite-v-buletinata-za-izborite-na-27-oktomvri/a-70323854

## Key Facts

| Parameter                | Value          |
|--------------------------|----------------|
| Election date            | 27 October 2024 |
| Assembly                 | 51st National Assembly |
| Total seats              | 240            |
| Multi-member constituencies (MIR) | 31   |
| Registered parties       | 19             |
| Registered coalitions    | 9              |
| Ballot numbers assigned  | 1–28 (with gaps) |
| Electoral threshold      | 4%             |
| Preference threshold     | 7%             |
| Seat allocation method   | Hare-Niemeyer  |
| Parties passing threshold| 8 (+ Velichie by Constitutional Court) |
