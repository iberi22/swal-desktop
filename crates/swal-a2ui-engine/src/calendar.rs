//! Calendar and Agenda Engine for SWAL A2UI
//!
//! Provides date calculations, calendar grid state, and agenda event management.

use serde::{Deserialize, Serialize};

/// Represents an interactive monthly calendar grid component.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CalendarGrid {
    pub year: u32,
    pub month: u32,
    #[serde(default)]
    pub highlighted_days: Vec<u32>,
}

impl CalendarGrid {
    pub fn new(year: u32, month: u32, highlighted_days: Vec<u32>) -> Self {
        Self {
            year,
            month,
            highlighted_days,
        }
    }

    /// Returns the number of days in the current month for the specified year.
    pub fn days_in_month(&self) -> u32 {
        days_in_month(self.year, self.month)
    }

    /// Returns the day of the week for the 1st of the month (0 = Sunday, 1 = Monday, ..., 6 = Saturday).
    pub fn starting_day_of_week(&self) -> u32 {
        starting_day_of_week(self.year, self.month)
    }

    /// Checks if the current year is a leap year.
    pub fn is_leap_year(&self) -> bool {
        is_leap_year(self.year)
    }

    /// Checks whether a specific day is highlighted in the grid.
    pub fn is_day_highlighted(&self, day: u32) -> bool {
        self.highlighted_days.contains(&day)
    }

    /// Adds a day to the highlighted list if not already present.
    pub fn highlight_day(&mut self, day: u32) {
        if !self.highlighted_days.contains(&day) {
            self.highlighted_days.push(day);
        }
    }

    /// Removes a day from the highlighted list.
    pub fn unhighlight_day(&mut self, day: u32) {
        self.highlighted_days.retain(|&d| d != day);
    }
}

/// A single event entry in an agenda list.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AgendaEvent {
    pub title: String,
    pub time: String,
    pub tag: String,
}

impl AgendaEvent {
    pub fn new(title: impl Into<String>, time: impl Into<String>, tag: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            time: time.into(),
            tag: tag.into(),
        }
    }
}

/// An agenda list containing scheduled events.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AgendaList {
    #[serde(default)]
    pub events: Vec<AgendaEvent>,
}

impl AgendaList {
    pub fn new(events: Vec<AgendaEvent>) -> Self {
        Self { events }
    }

    /// Adds an event to the agenda list.
    pub fn add_event(&mut self, event: AgendaEvent) {
        self.events.push(event);
    }

    /// Appends a new event fluently.
    pub fn push_event(
        &mut self,
        title: impl Into<String>,
        time: impl Into<String>,
        tag: impl Into<String>,
    ) -> &mut Self {
        self.events.push(AgendaEvent::new(title, time, tag));
        self
    }

    /// Filters and returns references to events matching a given tag.
    pub fn events_for_tag(&self, tag: &str) -> Vec<&AgendaEvent> {
        self.events.iter().filter(|e| e.tag == tag).collect()
    }
}

/// Determines whether a year is a leap year in the Gregorian calendar.
pub fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Calculates the number of days in a given month (1..=12) for a given year.
/// Returns 0 for invalid month numbers.
pub fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Computes the day of the week for a specific date (year, month 1..=12, day 1..=31).
/// Uses Sakamoto's algorithm.
/// Returns: 0 = Sunday, 1 = Monday, 2 = Tuesday, 3 = Wednesday, 4 = Thursday, 5 = Friday, 6 = Saturday.
pub fn day_of_week(year: u32, month: u32, day: u32) -> u32 {
    if month < 1 || month > 12 || day < 1 || day > 31 {
        return 0;
    }
    let t = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let y = if month < 3 {
        year.saturating_sub(1)
    } else {
        year
    };
    (y + y / 4 - y / 100 + y / 400 + t[(month - 1) as usize] + day) % 7
}

