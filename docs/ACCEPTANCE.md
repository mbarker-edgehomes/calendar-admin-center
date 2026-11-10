# Tenant acceptance and release checklist

This repository is not production-accepted until these checks are completed with authorized test mailboxes. Do not infer live success from offline form coverage.

- [ ] Launch native builds on macOS, Windows and Linux; verify resizing, text, clipboard and browser sign-in.
- [ ] Check PowerShell/module discovery and actionable errors with each prerequisite missing.
- [ ] Authenticate using browser/MFA and, where allowed, device code; exercise Conditional Access rejection and expiry.
- [ ] Confirm the loaded organization and RBAC-limited command/parameter availability.
- [ ] Verify directory enumeration for user, shared, room and equipment mailboxes, and a large tenant.
- [ ] Discover localized primary and secondary calendars and use their actual folder identities.
- [ ] Read current calendar permissions, grant/edit/remove test permissions, verify Default/Anonymous and delegate/private-item combinations.
- [ ] Exercise each calendar appearance, regional, publishing and events-from-email setting supported in the tenant.
- [ ] Read and modify resource processing rules, delegates, booking policy, work hours, resource metadata and place details.
- [ ] Inspect diagnostic logs by subject, dates and IDs and verify table/clipboard output. Do not equate diagnostic log records with a complete agenda.
- [ ] Preview cancellation and time-zone migration, then validate test-only execution, resulting changes and attendee notices.
- [ ] Validate organization sharing, free/busy availability and relationships with suitable test organizations.
- [ ] For every active command in COMMAND_COVERAGE.md, verify at least one read or preview and each supported parameter-set binding. Use isolated test resources for writes.
- [ ] Validate searchable people/group pickers, duplicate display names, collection controls, explicit false/reset behavior, dates and backend range errors.
- [ ] Connect through an existing GDAP grant and test insufficient/missing grants. Disconnect, switch customers and confirm mailbox/schema/result isolation.
- [ ] Confirm shutdown/disconnect, cancellation during sign-in, network failure and no accidental automatic retries of writes.
- [ ] Review copied diagnostic data and local profile storage for organizational requirements.
- [ ] Sign and notarize release packages as appropriate; verify package integrity and required external runtime versions.

Record the tested OS, PowerShell and Exchange module versions, tenant role setup, date and result for each command. Never commit production mailbox data, authentication material or diagnostic exports to this repository.
