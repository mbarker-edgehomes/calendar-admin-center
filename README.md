# Calendar admin center

A native Rust desktop application for **Exchange Online calendar administration**, with a Microsoft 365 / Entra inspired interface. Runs on macOS, Windows and Linux. This is an independent project, not a Microsoft product.

## Features

- Searchable mailbox directory for people, shared mailboxes, rooms and equipment.
- An integrated calendar workspace: keep the mailbox list visible, switch calendars in place, and manage access, booking, preferences and sharing with tabs.
- Type-to-filter people and group pickers, plain-language permission levels, delegate controls and invitation options.
- Current settings loaded into normal controls; save only the fields you change. No command browser, raw parameter inputs, JSON entry or PowerShell switches in the normal interface.
- Microsoft 365 connections in the top bar. Single organization by default, with optional customer/GDAP connections and saved profiles.
- Meeting activity search, previewable meeting cancellation and event time-zone updates, plus organization sharing, availability and resource settings.
- Clearly labelled offline demo mode, accessible native widgets, result tables and copy-to-spreadsheet support.

The backend covers 47 Exchange Online operations. See [the developer command inventory](docs/COMMAND_COVERAGE.md) for the precise scope and exclusions.

## Authentication and architecture

**No custom app registration, client ID, client secret, certificate or app-only permissions.** The application starts a persistent PowerShell 7 child process and uses Microsoft's `Connect-ExchangeOnline` interactive authentication. Operations execute with the signed-in user's Exchange RBAC permissions. Microsoft handles MFA, Conditional Access, consent and token caching under the Exchange Online module's existing authentication flow.

The GUI, application logic and process broker are Rust (`eframe`/`egui`); there is no JavaScript, browser UI, .NET application or Tauri runtime. To expose real Exchange Online cmdlets, the Rust binary embeds a small fixed PowerShell broker program and requires Microsoft's PowerShell runtime and ExchangeOnlineManagement module. **This is not a dependency-free, all-Rust reimplementation of Microsoft's module.** See [architecture](docs/ARCHITECTURE.md).

## Prerequisites

1. [PowerShell 7.4 or newer](https://learn.microsoft.com/en-us/powershell/scripting/install/installing-powershell), accessible as `pwsh` or configured under Connection troubleshooting.
2. ExchangeOnlineManagement 3.7 or newer; 3.9+ recommended. Install in PowerShell:

   ```powershell
   Install-Module ExchangeOnlineManagement -Scope CurrentUser
   ```

3. A Microsoft 365 user with the Exchange roles needed for the operations you intend to perform. The application does not grant permissions or bypass mailbox access restrictions.
4. [Rust 1.90](https://rustup.rs/) with a native compiler/linker to build from source. A pinned toolchain and Cargo.lock are included.

On macOS, install Xcode Command Line Tools. On Windows, use the MSVC Rust toolchain and Visual Studio C++ Build Tools. On Debian/Ubuntu Linux, install:

```sh
sudo apt-get install build-essential pkg-config libx11-dev libxi-dev libxrandr-dev libxcursor-dev libxinerama-dev libgl1-mesa-dev libwayland-dev libxkbcommon-dev
```

## Build and run

```sh
cargo build --locked --release
cargo run --locked --release
```

Inspect the interface without authentication:

```sh
cargo run --locked --release -- --demo
```

Inspect the bundled coverage inventory without opening a window:

```sh
cargo run --locked -- --check-catalog
```

The release binary is `target/release/calendar-admin-center` (`.exe` on Windows). You can run it directly. The macOS packaging workflow also produces an unsigned `.app` bundle.

## Using the application

1. Select **Connect Microsoft 365** in the top bar. Enter an optional account hint, choose **Sign in**, and complete Microsoft's browser/MFA flow.
2. Search **Mailboxes** and select a person, shared mailbox or room. The directory stays alongside the calendar workspace.
3. Use the **Calendar** dropdown to switch between that mailbox's calendars without leaving the workspace.
4. Open **People & access** to add or change access. **Add people** opens a searchable directory dropdown; type part of a name or address, select the matching person/group and choose their access level.
5. Use **Preferences**, **Booking** (for resources), **Sharing**, or **Troubleshooting** for the corresponding task. Edit dialogs load current settings and submit only changes.
6. Destructive actions show a readable confirmation. Meeting cancellation and time-zone updates offer **Preview changes** before execution. Reports can be copied as a table for a spreadsheet.

## GDAP / multiple tenants

Single organization is the default at each launch. Open the connection control in the top bar, select **I'm managing a customer organization**, enter the customer's tenant domain, and sign in with your partner account. An existing GDAP relationship and appropriate Exchange roles are required; the app does not create grants or bypass permissions.

Use **Add organization** and **Save connection** to keep account hints and customer domains. Disconnect before switching organizations. Mailbox data, people, forms and results are cleared between sessions. Partner Center customer discovery is not included.

## Data and configuration

Connection profiles contain only names, user principal name hints and organization domains. They are saved only when you click **Save connection profiles**:

- macOS: `~/Library/Application Support/calendar-admin-center/connections.json`
- Windows: `%APPDATA%\calendar-admin-center\connections.json`
- Linux: `$XDG_CONFIG_HOME/calendar-admin-center/connections.json`, defaulting to `~/.config/...`

`CAC_CONFIG_DIR` overrides the configuration folder. `CAC_PWSH` overrides the PowerShell executable path. Credentials entered for optional remote availability-service parameters are not persisted. Activity and result data are held in memory. Reports are copied only when you choose Copy table. Microsoft may cache its own authentication tokens according to its module behavior.

## Scope and validation status

Exchange Online only. On-premises calendar repair/connectivity and retired SMS calendar notification cmdlets are excluded. The exact inventory is based on the Microsoft reference retrieved November 10, 2026; it is not an automatic guarantee of future cmdlets added by Microsoft.

**Calendar viewing means calendar folders, configuration, permissions and diagnostic-event records.** Exchange Online PowerShell does not offer arbitrary appointment CRUD or a complete Outlook event agenda. Diagnostics can contain multiple log records per meeting, are subject to retention, and must not be interpreted as a complete schedule. Those additional event capabilities would require a separate API/authentication design.

The project includes workflow, interface-language and coverage tests, real-PowerShell broker tests using a stub Exchange service, and CI workflows for all three operating systems. A successful build does not establish that every mutation works against a tenant. Live single-tenant/GDAP acceptance and Windows/Linux runtime checks remain required before a production release; see [the acceptance checklist](docs/ACCEPTANCE.md).

## Development

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

Optional broker tests use a local PowerShell runtime and a stub Exchange service without signing in:

```sh
cargo test --locked broker -- --ignored
```

See [contributing](CONTRIBUTING.md). CI checks formatting, linting, tests and release compilation on macOS, Windows and Ubuntu, and uploads native binaries. A tag matching `v*` runs the packaging workflow. It creates downloadable build artifacts; it does not automatically publish a GitHub release.

## Publish to GitHub

The directory is initialized as a local Git repository. Review the content, then create the first commit and attach your own empty repository:

```sh
git add .
git commit -m "Initial Calendar Admin Center implementation"
git remote add origin https://github.com/YOUR-ACCOUNT/calendar-admin-center.git
git push -u origin main
```

No GitHub repository or remote is created automatically. Code signing, notarization, installers and tenant acceptance should precede a public production release. Licensed under [MIT](LICENSE).
