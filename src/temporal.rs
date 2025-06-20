use std::{
    fmt::{self},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, PartialEq)]
pub enum Month {
    Jan,
    Feb,
    Mar,
    Apr,
    May,
    Jun,
    Jul,
    Aug,
    Sep,
    Oct,
    Nov,
    Dec,
}

impl Month {
    fn from_number(n: u64) -> Result<Self, &'static str> {
        match n {
            1 => Ok(Month::Jan),
            2 => Ok(Month::Feb),
            3 => Ok(Month::Mar),
            4 => Ok(Month::Apr),
            5 => Ok(Month::May),
            6 => Ok(Month::Jun),
            7 => Ok(Month::Jul),
            8 => Ok(Month::Aug),
            9 => Ok(Month::Sep),
            10 => Ok(Month::Oct),
            11 => Ok(Month::Nov),
            12 => Ok(Month::Dec),
            _ => Err("Invalid month number"),
        }
    }

    fn full_name(&self) -> &'static str {
        match self {
            Month::Jan => "January",
            Month::Feb => "February",
            Month::Mar => "March",
            Month::Apr => "April",
            Month::May => "May",
            Month::Jun => "June",
            Month::Jul => "July",
            Month::Aug => "August",
            Month::Sep => "September",
            Month::Oct => "October",
            Month::Nov => "November",
            Month::Dec => "December",
        }
    }
}

impl fmt::Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            // {:#} - Full month name
            write!(f, "{}", self.full_name())
        } else {
            // Default - 3-letter abbreviation
            write!(f, "{:?}", self)
        }
    }
    // TODO Complete the display trait
}

#[derive(Debug, PartialEq)]
pub struct Date {
    pub year: u64,
    pub month: Month,
    pub day: u64,
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:-02}", self.year, self.month, self.day)
    }
}

// Gregorian calendar leap years
pub fn is_leap_year(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

pub fn days_in_year(year: u64) -> u64 {
    if is_leap_year(year) { 366 } else { 365 }
}

pub fn days_in_month(year: u64, month: u64) -> u64 {
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
        _ => panic!("Invalid month: {}", month),
    }
}

// TODO ERROR HANDLING
pub fn today() -> Date {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Could not get current time");

    let total_seconds = now.as_secs();
    let mut remaining_days = total_seconds / (24 * 60 * 60);

    let mut year = 1970;

    while remaining_days >= days_in_year(year) {
        remaining_days -= days_in_year(year);
        year += 1;
    }

    let mut month = 1;
    while remaining_days >= days_in_month(year, month) {
        remaining_days -= days_in_month(year, month);
        month += 1;
    }

    let month = Month::from_number(month).expect("Could not unwrap month");

    // The remaining days + 1 is the day of the month (1-indexed)
    let day = remaining_days + 1;

    Date { year, month, day }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_today() {
        let date = today();
        println!("{:?}", date);
        assert_eq!(
            date,
            Date {
                year: 2025,
                month: Month::Jun,
                day: 20
            }
        );
    }
}
