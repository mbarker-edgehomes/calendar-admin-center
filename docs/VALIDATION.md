# Validation record

Date: November 10, 2026.

Verified on macOS ARM64 with Rust 1.90.0, PowerShell 7.6.6 and ExchangeOnlineManagement 3.9.0 installed:

- Native optimized macOS build completed.
- Formatting and strict Clippy checks passed.
- 15 standard tests passed: reference inventory, exclusions, parameter handling, secret redaction, calendar targeting, dependent delegate options, change tracking, stale mailbox response isolation, tenant cleanup, operation routing, interface terminology and Windows command-line size.
- The additional real-PowerShell broker test passed using a stub Exchange service. It checks startup, time zone discovery, mailbox/recipient/calendar retrieval, live parameter metadata, booleans/arrays, rejected parameters, command allowlisting and hostile strings remaining data. It does not authenticate to Exchange Online.
- Native interface inspected in demo mode. Verified mailbox/calendar workspace, readable top bar, access dialog, typing `meg` to filter the people dropdown, and choosing the returned Megan Bowen entry.
- Installed PowerShell and Exchange module discovery verified without signing in.

Not verified:

- Live Microsoft 365 authentication, RBAC behavior, tenant mutations or GDAP grants.
- Windows/Linux compilation and runtime on actual machines. CI workflows are included but have not been run on GitHub.
- Production scale, signing, notarization or installers.

The source reference classifies 47 active Exchange Online operations and five on-premises/discontinued exclusions. Workflow coverage is not a substitute for the tenant acceptance checks in ACCEPTANCE.md.
