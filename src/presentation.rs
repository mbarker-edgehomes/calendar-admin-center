use chrono::{NaiveTime, Timelike};
use serde_json::Value;

pub const WEEK_DAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

fn scalar(value: &Value) -> &Value {
    if let Value::Object(object) = value {
        for key in ["Value", "value"] {
            if let Some(inner) = object.get(key) {
                if !inner.is_object() {
                    return inner;
                }
            }
        }
    }
    value
}

/// Exchange can return TimeSpan as a string or a deserialized .NET object.
/// Interpret its value; never render its implementation properties.
pub fn duration_seconds(value: &Value) -> Option<f64> {
    let value = scalar(value);
    if let Some(object) = value.as_object() {
        if let Some(seconds) = object.get("TotalSeconds").and_then(Value::as_f64) {
            return Some(seconds);
        }
        if let Some(ticks) = object.get("Ticks").and_then(Value::as_f64) {
            return Some(ticks / 10_000_000.0);
        }
        if object.contains_key("Hours") && object.contains_key("Minutes") {
            return Some(
                object.get("Days").and_then(Value::as_f64).unwrap_or(0.0) * 86400.0
                    + object.get("Hours")?.as_f64()? * 3600.0
                    + object.get("Minutes")?.as_f64()? * 60.0
                    + object.get("Seconds").and_then(Value::as_f64).unwrap_or(0.0),
            );
        }
    }
    if let Some(text) = value.as_str() {
        for format in ["%H:%M:%S%.f", "%H:%M", "%I:%M %p", "%I:%M:%S %p"] {
            if let Ok(time) = NaiveTime::parse_from_str(text.trim(), format) {
                return Some(time.num_seconds_from_midnight() as f64);
            }
        }
        // Invariant TimeSpan strings can include a day component, e.g. 1.02:30:00.
        if let Some((days, clock)) = text.split_once('.') {
            if let (Ok(days), Some(clock)) = (
                days.parse::<i64>(),
                duration_seconds(&Value::String(clock.into())),
            ) {
                return Some(days as f64 * 86400.0 + clock);
            }
        }
    }
    None
}

pub fn clock(value: &Value) -> Option<NaiveTime> {
    let seconds = duration_seconds(value)?;
    if !seconds.is_finite() || !(0.0..86400.0).contains(&seconds) {
        return None;
    }
    NaiveTime::from_num_seconds_from_midnight_opt(seconds as u32, 0)
}

pub fn is_clock_field(name: &str) -> bool {
    matches!(name, "WorkingHoursStartTime" | "WorkingHoursEndTime")
}

pub fn working_days(value: &Value) -> Option<Vec<&'static str>> {
    let value = scalar(value);
    if value.is_null() {
        return None;
    }
    let mut selected = [false; 7];
    let parts: Vec<Value> = if let Some(array) = value.as_array() {
        array.clone()
    } else if let Some(text) = value.as_str() {
        text.split(',')
            .map(|s| Value::String(s.trim().into()))
            .collect()
    } else {
        vec![value.clone()]
    };
    for part in parts {
        if let Some(bits) = part.as_u64() {
            if bits > 127 {
                return None;
            }
            for (i, day) in WEEK_DAYS.iter().enumerate() {
                let bit = if *day == "Sunday" { 1 } else { 1 << (i + 1) };
                selected[i] |= bits & bit != 0;
            }
            continue;
        }
        let text = part.as_str()?.to_ascii_lowercase();
        match text.as_str() {
            "weekdays" => selected[..5].fill(true),
            "alldays" | "all" | "everyday" => selected.fill(true),
            "weekenddays" | "weekends" | "weekend" => selected[5..].fill(true),
            "none" | "" => {}
            _ => {
                let i = WEEK_DAYS
                    .iter()
                    .position(|day| day.eq_ignore_ascii_case(&text))?;
                selected[i] = true;
            }
        }
    }
    Some(
        WEEK_DAYS
            .into_iter()
            .enumerate()
            .filter_map(|(i, day)| selected[i].then_some(day))
            .collect(),
    )
}

