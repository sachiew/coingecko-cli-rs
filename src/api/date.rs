pub(crate) fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

pub(crate) fn days_in_month(m: u32, y: i32) -> i64 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap(y) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

pub(crate) fn ymd_to_unix(year: i32, month: u32, day: u32) -> i64 {
    let mut days: i64 = 0;
    for y in 1970..year {
        days += if is_leap(y) { 366 } else { 365 };
    }
    for m in 1..month {
        days += days_in_month(m, year);
    }
    days += i64::from(day - 1);
    days * 86400
}

pub(crate) fn unix_to_ymd(ts_sec: i64) -> String {
    let mut days = ts_sec / 86400;
    let mut year = 1970i32;
    loop {
        let dy = if is_leap(year) { 366 } else { 365 };
        if days < dy {
            break;
        }
        days -= dy;
        year += 1;
    }
    let mut month = 1u32;
    loop {
        let dm = days_in_month(month, year);
        if days < dm {
            break;
        }
        days -= dm;
        month += 1;
    }
    format!("{}-{:02}-{:02}", year, month, days + 1)
}

pub(crate) fn parse_ymd(s: &str) -> Option<(i32, u32, u32)> {
    let p: Vec<&str> = s.splitn(3, '-').collect();
    if p.len() < 3 {
        return None;
    }
    Some((p[0].parse().ok()?, p[1].parse().ok()?, p[2].parse().ok()?))
}

pub(crate) fn to_api_date(s: &str) -> Option<String> {
    let (y, m, d) = parse_ymd(s)?;
    Some(format!("{d:02}-{m:02}-{y}"))
}

/// Convert a millisecond f64 timestamp (from JSON) to a date string.
pub(crate) fn ms_to_date(ms: f64) -> String {
    #[allow(clippy::cast_possible_truncation)]
    let secs = (ms as i64) / 1000;
    unix_to_ymd(secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── is_leap ─────────────────────────────────────────────────────────────

    #[test]
    fn leap_year_divisible_by_400() {
        assert!(is_leap(2000));
    }

    #[test]
    fn not_leap_year_divisible_by_100() {
        assert!(!is_leap(1900));
    }

    #[test]
    fn leap_year_divisible_by_4() {
        assert!(is_leap(2024));
    }

    #[test]
    fn not_leap_year_regular() {
        assert!(!is_leap(2023));
    }

    // ── days_in_month ───────────────────────────────────────────────────────

    #[test]
    fn days_in_month_31_day_months() {
        for m in [1, 3, 5, 7, 8, 10, 12] {
            assert_eq!(days_in_month(m, 2023), 31, "month {m} should have 31 days");
        }
    }

    #[test]
    fn days_in_month_30_day_months() {
        for m in [4, 6, 9, 11] {
            assert_eq!(days_in_month(m, 2023), 30, "month {m} should have 30 days");
        }
    }

    #[test]
    fn days_in_month_feb_leap() {
        assert_eq!(days_in_month(2, 2024), 29);
    }

    #[test]
    fn days_in_month_feb_non_leap() {
        assert_eq!(days_in_month(2, 2023), 28);
    }

    #[test]
    fn days_in_month_invalid() {
        assert_eq!(days_in_month(13, 2023), 0);
    }

    // ── ymd_to_unix ─────────────────────────────────────────────────────────

    #[test]
    fn ymd_to_unix_epoch() {
        assert_eq!(ymd_to_unix(1970, 1, 1), 0);
    }

    #[test]
    fn ymd_to_unix_known_date() {
        // 2024-01-01 00:00:00 UTC = 1704067200
        assert_eq!(ymd_to_unix(2024, 1, 1), 1_704_067_200);
    }

    // ── unix_to_ymd ─────────────────────────────────────────────────────────

    #[test]
    fn unix_to_ymd_epoch() {
        assert_eq!(unix_to_ymd(0), "1970-01-01");
    }

    #[test]
    fn unix_to_ymd_known_timestamp() {
        assert_eq!(unix_to_ymd(1_704_067_200), "2024-01-01");
    }

    // ── roundtrip ───────────────────────────────────────────────────────────

    #[test]
    fn ymd_roundtrip() {
        let dates = [(1970, 1, 1), (2000, 2, 29), (2023, 12, 31), (2024, 6, 15)];
        for (y, m, d) in dates {
            let ts = ymd_to_unix(y, m, d);
            assert_eq!(
                unix_to_ymd(ts),
                format!("{y}-{m:02}-{d:02}"),
                "roundtrip failed for {y}-{m:02}-{d:02}"
            );
        }
    }

    // ── parse_ymd ───────────────────────────────────────────────────────────

    #[test]
    fn parse_ymd_valid() {
        assert_eq!(parse_ymd("2024-01-15"), Some((2024, 1, 15)));
    }

    #[test]
    fn parse_ymd_too_few_parts() {
        assert_eq!(parse_ymd("2024-01"), None);
    }

    #[test]
    fn parse_ymd_non_numeric() {
        assert_eq!(parse_ymd("abc-01-01"), None);
    }

    // ── to_api_date ─────────────────────────────────────────────────────────

    #[test]
    fn to_api_date_valid() {
        assert_eq!(to_api_date("2024-01-15"), Some("15-01-2024".to_string()));
    }

    #[test]
    fn to_api_date_invalid() {
        assert_eq!(to_api_date("bad-date"), None);
    }

    // ── ms_to_date ──────────────────────────────────────────────────────────

    #[test]
    fn ms_to_date_epoch() {
        assert_eq!(ms_to_date(0.0), "1970-01-01");
    }

    #[test]
    fn ms_to_date_known() {
        // 1704067200 * 1000 = 1704067200000.0
        assert_eq!(ms_to_date(1_704_067_200_000.0), "2024-01-01");
    }
}
