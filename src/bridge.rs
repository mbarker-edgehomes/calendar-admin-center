use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

#[path = "powershell.rs"]
mod powershell;

#[derive(Debug, Deserialize)]
pub struct Response {
    pub id: u64,
    pub ok: bool,
    pub data: Value,
    pub message: String,
}
pub enum Event {
    Response(Response),
    Notice(String),
    Closed,
}

#[derive(Serialize)]
struct Request<'a> {
    id: u64,
    action: &'a str,
    payload: Value,
}

pub struct Bridge {
    child: Child,
    stdin: Option<ChildStdin>,
    pub events: Receiver<Event>,
    sequence: u64,
}

impl Bridge {
    pub fn start(executable: &str) -> Result<Self, String> {
        let allow = crate::catalog::load()
            .into_iter()
            .filter(|c| c.unavailable.is_none())
            .map(|c| format!("'{}'", c.name))
            .collect::<Vec<_>>()
            .join(",");
        let script = powershell::SCRIPT
            .replace("__ALLOWLIST__", &allow)
            .lines()
            .map(str::trim)
            .collect::<Vec<_>>()
            .join("\n");
        Self::spawn_script(executable, &script)
    }
    fn spawn_script(executable: &str, script: &str) -> Result<Self, String> {
        let encoded = STANDARD.encode(
            script
                .encode_utf16()
                .flat_map(u16::to_le_bytes)
                .collect::<Vec<_>>(),
        );
        let mut cmd = Command::new(executable);
        cmd.args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-OutputFormat",
            "Text",
            "-EncodedCommand",
            &encoded,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        let mut child=cmd.spawn().map_err(|e| format!("Cannot start PowerShell 7 ({executable}): {e}. Install PowerShell 7 and configure its full path under Connection."))?;
        let stdin = child.stdin.take();
        let stdout = child
            .stdout
            .take()
            .ok_or("PowerShell stdout is unavailable")?;
        let stderr = child
            .stderr
            .take()
            .ok_or("PowerShell stderr is unavailable")?;
        let (tx, events) = mpsc::channel();
        let errtx = tx.clone();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                if let Some((_, packet)) = line.split_once("@@CAC@@") {
                    match serde_json::from_str(packet) {
                        Ok(response) => {
                            let _ = tx.send(Event::Response(response));
                        }
                        Err(e) => {
                            let _ = tx.send(Event::Notice(format!(
                                "Broker response could not be decoded: {e}"
                            )));
                        }
                    }
                } else if !line.trim().is_empty() {
                    let _ = tx.send(Event::Notice(line));
                }
            }
            let _ = tx.send(Event::Closed);
        });
        thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                if !line.trim().is_empty() {
                    let _ = errtx.send(Event::Notice(line));
                }
            }
        });
        Ok(Self {
            child,
            stdin,
            events,
            sequence: 0,
        })
    }
    pub fn send(&mut self, action: &str, payload: Value) -> Result<u64, String> {
        self.sequence += 1;
        let packet = serde_json::to_vec(&Request {
            id: self.sequence,
            action,
            payload,
        })
        .map_err(|e| e.to_string())?;
        let stdin = self.stdin.as_mut().ok_or("PowerShell is closed")?;
        stdin
            .write_all(&packet)
            .and_then(|_| stdin.write_all(b"\n"))
            .and_then(|_| stdin.flush())
            .map_err(|e| e.to_string())?;
        Ok(self.sequence)
    }
    pub fn stop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.stdin.take();
    }
}

impl Drop for Bridge {
    fn drop(&mut self) {
        // EOF lets the broker disconnect normally; bound shutdown if authentication is pending.
        self.stdin.take();
        let start = Instant::now();
        while start.elapsed() < Duration::from_millis(1500) {
            if matches!(self.child.try_wait(), Ok(Some(_))) {
                return;
            }
            thread::sleep(Duration::from_millis(25));
        }
        self.stop();
    }
}

#[cfg(test)]
mod broker_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn encoded_broker_fits_windows_command_line_limit() {
        let allow = crate::catalog::load()
            .into_iter()
            .filter(|c| c.unavailable.is_none())
            .map(|c| format!("'{}'", c.name))
            .collect::<Vec<_>>()
            .join(",");
        let script = powershell::SCRIPT
            .replace("__ALLOWLIST__", &allow)
            .lines()
            .map(str::trim)
            .collect::<Vec<_>>()
            .join("\n");
        let bytes: Vec<_> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
        assert!(
            STANDARD.encode(bytes).len() + 1024 < 32767,
            "Broker exceeds Windows process argument limit"
        );
    }

    fn reply(bridge: &Bridge, id: u64) -> Response {
        loop {
            match bridge
                .events
                .recv_timeout(Duration::from_secs(20))
                .expect("PowerShell broker timed out")
            {
                Event::Response(r) if r.id == id => return r,
                Event::Closed => panic!("PowerShell broker closed unexpectedly"),
                _ => {}
            }
        }
    }

    const STUB: &str = r#"