/// Canonical editable data is deliberately separate from its readable label.
pub fn edit_value(name: &str, value: &Value) -> Option<String> {
    let value = scalar(value);
    if value.is_null() {
        return None;
    }
    if is_clock_field(name) {
        return clock(value).map(|t| t.format("%H:%M:%S").to_string());
    }
    if name == "WorkDays" {
        return working_days(value).map(|days| {
            if days.is_empty() {
                "None".into()
            } else {
                days.join(", ")
            }
        });
    }
    if name == "DefaultReminderTime"
        || name == "DefaultMeetingDuration"
        || name.ends_with("InMinutes")
    {
        if let Some(seconds) = duration_seconds(value) {
            return Some((seconds / 60.0).round().to_string());
        }
    }
    if name.ends_with("InDays") {
        if let Some(seconds) = duration_seconds(value) {
            return Some((seconds / 86400.0).round().to_string());
        }
    }
    if name == "Domains" {
        if let Some(domain) = value.get("Domain").or_else(|| value.get("DomainName")) {
            let domain = edit_value("Domain", domain)?;
            let action = edit_value("Action", value.get("Actions")?)?.replace('\n', ",");
            return Some(format!("{domain}:{action}"));
        }
    }
    if name.contains("TimeZone") {
        if let Some(id) = value
            .get("Id")
            .or_else(|| value.get("IdString"))
            .and_then(Value::as_str)
        {
            return Some(id.into());
        }
    }
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Number(n) => Some(n.to_string()),
        Value::Array(a) => a
            .iter()
            .map(|v| edit_value(name, v))
            .collect::<Option<Vec<_>>>()
            .map(|a| a.join("\n")),
        Value::Object(o) => {
            for key in [
                "PrimarySmtpAddress",
                "Identity",
                "Id",
                "Name",
                "DisplayName",
            ] {
                if let Some(value) = o.get(key).filter(|v| v.is_string() || v.is_number()) {
                    return edit_value(name, value);
                }
            }
            duration_seconds(value).map(|s| {
                let whole = s.abs().round() as u64;
                format!(
                    "{}{}{:02}:{:02}:{:02}",
                    if s < 0.0 { "-" } else { "" },
                    if whole >= 86400 {
                        format!("{}.", whole / 86400)
                    } else {
                        String::new()
                    },
                    (whole / 3600) % 24,
                    (whole / 60) % 60,
                    whole % 60
                )
            })
        }
        Value::Null => None,
    }
}

pub fn field_value(name: &str, value: &Value) -> String {
    if is_clock_field(name) {
        return clock(value)
            .map(|time| time.format("%-I:%M %p").to_string())
            .unwrap_or_else(|| "Not set".into());
    }
    if name == "WorkDays" {
        return working_days(value)
            .map(|days| {
                if days == WEEK_DAYS[..5] {
                    "Monday–Friday".into()
                } else if days == WEEK_DAYS {
                    "Every day".into()
                } else if days == WEEK_DAYS[5..] {
                    "Saturday–Sunday".into()
                } else if days.is_empty() {
                    "No working days".into()
                } else {
                    days.join(", ")
                }
            })
            .unwrap_or_else(|| "Not set".into());
    }
    if name == "DefaultReminderTime" {
        if let Some(minutes) = edit_value(name, value) {
            return format!("{minutes} minutes");
        }
    }
    if name.contains("TimeZone") {
        if let Some(id) = edit_value(name, value) {
            return match id.as_str() {
                "Pacific Standard Time" => "Pacific Time (US & Canada)".into(),
                "Mountain Standard Time" => "Mountain Time (US & Canada)".into(),
                "Central Standard Time" => "Central Time (US & Canada)".into(),
                "Eastern Standard Time" => "Eastern Time (US & Canada)".into(),
                "UTC" => "Coordinated Universal Time".into(),
                _ => id,
            };
        }
    }
    if name == "Language" {
        if let Some(display) = value.get("DisplayName").and_then(Value::as_str) {
            return display.into();
        }
    }
    if let Some(text) = scalar(value).as_str() {
        option(text)
    } else {
        self::value(value)
    }
}

