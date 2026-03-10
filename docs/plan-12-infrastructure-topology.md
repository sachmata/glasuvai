## 12. INFRASTRUCTURE TOPOLOGY

```
                            ┌──────────────┐
                            │   VOTERS     │
                            │  (browsers   │
                            │   + apps)    │
                            └──────┬───────┘
                                   │ HTTPS
                            ┌──────▼───────┐
                            │  SOVEREIGN   │  Static web app
                            │  EDGE CACHE  │  (HTML+JS+WASM)
                            │  (BG state   │
                            │   data ctrs  │  Domestic: Sofia, Plovdiv,
                            │   + EU PoPs  │  Varna, Burgas edge nodes
                            │   for dias-  │  Diaspora: EU PoPs (Frankfurt,
                            │   pora)      │  Amsterdam, London, Chicago)
                            └──────┬───────┘
                                   │
                    ┌──────────────┼──────────────┐
                    │              │               │
             ┌──────▼──────┐ ┌────▼─────┐  ┌──────▼──────┐
             │  IDENTITY   │ │ VOTING   │  │  BULLETIN   │
             │  PROVIDER   │ │ SERVER   │  │  BOARD      │
             │  CLUSTER    │ │ CLUSTER  │  │  PRIMARY    │
             │             │ │          │  │             │
             │ (3+ nodes   │ │(3+ nodes │  │(3+ nodes    │
             │  behind LB) │ │ behind LB│  │ + mirrors)  │
             └─────────────┘ └──────────┘  └──────┬──────┘
                                                   │ WebSocket
                                           ┌───────┴────────┐
                                           │   BB MIRRORS    │
                                           │   (parties,     │
                                           │    NGOs, press) │
                                           └────────────────┘
```

Hosting: All backend services hosted on Bulgarian sovereign infrastructure (state data
centers operated by "Информационно обслужване" or equivalent state IT entity).
Static client served from edge cache nodes in Bulgarian data centers for domestic voters
and EU points of presence for diaspora. NO foreign commercial CDN (Cloudflare, AWS
CloudFront, etc.) in the critical path — a foreign entity must never control what code
voters receive. Mirrors run on each observer's own infrastructure.
Edge cache integrity: Each edge node serves pre-signed static assets. The signing key
is held by ЦИК. Voters and mirrors independently verify asset signatures match the
published build hash (Section 7.2). A compromised edge node cannot serve modified
code without detection.
---

### 12.1 DIASPORA VOTING — MIR 32
Bulgarian citizens residing abroad vote in MIR 32 (Чужбина) under different rules than domestic MIRs. The online voting system must fully support this.
#### 12.1.1 Ballot Structure
MIR 32 has its own registered party lists and candidates, distinct from domestic MIRs.
The ballot matrix follows the same encoding (Section 3.1) but uses MIR 32 party/candidate lists.
The Identity Provider uses a dedicated MIR 32 signing key (the 32nd key).
#### 12.1.2 Eligibility & Identity Verification
Diaspora voters are registered in the voter roll for MIR 32 based on:
- Prior registration at a Bulgarian consulate/embassy, OR
- Registration via the ЦИК online portal (existing process, with QES or consular verification)
Identity code distribution for diaspora:
- Path A (eID): Diaspora voters with valid Bulgarian QES certificates authenticate
  online and receive their identity code electronically (same as domestic Path A)
- Path B (Consular): Voters registered at a consulate visit in person, present ID,
  and receive their code via a voter-facing device at the consular desk (same
  on-demand generation protocol as domestic municipal distribution, Section 11)
#### 12.1.3 Time Zone Handling
Online voting for MIR 32 follows Bulgarian time (EET/EEST):
- Voting opens and closes at the same absolute time as domestic voting
- The web client displays both local time and Bulgarian time for diaspora voters
- Voters in western time zones (Americas) effectively have a longer "evening" window;
  voters in eastern time zones (Asia/Australia) have an early-morning start
- In-person override at consular polling stations follows the local opening hours
  of each consular station (set by ЦИК per consulate, typically 07:00-20:00 local)
#### 12.1.4 In-Person Override for Diaspora
Diaspora voters who voted online can override at their registered consular polling station.
The override protocol is identical to domestic (Section 4.4), with the consular election
commission performing the same role as the domestic polling station commission.
The "online voter" register is synchronized to all consular polling stations.