function Import-Module { [CmdletBinding()] param($Name, $MinimumVersion) }
function Connect-ExchangeOnline { [CmdletBinding()] param($UserPrincipalName, $DelegatedOrganization, $ShowBanner, [switch]$Device, [switch]$DisableWAM) }
function Disconnect-ExchangeOnline { [CmdletBinding(SupportsShouldProcess)] param() }
function Get-ConnectionInformation { [PSCustomObject]@{UserPrincipalName='test@example.com';TenantID='test-tenant';Organization='test.example';State='Connected'} }
function Get-EXOMailbox { [CmdletBinding()] param($ResultSize,$Properties) [PSCustomObject]@{DisplayName='Test Room';PrimarySmtpAddress='room@example.com';RecipientTypeDetails='RoomMailbox';ExternalDirectoryObjectId='test-id'} }
function Get-EXORecipient { [CmdletBinding()] param($ResultSize,$Properties) Get-EXOMailbox }
function Get-EXOMailboxFolderStatistics { [CmdletBinding()] param($Identity,$FolderScope) [PSCustomObject]@{Name='Kalender';FolderPath='/Kalender';FolderType='Calendar';ItemsInFolder=2;FolderSize='100 KB'} }
function Set-CalendarProcessing {
    [CmdletBinding(SupportsShouldProcess,DefaultParameterSetName='Default')]
    param([Parameter(Mandatory,ParameterSetName='Default')][string]$Identity,
        [bool]$AllowConflicts,[ValidateSet('None','AutoAccept')][string]$AutomateProcessing,[string[]]$ResourceDelegates)
    if ($PSCmdlet.ShouldProcess($Identity)) { [PSCustomObject]@{Identity=$Identity;AllowConflicts=$AllowConflicts;Delegates=$ResourceDelegates} }
}
"#;

    #[test]
    #[ignore = "requires a local PowerShell 7 executable; uses stubs, never authenticates"]
    fn broker_roundtrip_schema_types_and_injection_isolation() {
        let executable = std::env::var("CAC_PWSH").unwrap_or_else(|_| {
            if cfg!(target_os = "macos") {
                "/opt/homebrew/bin/pwsh".into()
            } else {
                "pwsh".into()
            }
        });
        let program = format!(
            "{}\n{}",
            STUB,
            powershell::SCRIPT.replace(
                "__ALLOWLIST__",
                "'Set-CalendarProcessing','Get-EXOMailbox','Get-EXOMailboxFolderStatistics'"
            )
        );
        let mut bridge = Bridge::spawn_script(&executable, &program).unwrap();
        assert!(reply(&bridge, 0).ok);
        let id = bridge.send("zones", json!({})).unwrap();
        let zones = reply(&bridge, id);
        assert!(zones.ok, "{}", zones.message);
        assert!(!zones.data.as_array().unwrap().is_empty());
        let id = bridge
            .send(
                "invoke",
                json!({"command":"Set-CalendarProcessing","arguments":{}}),
            )
            .unwrap();
        assert!(!reply(&bridge, id).ok);
        let id = bridge
            .send(
                "connect",
                json!({"upn":"test@example.com","organization":"customer.example"}),
            )
            .unwrap();
        let connected = reply(&bridge, id);
        assert!(connected.ok, "{}", connected.message);
        assert!(connected.data["commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == "Set-CalendarProcessing"));
        let id = bridge.send("directory", json!({})).unwrap();
        let directory = reply(&bridge, id);
        assert!(directory.ok, "{}", directory.message);
        assert_eq!(directory.data[0]["PrimarySmtpAddress"], "room@example.com");
        let id = bridge.send("recipients", json!({})).unwrap();
        let recipients = reply(&bridge, id);
        assert!(recipients.ok, "{}", recipients.message);
        assert_eq!(recipients.data[0]["PrimarySmtpAddress"], "room@example.com");
        let id = bridge
            .send("calendars", json!({"mailbox":"room@example.com"}))
            .unwrap();
        let calendars = reply(&bridge, id);
        assert!(calendars.ok, "{}", calendars.message);
        assert_eq!(calendars.data[0]["FolderPath"], "/Kalender");
        let id = bridge
            .send("schema", json!({"command":"Set-CalendarProcessing"}))
            .unwrap();
        let schema = reply(&bridge, id);
        assert!(schema.ok, "{}", schema.message);
        let params = schema.data["parameters"].as_array().unwrap();
        let identity = params.iter().find(|v| v["name"] == "Identity").unwrap();
        assert_eq!(identity["mandatory_sets"], json!(["Default"]));
        let enum_field = params
            .iter()
            .find(|v| v["name"] == "AutomateProcessing")
            .unwrap();
        assert_eq!(enum_field["values"], json!(["None", "AutoAccept"]));
        let hostile = "'; throw 'INJECTED'; $(Get-Process) #";
        let id=bridge.send("invoke",json!({"command":"Set-CalendarProcessing","arguments":{"Identity":hostile,"AllowConflicts":false,"ResourceDelegates":["a@example.com","b@example.com"]}})).unwrap();
        let result = reply(&bridge, id);
        assert!(result.ok, "{}", result.message);
        assert_eq!(result.data[0]["Identity"], hostile);
        assert_eq!(result.data[0]["AllowConflicts"], false);
        assert_eq!(
            result.data[0]["Delegates"],
            json!(["a@example.com", "b@example.com"])
        );
        let id=bridge.send("invoke",json!({"command":"Set-CalendarProcessing","arguments":{"Identity":"room@example.com","NotAParameter":true}})).unwrap();
        assert!(!reply(&bridge, id).ok);
        let id = bridge
            .send("invoke", json!({"command":"Remove-Mailbox","arguments":{}}))
            .unwrap();
        assert!(!reply(&bridge, id).ok);
        let id = bridge.send("disconnect", json!({})).unwrap();
        assert!(reply(&bridge, id).ok);
    }
}