pub fn label(name: &str) -> String {
    match name {
        "Identity" => "Name",
        "User" => "Person or group",
        "AccessRights" => "Access level",
        "SharingPermissionFlags" => "Delegate access",
        "SendNotificationToUser" => "Send an invitation",
        "AutomateProcessing" => "Booking approval",
        "ResourceDelegates" => "Booking approvers",
        "AllBookInPolicy" => "Allow everyone to book automatically",
        "AllRequestInPolicy" => "Let everyone request approval",
        "AllRequestOutOfPolicy" => "Let everyone request exceptions",
        "BookInPolicy" => "People who can book automatically",
        "RequestInPolicy" => "People who need approval",
        "RequestOutOfPolicy" => "People who can request exceptions",
        "AllowConflicts" => "Allow overlapping bookings",
        "AllowRecurringMeetings" => "Allow recurring meetings",
        "BookingWindowInDays" => "Days people can book ahead",
        "MaximumDurationInMinutes" => "Longest booking (minutes)",
        "MinimumDurationInMinutes" => "Shortest booking (minutes)",
        "ScheduleOnlyDuringWorkHours" => "Only accept bookings during working hours",
        "EnforceSchedulingHorizon" => "Decline bookings outside the booking window",
        "EnforceCapacity" => "Decline bookings over capacity",
        "ConflictPercentageAllowed" => "Allowed conflict percentage",
        "MaximumConflictInstances" => "Allowed conflicting occurrences",
        "ForwardRequestsToDelegates" => "Send booking requests to approvers",
        "ProcessExternalMeetingMessages" => "Accept requests from outside the organization",
        "DeleteSubject" => "Hide the meeting title",
        "AddOrganizerToSubject" => "Show the organizer in the title",
        "DeleteComments" => "Remove the meeting description",
        "DeleteAttachments" => "Remove meeting attachments",
        "DeleteNonCalendarItems" => "Remove messages that aren't meeting requests",
        "RemovePrivateProperty" => "Remove the private flag",
        "OrganizerInfo" => "Show organizer information",
        "AddAdditionalResponse" => "Include a custom booking reply",
        "AdditionalResponse" => "Booking reply message",
        "RemoveCanceledMeetings" => "Remove canceled meetings",
        "RemoveOldMeetingMessages" => "Remove old meeting messages",
        "RemoveForwardedMeetingNotifications" => "Remove forwarded meeting notices",
        "EnableAutoRelease" => "Release unattended bookings",
        "PostReservationMaxClaimTimeInMinutes" => "Check-in grace period (minutes)",
        "EnableResponseDetails" => "Include booking details in replies",
        "TentativePendingApproval" => "Mark requests tentative while waiting for approval",
        "AddNewRequestsTentatively" => "Add new requests as tentative",
        "BookingType" => "Who can reserve this resource",
        "WorkingHoursStartTime" => "Workday starts",
        "WorkingHoursEndTime" => "Workday ends",
        "WorkingHoursTimeZone" => "Working-hours time zone",
        "WorkDays" => "Working days",
        "WeekStartDay" => "First day of the week",
        "TimeZone" => "Time zone",
        "Language" => "Language",
        "DateFormat" => "Date format",
        "TimeFormat" => "Time format",
        "LocalizeDefaultFolderName" => "Translate default folder names",
        "DefaultReminderTime" => "Default reminder (minutes)",
        "RemindersEnabled" => "Show reminders",
        "ReminderSoundEnabled" => "Play a reminder sound",
        "DefaultMeetingDuration" => "Default meeting length (minutes)",
        "DefaultOnlineMeetingProvider" => "Default online meeting service",
        "OnlineMeetingsByDefaultEnabled" => "Make new meetings online by default",
        "ShortenEventScopeDefault" => "Shorten new meetings",
        "DefaultMinutesToReduceShortEventsBy" => "Short-meeting reduction (minutes)",
        "DefaultMinutesToReduceLongEventsBy" => "Long-meeting reduction (minutes)",
        "ShowWeekNumbers" => "Show week numbers",
        "ShowWorkWeek" => "Show the working week",
        "ShowWeather" => "Show weather",
        "WeatherUnit" => "Temperature unit",
        "PublishEnabled" => "Publish this calendar",
        "DetailLevel" => "Published event details",
        "PublishDateRangeFrom" => "Include past events",
        "PublishDateRangeTo" => "Include future events",
        "SearchableUrlEnabled" => "Allow search engines to find the calendar",
        "ResetUrl" => "Create new sharing links",
        "PublishedCalendarUrl" => "Calendar web link",
        "PublishedICalUrl" => "Calendar subscription link",
        "SharingPolicy" => "External sharing policy",
        "GrantSendOnBehalfTo" => "People who can send on behalf of this mailbox",
        "CalendarRepairDisabled" => "Turn off automatic calendar repair",
        "CalendarVersionStoreDisabled" => "Turn off calendar change history",
        "EventsFromEmailEnabled" => "Add events from email",
        "CreateEventsFromEmailAsPrivate" => "Keep events from email private",
        "FlightReservationProcessingLevel" => "Flights",
        "HotelReservationProcessingLevel" => "Hotels",
        "RentalCarReservationProcessingLevel" => "Car rentals",
        "PackageDeliveryProcessingLevel" => "Package deliveries",
        "DiningReservationProcessingLevel" => "Dining reservations",
        "EventReservationProcessingLevel" => "Event reservations",
        "InvoiceProcessingLevel" => "Invoices",
        "ServiceReservationProcessingLevel" => "Service reservations",
        "Subject" => "Meeting title",
        "StartDate" => "From",
        "EndDate" => "To",
        "ExactMatch" => "Match the title exactly",
        "MeetingId" => "Meeting reference",
        "EntryId" | "EwsId" | "ODataId" => "Event reference",
        "QueryStartDate" => "Starting date",
        "QueryWindowInDays" => "Number of days to include",
        "CancelOrganizedMeetings" => "Cancel meetings organized by this person",
        "FromWindowsTimeZoneId" => "Current time zone",
        "ToWindowsTimeZoneId" => "New time zone",
        "MaxResults" | "ResultSize" => "Maximum results",
        "Name" => "Name",
        "Parent" => "Parent folder",
        "Domains" => "Allowed external domains",
        "Default" => "Use as the default policy",
        "Enabled" => "Enabled",
        "DomainNames" => "Organization domains",
        "FreeBusyAccessEnabled" => "Share availability",
        "FreeBusyAccessLevel" => "Availability details",
        "FreeBusyAccessScope" => "People whose availability is shared",
        "TargetApplicationUri" => "Partner application address",
        "TargetAutodiscoverEpr" => "Partner discovery address",
        "TargetSharingEpr" => "Partner sharing address",
        "ForestName" => "Partner domain",
        "AccessMethod" => "Connection type",
        "Capacity" => "Seats",
        "Building" => "Building",
        "Floor" => "Floor",
        "FloorLabel" => "Floor name",
        "Label" => "Room label",
        "AudioDeviceName" => "Audio equipment",
        "VideoDeviceName" => "Video equipment",
        "DisplayDeviceName" => "Display equipment",
        "IsWheelChairAccessible" => "Wheelchair accessible",
        "Tags" => "Features",
        "ResourcePropertySchema" => "Room and equipment attributes",
        "GetChildren" => "Include child folders",
        "Recurse" => "Include all subfolders",
        "Force" => "Allow the operation when additional checks would block it",
        _ => return sentence_case(&crate::catalog::humanize(name)),
    }
    .into()
}

