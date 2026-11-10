# Security

Do not post access tokens, credentials, tenant identifiers, mailbox exports or calendar diagnostic data in public issues. If you publish this repository, enable GitHub private vulnerability reporting before accepting security reports.

The application uses delegated user authentication through Microsoft's ExchangeOnlineManagement module. It does not store authentication tokens itself; Microsoft's module can cache tokens. Protect the operating-system account that runs the app and keep PowerShell and the module updated.

Optional availability-service credentials are held in process memory while submitting the selected operation. The app does not promise forensic memory erasure. Calendar/organization changes use the signed-in user's permissions and can affect users and attendees; review targets and use previews with test resources before production use.