/// Computes the starting day of the week for the 1st day of the given month (1..=12) and year.
/// Returns: 0 = Sunday, 1 = Monday, 2 = Tuesday, 3 = Wednesday, 4 = Thursday, 5 = Friday, 6 = Saturday.
pub fn starting_day_of_week(year: u32, month: u32) -> u32 {
    day_of_week(year, month, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2000));
        assert!(is_leap_year(2024));
        assert!(is_leap_year(2028));
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(2021));
        assert!(!is_leap_year(2022));
        assert!(!is_leap_year(2023));
        assert!(!is_leap_year(2025));
        assert!(!is_leap_year(2026));
    }

    #[test]
    fn test_days_in_month() {
        // 31 day months
        assert_eq!(days_in_month(2026, 1), 31);
        assert_eq!(days_in_month(2026, 3), 31);
        assert_eq!(days_in_month(2026, 5), 31);
        assert_eq!(days_in_month(2026, 7), 31);
        assert_eq!(days_in_month(2026, 8), 31);
        assert_eq!(days_in_month(2026, 10), 31);
        assert_eq!(days_in_month(2026, 12), 31);

        // 30 day months
        assert_eq!(days_in_month(2026, 4), 30);
        assert_eq!(days_in_month(2026, 6), 30);
        assert_eq!(days_in_month(2026, 9), 30);
        assert_eq!(days_in_month(2026, 11), 30);

        // February
        assert_eq!(days_in_month(2024, 2), 29); // leap year
        assert_eq!(days_in_month(2026, 2), 28); // common year
        assert_eq!(days_in_month(2000, 2), 29); // 400-year leap
        assert_eq!(days_in_month(1900, 2), 28); // 100-year non-leap

        // Invalid month
        assert_eq!(days_in_month(2026, 0), 0);
        assert_eq!(days_in_month(2026, 13), 0);
    }

    #[test]
    fn test_starting_day_of_week() {
        // 2026-08-01 is Saturday (6)
        assert_eq!(starting_day_of_week(2026, 8), 6);

        // 2024-02-01 is Thursday (4)
        assert_eq!(starting_day_of_week(2024, 2), 4);

        // 2025-01-01 is Wednesday (3)
        assert_eq!(starting_day_of_week(2025, 1), 3);

        // 2023-01-01 is Sunday (0)
        assert_eq!(starting_day_of_week(2023, 1), 0);

        // 2021-01-01 is Friday (5)
        assert_eq!(starting_day_of_week(2021, 1), 5);

        // 2021-02-01 is Monday (1)
        assert_eq!(starting_day_of_week(2021, 2), 1);

        // 2022-02-01 is Tuesday (2)
        assert_eq!(starting_day_of_week(2022, 2), 2);
    }

    #[test]
    fn test_calendar_grid_methods() {
        let mut cal = CalendarGrid::new(2026, 8, vec![1, 15, 23]);
        assert_eq!(cal.days_in_month(), 31);
        assert_eq!(cal.starting_day_of_week(), 6);
        assert!(!cal.is_leap_year());

        assert!(cal.is_day_highlighted(15));
        assert!(cal.is_day_highlighted(23));
        assert!(!cal.is_day_highlighted(24));

        cal.highlight_day(24);
        assert!(cal.is_day_highlighted(24));

        cal.unhighlight_day(15);
        assert!(!cal.is_day_highlighted(15));
    }

    #[test]
    fn test_agenda_list_methods() {
        let mut agenda = AgendaList::default();
        agenda.push_event("Sprint Review", "10:00 AM", "work");
        agenda.push_event("Doctor Appointment", "02:30 PM", "personal");
        agenda.push_event("Architecture Sync", "04:00 PM", "work");

        assert_eq!(agenda.events.len(), 3);
        let work_events = agenda.events_for_tag("work");
        assert_eq!(work_events.len(), 2);
        assert_eq!(work_events[0].title, "Sprint Review");
        assert_eq!(work_events[1].title, "Architecture Sync");

        let personal_events = agenda.events_for_tag("personal");
        assert_eq!(personal_events.len(), 1);
        assert_eq!(personal_events[0].title, "Doctor Appointment");
    }

    #[test]
    fn test_calendar_and_agenda_serde() {
        let cal = CalendarGrid::new(2026, 12, vec![24, 25, 31]);
        let json_cal = serde_json::to_string(&cal).expect("Must serialize CalendarGrid");
        let de_cal: CalendarGrid = serde_json::from_str(&json_cal).expect("Must deserialize CalendarGrid");
        assert_eq!(cal, de_cal);

        let mut agenda = AgendaList::default();
        agenda.push_event("Xmas Eve", "18:00", "holiday");
        let json_agenda = serde_json::to_string(&agenda).expect("Must serialize AgendaList");
        let de_agenda: AgendaList = serde_json::from_str(&json_agenda).expect("Must deserialize AgendaList");
        assert_eq!(agenda, de_agenda);
    }
}