pub fn sentence_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
        None => String::new(),
    }
}

pub fn option(value: &str) -> String {
    match value {
        "AvailabilityOnly" => "Can see when I'm busy",
        "LimitedDetails" => "Can see titles and locations",
        "Reviewer" => "Can view all details",
        "Editor" => "Can edit events",
        "PublishingEditor" => "Can edit events and create subfolders",
        "Owner" => "Full control",
        "Contributor" => "Can add events",
        "Author" => "Can edit their own events",
        "NonEditingAuthor" => "Can add and read their own events",
        "PublishingAuthor" => "Can manage their events and subfolders",
        "None" => "No access",
        "Default" => "Everyone in your organization",
        "Anonymous" => "People outside your organization",
        "AutoAccept" => "Accept or decline automatically",
        "AutoUpdate" => "Add requests and let someone respond",
        "AutoDecline" => "Decline automatically",
        "true" => "On",
        "false" => "Off",
        "Disabled" => "Off",
        "Enabled" => "On",
        "AddToCalendarAndCard" => "Show in email and add to calendar",
        "EmailOnly" => "Show in email only",
        "CardOnly" => "Show in email only",
        "FullDetails" => "All event details",
        "AvailabilityOnlyFreeBusy" => "Availability only",
        "LimitedDetailsFreeBusy" => "Titles and locations",
        _ => return crate::catalog::humanize(value),
    }
    .into()
}

pub fn value(value: &Value) -> String {
    match value {
        Value::Null => "Not set".into(),
        Value::Bool(v) => if *v { "On" } else { "Off" }.into(),
        Value::String(v) => v.clone(),
        Value::Array(a) => a.iter().map(self::value).collect::<Vec<_>>().join(", "),
        Value::Object(o) => {
            if let Some(seconds) = duration_seconds(value) {
                let minutes = (seconds / 60.0).round() as i64;
                return if minutes % 60 == 0 {
                    format!("{} hours", minutes / 60)
                } else {
                    format!("{minutes} minutes")
                };
            }
            for key in [
                "DisplayName",
                "Name",
                "Identity",
                "PrimarySmtpAddress",
                "Value",
                "value",
            ] {
                if let Some(v) = o.get(key) {
                    return self::value(v);
                }
            }
            "Not available".into()
        }
        _ => value.to_string(),
    }
}

