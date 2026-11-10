# Architecture

## Native interface

`src/app.rs` implements the native calendar workspace with a persistent mailbox list, in-place calendar selection, task tabs and centered modal editors. The tenant connection control is in the top bar. `src/presentation.rs` translates backend concepts into user-facing labels and permission roles. Directory pickers filter people and groups as the user types. Collection and sharing-domain controls construct structured values internally rather than asking for JSON.

`src/catalog.rs` models command metadata and request construction. `data/commands.json` is the offline reference baseline and filters documented on-premises-only parameters. `data/source-calendar-cmdlets.json` is the source coverage fixture. Live session metadata determines command/parameter availability. The broad Set-Mailbox and Set-OrganizationConfig operations show calendar-related settings rather than unrelated administration features.

The interface never presents cmdlet names, parameter counts, raw type names, command previews or WhatIf switches. PowerShell metadata and source coverage are developer documentation, not the user workflow. An interface regression test checks the access dialog for leaked command terminology.

## Delegated Exchange broker

`src/bridge.rs` starts one PowerShell 7 process with `-NoProfile -NonInteractive`. `src/powershell.rs` contains the fixed embedded broker, passed using UTF-16LE `-EncodedCommand`. This option transports the fixed program; it is not encryption. Request payloads, including any optional remote-service credentials, travel on stdin as JSON and are not placed in command-line arguments or temporary files.

Each JSON request has an ID, an operation and a payload. Responses use a distinct prefix on stdout. A separate reader consumes stderr and session messages, including device-code instructions. Only catalog commands classified for Exchange Online are allowlisted. The broker uses `Get-Command` and splatted parameter dictionaries; it never uses `Invoke-Expression` and does not offer an arbitrary shell console.

One operation runs at a time. A UI queue sequences connection, mailbox loading, schema discovery and commands while pipe readers run off the UI thread. No HTTP listener, localhost callback server or application database is used.

Browser/device authentication uses Microsoft's existing `Connect-ExchangeOnline` behavior, not a developer-created Entra application. The broker adds `-DelegatedOrganization` only when GDAP mode is selected. Each organization uses an isolated logical session; disconnect clears session-specific data, schemas and result buffers. Closing stdin requests a normal Exchange disconnect; a bounded shutdown kills a stuck process if necessary. Stop session terminates the local worker and does not promise to cancel a server operation already submitted.

## Mutations and results

Editors load current values and track changes. Unchanged optional fields are omitted; explicit false is distinct from omission. Resetting a setting submits null. The selected mailbox/calendar supplies the backend identity automatically. Directory selections serialize stable addresses, permission roles use friendly labels, and delegate dependencies are normalized before submission. Baseline data is scoped to the active mailbox/calendar; stale reads cannot replace another mailbox's settings.

The broker converts switches, booleans, dates and any required secure values, then lets PowerShell's native parameter binder enforce Exchange types and constraints. Its Confirm handling avoids hidden terminal prompts. Ordinary edits save directly. Destructive changes use a readable confirmation; operations that support PreviewOnly expose a Preview changes button.

Activity and report data are kept in memory. Results are formatted as a table, with an explicit copy-to-clipboard action. Connection profiles contain only account hints and organization names/domains. The Microsoft module owns its authentication cache. The application does not persist credentials or mailbox data.

## Extension points

For a new calendar-related cmdlet: confirm Exchange Online applicability, add reference metadata and a workflow title to the catalog, update the source inventory when relevant, add it to coverage documentation, and validate live schema/execution against a test tenant. The broker's allowlist derives from the catalog automatically. The broad mailbox/organization cmdlets are filtered to their calendar-related fields in the interface.

This architecture is portable across the three native targets. Only macOS can be runtime-tested on the development host; the included CI matrix builds other targets natively. Packaging creates an unsigned macOS bundle and portable Windows/Linux binaries. Runtime dependencies remain external rather than being silently installed or bundled.
