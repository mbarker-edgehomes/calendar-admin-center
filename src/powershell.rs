// Fixed broker program. User values travel only as JSON on stdin and are splatted as data.
pub const SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$PSStyle.OutputRendering = 'PlainText'
[Console]::InputEncoding = [Text.UTF8Encoding]::new($false)
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
$allowed = @(__ALLOWLIST__)
$connected = $false
function Reply($id, $ok, $data, $message) {
    $packet = @{ id=$id; ok=$ok; data=$data; message=$message }
    [Console]::Out.WriteLine('@@CAC@@' + (ConvertTo-Json -InputObject $packet -Depth 30 -Compress -EnumsAsStrings -WarningAction SilentlyContinue))
    [Console]::Out.Flush()
}
function Schema($name) {
    if ($name -notin $allowed) { throw 'Command is outside the calendar administration catalog.' }
    $cmd = Get-Command -Name $name -ErrorAction Stop
    $common = @('Verbose','Debug','ErrorAction','WarningAction','InformationAction','ProgressAction','ErrorVariable','WarningVariable','InformationVariable','OutVariable','OutBuffer','PipelineVariable')
    $items = @($cmd.Parameters.Values | Where-Object { $_.Name -notin $common } | ForEach-Object {
        $p=$_; $vals=@()
        if ($p.ParameterType.IsEnum) { $vals = @([Enum]::GetNames($p.ParameterType)) }
        foreach ($attr in $p.Attributes) {
            if ($attr -is [Management.Automation.ValidateSetAttribute]) { $vals = @($attr.ValidValues) }
        }
        $sets=@($p.ParameterSets.Keys); $mandatory=@($p.ParameterSets.Keys | Where-Object { $p.ParameterSets[$_].IsMandatory })
        @{ name=$p.Name; type=$p.ParameterType.FullName; required=($mandatory.Count -gt 0); sets=$sets; mandatory_sets=$mandatory; values=$vals }
    })
    @{ parameters=$items; default_set=$cmd.DefaultParameterSet }
}
try {
    $version = $PSVersionTable.PSVersion.ToString()
    $module = Get-Module -ListAvailable ExchangeOnlineManagement | Sort-Object Version -Descending | Select-Object -First 1
    Reply 0 $true @{ powershell=$version; module=$(if($module){$module.Version.ToString()}else{''}) } ''
} catch { Reply 0 $false $null $_.Exception.Message }
while ($null -ne ($line = [Console]::In.ReadLine())) {
    $request=$null
    try {
        $request = ConvertFrom-Json -InputObject $line -AsHashtable
        $id=$request.id; $action=$request.action; $p=$request.payload
        switch ($action) {
            'zones' {
                $zones=@([TimeZoneInfo]::GetSystemTimeZones() | ForEach-Object {
                    $zone=$_; $zoneId=$zone.Id
                    if ($IsWindows -or [TimeZoneInfo]::TryConvertIanaIdToWindowsId($zone.Id,[ref]$zoneId)) {
                        @{ id=$zoneId; name=$zone.DisplayName }
                    }
                } | Sort-Object -Property @{e={$_.id}} -Unique)
                Reply $id $true $zones ''
            }
            'connect' {
                if ($connected) { Disconnect-ExchangeOnline -Confirm:$false -ErrorAction Stop; $connected=$false }
                Import-Module ExchangeOnlineManagement -MinimumVersion 3.7.0 -ErrorAction Stop
                $connection = @{ ShowBanner=$false; ErrorAction='Stop' }
                if ($p.upn) { $connection.UserPrincipalName=[string]$p.upn }
                if ($p.organization) { $connection.DelegatedOrganization=[string]$p.organization }
                if ($p.device) { $connection.Device=$true }
                if ($IsWindows -and $p.disable_wam) { $connection.DisableWAM=$true }
                Connect-ExchangeOnline @connection
                $connected=$true
                $info=@(Get-ConnectionInformation | Select-Object UserPrincipalName,TenantID,Organization,DelegatedOrganization,State)
                $available=@($allowed | Where-Object { Get-Command -Name $_ -ErrorAction SilentlyContinue })
                Reply $id $true @{ connections=$info; commands=$available } ''
            }
            'disconnect' {
                if ($connected) { Disconnect-ExchangeOnline -Confirm:$false -ErrorAction Stop }
                $connected=$false
                Reply $id $true $null ''
            }
            'directory' {
                if (!$connected) { throw 'Sign in to Exchange Online first.' }
                $rows=@(Get-EXOMailbox -ResultSize Unlimited -Properties DisplayName,PrimarySmtpAddress,RecipientTypeDetails,ExternalDirectoryObjectId |
                    Select-Object @{n='DisplayName';e={[string]$_.DisplayName}},@{n='PrimarySmtpAddress';e={[string]$_.PrimarySmtpAddress}},@{n='RecipientTypeDetails';e={[string]$_.RecipientTypeDetails}},@{n='ExternalDirectoryObjectId';e={[string]$_.ExternalDirectoryObjectId}})
                Reply $id $true $rows ''
            }
            'calendars' {
                if (!$connected) { throw 'Sign in to Exchange Online first.' }
                $rows=@(Get-EXOMailboxFolderStatistics -Identity ([string]$p.mailbox) -FolderScope Calendar |
                    Select-Object @{n='Name';e={[string]$_.Name}},@{n='FolderPath';e={[string]$_.FolderPath}},@{n='FolderType';e={[string]$_.FolderType}},ItemsInFolder,@{n='FolderSize';e={[string]$_.FolderSize}})
                Reply $id $true $rows ''
            }
            'recipients' {
                if (!$connected) { throw 'Sign in to Exchange Online first.' }
                $rows=@(Get-EXORecipient -ResultSize Unlimited -Properties DisplayName,PrimarySmtpAddress,RecipientTypeDetails,ExternalDirectoryObjectId |
                    Select-Object @{n='DisplayName';e={[string]$_.DisplayName}},@{n='PrimarySmtpAddress';e={[string]$_.PrimarySmtpAddress}},@{n='RecipientTypeDetails';e={[string]$_.RecipientTypeDetails}},@{n='ExternalDirectoryObjectId';e={[string]$_.ExternalDirectoryObjectId}})
                Reply $id $true $rows ''
            }
            'schema' {
                if (!$connected) { throw 'Sign in to Exchange Online first.' }
                Reply $id $true (Schema ([string]$p.command)) ''
            }
            'invoke' {
                if (!$connected) { throw 'Sign in to Exchange Online first.' }
                $name=[string]$p.command
                if ($name -notin $allowed) { throw 'Command is outside the calendar administration catalog.' }
                $cmd=Get-Command -Name $name -ErrorAction Stop
                $arguments=@{}
                foreach ($key in $p.arguments.Keys) {
                    if (!$cmd.Parameters.ContainsKey($key)) { throw "Parameter $key is unavailable in the current Exchange session." }
                    if ($key -in @('Confirm','ErrorAction','WarningAction','InformationAction','OutVariable','ErrorVariable')) { throw 'Reserved broker parameter.' }
                    $value=$p.arguments[$key]; $type=$cmd.Parameters[$key].ParameterType
                    if ($null -eq $value) { $arguments[$key]=$null }
                    elseif ($type -eq [Management.Automation.PSCredential]) {
                        $arguments[$key]=[Management.Automation.PSCredential]::new([string]$value.username,(ConvertTo-SecureString ([string]$value.password) -AsPlainText -Force))
                    } elseif ($type -eq [Security.SecureString]) {
                        $arguments[$key]=ConvertTo-SecureString ([string]$value) -AsPlainText -Force
                    } elseif ($type -eq [Management.Automation.SwitchParameter] -or $type -eq [bool]) {
                        if ($value -isnot [bool]) { throw "Parameter $key needs a Boolean value." }
                        $arguments[$key]=[bool]$value
                    } elseif ($type -eq [datetime] -or $type.FullName -like '*ExDateTime*') {
                        $arguments[$key]=[datetime]::Parse([string]$value,[Globalization.CultureInfo]::InvariantCulture,[Globalization.DateTimeStyles]::RoundtripKind)
                    } else { $arguments[$key]=$value }
                }
                if ($cmd.Parameters.ContainsKey('Confirm')) { $arguments.Confirm=$false }
                $arguments.ErrorAction='Stop'
                $rows=@(& $name @arguments)
                if ($name -in @('Get-MailboxFolderPermission','Get-EXOMailboxFolderPermission')) {
                    $rows=@($rows | Select-Object @{n='User';e={[string]$_.User}},@{n='AccessRights';e={@($_.AccessRights | ForEach-Object { [string]$_ })}},@{n='SharingPermissionFlags';e={[string]$_.SharingPermissionFlags}})
                }
                Reply $id $true $rows ''
                $arguments.Clear(); $p.arguments.Clear()
            }
            default { throw 'Unknown broker operation.' }
        }
    } catch {
        $message=$_.Exception.Message
        if ($request -and $request.payload.arguments) {
            foreach($v in $request.payload.arguments.Values) {
                if ($v -is [System.Collections.IDictionary] -and $v.password) { $message=$message.Replace([string]$v.password,'[redacted]') }
            }
        }
        Reply $(if($request){$request.id}else{-1}) $false $null $message
    }
}
if ($connected) { Disconnect-ExchangeOnline -Confirm:$false -ErrorAction SilentlyContinue }
"#;