pub fn description(name: &str) -> &'static str {
    match name {
        "AccessRights" => "Choose what this person can see or change.",
        "BookingWindowInDays" => "How far in advance someone can reserve this resource.",
        "ResourceDelegates" => "These people can approve or decline booking requests.",
        "DeleteSubject" => "Only availability will be visible unless another rule adds a title.",
        "WorkingHoursTimeZone" | "TimeZone" => "Select the time zone used for this calendar.",
        "SearchableUrlEnabled" => "Published links may be indexed by search engines.",
        "ResetUrl" => "Existing published links will stop working.",
        "CancelOrganizedMeetings" => "Attendees will receive cancellation notices.",
        "FromWindowsTimeZoneId" => "The time zone currently assigned to the affected events.",
        "ToWindowsTimeZoneId" => {
            "Events keep their local time in this time zone. Attendees may receive updates."
        }
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn exchange_settings() -> Value {
        serde_json::from_str(include_str!("../data/demo-calendar.json")).unwrap()
    }
    #[test]
    fn real_powershell_time_spans_are_readable_and_editable() {
        let data = exchange_settings();
        assert_eq!(
            field_value("WorkingHoursStartTime", &data["WorkingHoursStartTime"]),
            "8:00 AM"
        );
        assert_eq!(
            field_value("WorkingHoursEndTime", &data["WorkingHoursEndTime"]),
            "5:00 PM"
        );
        assert_eq!(
            edit_value("WorkingHoursStartTime", &data["WorkingHoursStartTime"]).as_deref(),
            Some("08:00:00")
        );
        assert_eq!(
            edit_value("WorkingHoursEndTime", &data["WorkingHoursEndTime"]).as_deref(),
            Some("17:00:00")
        );
        assert_eq!(
            edit_value("DefaultReminderTime", &data["DefaultReminderTime"]).as_deref(),
            Some("15")
        );
        assert_eq!(field_value("WorkDays", &data["WorkDays"]), "Monday–Friday");
        assert_eq!(
            field_value("Language", &data["Language"]),
            "English (United States)"
        );
    }
    #[test]
    fn clock_strings_wrappers_and_boundary_times_are_supported() {
        for value in [
            json!("08:00:00"),
            json!({"Hours":8,"Minutes":0,"Seconds":0,"Days":0}),
            json!({"Value":"08:00:00"}),
            json!({"Ticks":288000000000_i64}),
        ] {
            assert_eq!(field_value("WorkingHoursStartTime", &value), "8:00 AM");
        }
        assert_eq!(
            field_value("WorkingHoursEndTime", &json!("17:45:00")),
            "5:45 PM"
        );
        assert_eq!(
            field_value("WorkingHoursStartTime", &json!("00:00:00")),
            "12:00 AM"
        );
        assert_eq!(
            field_value("WorkingHoursStartTime", &json!("12:00:00")),
            "12:00 PM"
        );
        assert!(clock(&json!({"TotalSeconds":-3600})).is_none());
        assert!(clock(&json!({"TotalSeconds":86400})).is_none());
        assert!(clock(&Value::Null).is_none());
        assert_eq!(
            edit_value("DefaultReminderTime", &json!("00:15:00")).as_deref(),
            Some("15")
        );
    }
    #[test]
    fn grouped_working_days_round_trip_to_individual_controls() {
        for value in [
            json!("Weekdays"),
            json!(62),
            json!({"Value":"Weekdays"}),
            json!(["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"]),
        ] {
            assert_eq!(
                edit_value("WorkDays", &value).as_deref(),
                Some("Monday, Tuesday, Wednesday, Thursday, Friday")
            );
        }
        assert_eq!(field_value("WorkDays", &json!("AllDays")), "Every day");
        assert_eq!(
            field_value("WorkDays", &json!("WeekendDays")),
            "Saturday–Sunday"
        );
        assert_eq!(
            edit_value("WorkDays", &json!("None")).as_deref(),
            Some("None")
        );
    }
    #[test]
    fn arbitrary_objects_are_never_dumped_into_the_interface() {
        let object =
            json!({"InternalType":"example","Nanoseconds":123,"ImplementationDetail":{"X":1}});
        assert_eq!(value(&object), "Not available");
        assert!(edit_value("UnsupportedSetting", &object).is_none());
        let data = exchange_settings();
        let rendered = value(&data["WorkingHoursStartTime"]);
        assert!(!rendered.contains("Ticks"));
        assert!(!rendered.contains("Nanoseconds"));
    }
}
