# Contributing

Use Rust 1.90 and keep the lockfile committed. Run `cargo fmt --all --check`, `cargo clippy --locked --all-targets -- -D warnings`, and `cargo test --locked` before proposing changes. Use `--demo` for interface work without a tenant.

Keep the GUI in Rust and route Exchange operations through the fixed, allowlisted JSON broker. Never interpolate user values into executable code, log credentials, auto-retry mutations, or add application-only authentication as a fallback.

When adding commands, verify Exchange Online applicability against Microsoft's reference. Update `data/commands.json`, the reference inventory and command coverage documentation. Use live `Get-Command` metadata as the execution authority. Test field omission, explicit false, null clearing, parameter-set requirements and any sensitive-value handling.

Use authorized test mailboxes for live tests. Keep tenant data and diagnostic exports outside the repository. Describe which tests ran and distinguish form availability from live functional verification.
