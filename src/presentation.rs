use serde_json::Value;

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
            o.iter()
                .map(|(k, v)| format!("{}: {}", label(k), self::value(v)))
                .collect::<Vec<_>>()
                .join("; ")
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
