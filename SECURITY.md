# Security Policy

Q-Safe is a personal/portfolio project, not an operated service - there is no production deployment handling real user data today. That said, if you find a vulnerability in the code (crypto implementation, auth flow, or otherwise), please report it responsibly rather than opening a public issue.

## Reporting a vulnerability

Email **shauryagaur07@gmail.com** with:
- A description of the issue and its potential impact.
- Steps to reproduce (a minimal repro is ideal).
- Which component it affects (`host-server`, `common`, or `firmware`).

Please don't open a public GitHub issue for anything that could be actively exploitable until it's been addressed.

## Scope and known limitations

Before reporting, check [docs/HSM_VERIFICATION_STATUS.md](docs/HSM_VERIFICATION_STATUS.md) - several things are *known and already documented* as unfinished, not vulnerabilities:

- The RP2040 firmware doesn't exist yet; only the host-side driver and Mock HSM are implemented.
- Refresh-token rotation doesn't yet revoke the previous token (no server-side token store).
- The QKD/decoy-check handshake protocol (`handshake.rs`, `qkd.rs`) is implemented but not wired to any network endpoint.
- `tower_governor`'s rate limiter keys on raw peer IP, which is not proxy-aware; behind a load balancer or reverse proxy this becomes a shared bucket for all users behind it. Known, not yet addressed.

A genuinely new finding outside of what's listed above (or in the CHANGELOG's audit entries) is very welcome.
