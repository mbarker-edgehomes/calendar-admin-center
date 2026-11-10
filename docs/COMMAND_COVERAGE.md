# Command coverage

Reference retrieved: 2026-11-10. Exchange Online only.

Every cmdlet in the linked reference whose entry mentions calendar, free/busy, scheduling or booking is classified. Companion operations for folder permissions, sharing policies, organization relationships, availability, regional settings, resources and places are included. General mailbox and organization commands provide calendar-related settings alongside their other settings.

The reference includes on-premises and retired operations; those are explicitly excluded below. Calendar workflows use live `Get-Command` metadata for the authenticated user. Set-Mailbox and Set-OrganizationConfig are scoped to calendar-related settings; their unrelated features are outside this app. The offline catalog filters documented on-premises-only parameters; execution requires a live form.

## Exchange Online forms

| Workflow | Cmdlet | Reference parameters |
|---|---|---:|
| Sharing & availability | [Add-AvailabilityAddressSpace](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/add-availabilityaddressspace?view=exchange-ps) | 8 |
| Folder permissions | [Add-MailboxFolderPermission](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/add-mailboxfolderpermission?view=exchange-ps) | 7 |
| Sharing & availability | [Get-AvailabilityAddressSpace](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/get-availabilityaddressspace?view=exchange-ps) | 1 |
| Sharing & availability | [Get-AvailabilityConfig](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/get-availabilityconfig?view=exchange-ps) | 1 |
| Diagnostics & events | [Get-CalendarDiagnosticObjects](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/get-calendardiagnosticobjects?view=exchange-ps) | 22 |
| Booking & resources | [Get-CalendarProcessing](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/get-calendarprocessing?view=exchange-ps) | 2 |
| Directory & folders | [Get-EXOMailbox](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/get-exomailbox?view=exchange-ps) | 20 |
| Folder permissions | [Get-EXOMailboxFolderPermission](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/get-exomailboxfolderpermission?view=exchange-ps) | 6 |
| Directory & folders | [Get-EXOMailboxFolderStatistics](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/get-exomailboxfolderstatistics?view=exchange-ps) | 9 |
| Calendar appearance | [Get-EventsFromEmailConfiguration](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/get-eventsfromemailconfiguration?view=exchange-ps) | 1 |
| Directory & folders | [Get-Mailbox](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/get-mailbox?view=exchange-ps) | 21 |
| Calendar appearance | [Get-MailboxCalendarConfiguration](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/get-mailboxcalendarconfiguration?view=exchange-ps) | 2 |
| Calendar publishing | [Get-MailboxCalendarFolder](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/get-mailboxcalendarfolder?view=exchange-ps) | 2 |
| Directory & folders | [Get-MailboxFolder](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/get-mailboxfolder?view=exchange-ps) | 5 |
| Folder permissions | [Get-MailboxFolderPermission](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/get-mailboxfolderpermission?view=exchange-ps) | 6 |
| Directory & folders | [Get-MailboxFolderStatistics](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/get-mailboxfolderstatistics?view=exchange-ps) | 12 |
| Calendar appearance | [Get-MailboxRegionalConfiguration](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/get-mailboxregionalconfiguration?view=exchange-ps) | 5 |
| Sharing & availability | [Get-OrganizationConfig](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/get-organizationconfig?view=exchange-ps) | 0 |
| Sharing & availability | [Get-OrganizationRelationship](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/get-organizationrelationship?view=exchange-ps) | 1 |
| Booking & resources | [Get-Place](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/get-place?view=exchange-ps) | 5 |
| Booking & resources | [Get-ResourceConfig](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/get-resourceconfig?view=exchange-ps) | 0 |
| Sharing & availability | [Get-SharingPolicy](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/get-sharingpolicy?view=exchange-ps) | 1 |
| Diagnostics & events | [Invoke-CalendarEventMigration](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/invoke-calendareventmigration?view=exchange-ps) | 9 |
| Sharing & availability | [New-AvailabilityConfig](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/new-availabilityconfig?view=exchange-ps) | 4 |
| Directory & folders | [New-MailboxFolder](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/new-mailboxfolder?view=exchange-ps) | 5 |
| Sharing & availability | [New-OrganizationRelationship](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/new-organizationrelationship?view=exchange-ps) | 23 |
| Sharing & availability | [New-SharingPolicy](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/new-sharingpolicy?view=exchange-ps) | 6 |
| Sharing & availability | [Remove-AvailabilityAddressSpace](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/remove-availabilityaddressspace?view=exchange-ps) | 3 |
| Sharing & availability | [Remove-AvailabilityConfig](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/remove-availabilityconfig?view=exchange-ps) | 3 |
| Diagnostics & events | [Remove-CalendarEvents](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/remove-calendarevents?view=exchange-ps) | 8 |
| Folder permissions | [Remove-MailboxFolderPermission](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/remove-mailboxfolderpermission?view=exchange-ps) | 7 |
| Sharing & availability | [Remove-OrganizationRelationship](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/remove-organizationrelationship?view=exchange-ps) | 3 |
| Sharing & availability | [Remove-SharingPolicy](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/remove-sharingpolicy?view=exchange-ps) | 3 |
| Sharing & availability | [Set-AvailabilityConfig](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/set-availabilityconfig?view=exchange-ps) | 4 |
| Booking & resources | [Set-CalendarProcessing](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/set-calendarprocessing?view=exchange-ps) | 42 |
| Calendar appearance | [Set-EventsFromEmailConfiguration](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/set-eventsfromemailconfiguration?view=exchange-ps) | 11 |
| Directory & folders | [Set-Mailbox](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/set-mailbox?view=exchange-ps) | 132 |
| Calendar appearance | [Set-MailboxCalendarConfiguration](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/set-mailboxcalendarconfiguration?view=exchange-ps) | 50 |
| Calendar publishing | [Set-MailboxCalendarFolder](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/set-mailboxcalendarfolder?view=exchange-ps) | 12 |
| Folder permissions | [Set-MailboxFolderPermission](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/set-mailboxfolderpermission?view=exchange-ps) | 7 |
| Calendar appearance | [Set-MailboxRegionalConfiguration](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/set-mailboxregionalconfiguration?view=exchange-ps) | 11 |
| Sharing & availability | [Set-OrganizationConfig](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/set-organizationconfig?view=exchange-ps) | 143 |
| Sharing & availability | [Set-OrganizationRelationship](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/set-organizationrelationship?view=exchange-ps) | 25 |
| Booking & resources | [Set-Place](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/set-place?view=exchange-ps) | 22 |
| Booking & resources | [Set-ResourceConfig](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/set-resourceconfig?view=exchange-ps) | 3 |
| Sharing & availability | [Set-SharingPolicy](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/set-sharingpolicy?view=exchange-ps) | 7 |
| Sharing & availability | [Test-OrganizationRelationship](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/test-organizationrelationship?view=exchange-ps) | 4 |

## Exclusions

| Cmdlet | Reason |
|---|---|
| [Get-CalendarDiagnosticAnalysis](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/get-calendardiagnosticanalysis?view=exchange-ps) | on-premises |
| [Get-CalendarDiagnosticLog](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/get-calendardiagnosticlog?view=exchange-ps) | on-premises |
| [Get-CalendarNotification](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/get-calendarnotification?view=exchange-ps) | discontinued |
| [Set-CalendarNotification](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/set-calendarnotification?view=exchange-ps) | discontinued |
| [Test-CalendarConnectivity](https://learn.microsoft.com/en-us/powershell/module/exchangepowershell/test-calendarconnectivity?view=exchange-ps) | on-premises |

## Verification boundaries

The source inventory test checks every matching reference entry against the catalog, and a workflow test checks that every active operation has an application route. Parameter controls, request construction, validation, broker protocol and a native macOS build can be verified without a tenant. Live tenant permissions, GDAP grants, Conditional Access and the effect of each administrative mutation require tenant acceptance testing. The repository does not claim those have been exercised merely because a form exists.

Mailbox folders and diagnostic records are inspectable. Exchange Online PowerShell does not provide arbitrary calendar event CRUD or a complete event agenda; those would require a separate API and authentication design.
