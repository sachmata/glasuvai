## 16. ACCESSIBILITY & INCLUSIVITY

### 16.1 Web Accessibility (WCAG 2.1 AA Compliance)
The web client must meet WCAG 2.1 Level AA as a minimum:
- Full keyboard navigation (no mouse required)
- Screen reader support (ARIA labels, semantic HTML) — tested with NVDA, JAWS, VoiceOver
- High-contrast mode and configurable font sizes
- No time limits on ballot construction (only the election window itself is timed)
- Clear error messages with recovery instructions
- No CAPTCHA (token-based authentication replaces bot prevention)

### 16.2 Language Support
- Primary: Bulgarian (български)
- Secondary: Turkish (türkçe) — significant minority, required under anti-discrimination law
- Tertiary: English — for diaspora voters
- All UI strings externalized; additional languages can be added without code changes

### 16.3 Assisted Voting
- Voters with disabilities may be assisted by a person of their choice (existing Electoral Code right)
- The system logs that assisted voting mode was used (for audit) but not the assistant's identity
- Audio ballot option: screen reader reads party names and candidates; voter confirms by keyboard

### 16.4 Low Digital Literacy Accommodation
- Municipal offices offer supervised kiosks where voters can cast online votes with staff assistance
  (staff assists with device operation but CANNOT see the ballot — privacy screen + voter confirms alone)
- Printed step-by-step guides available at municipal offices and distributed with code receipts
- Video tutorials on glasuvai.bg in Bulgarian and Turkish
- Phone helpline (non-technical) for authentication and navigation issues

### 16.5 Device Requirements
- Minimum: Any modern browser (Chrome/Firefox/Safari/Edge, last 2 major versions)
- Mobile: Android 8+ / iOS 14+
- No app installation required for web voting (progressive web app fallback)
- Estimated data usage per vote: ~500 KB (with Rust WASM crypto module)
