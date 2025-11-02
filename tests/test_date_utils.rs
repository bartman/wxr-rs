use chrono::NaiveDate;
use wxrust::utils::{parse_date_boundary, parse_date_range};

#[test]
fn test_parse_date_boundary_full_date() {
    // Full date ignores the end parameter
    let date = parse_date_boundary("2025-05-27", false).unwrap();
    assert_eq!(date, NaiveDate::from_ymd_opt(2025, 5, 27).unwrap());

    let date = parse_date_boundary("2025-05-27", true).unwrap();
    assert_eq!(date, NaiveDate::from_ymd_opt(2025, 5, 27).unwrap());

    // Test with different separators
    let date = parse_date_boundary("2025/05/27", false).unwrap();
    assert_eq!(date, NaiveDate::from_ymd_opt(2025, 5, 27).unwrap());

    let date = parse_date_boundary("2025.05.27", false).unwrap();
    assert_eq!(date, NaiveDate::from_ymd_opt(2025, 5, 27).unwrap());
}

#[test]
fn test_parse_date_boundary_compact_yyyymmdd() {
    let date = parse_date_boundary("20250527", false).unwrap();
    assert_eq!(date, NaiveDate::from_ymd_opt(2025, 5, 27).unwrap());

    let date = parse_date_boundary("20250527", true).unwrap();
    assert_eq!(date, NaiveDate::from_ymd_opt(2025, 5, 27).unwrap());
}

#[test]
fn test_parse_date_boundary_month_only_end_false() {
    let date = parse_date_boundary("2025-05", false).unwrap();
    assert_eq!(date, NaiveDate::from_ymd(2025, 5, 1));

    // Compact
    let date = parse_date_boundary("202505", false).unwrap();
    assert_eq!(date, NaiveDate::from_ymd(2025, 5, 1));
}

#[test]
fn test_parse_date_boundary_month_only_end_true() {
    let date = parse_date_boundary("2025-05", true).unwrap();
    assert_eq!(date, NaiveDate::from_ymd(2025, 5, 31));

    // Compact
    let date = parse_date_boundary("202505", true).unwrap();
    assert_eq!(date, NaiveDate::from_ymd(2025, 5, 31));

    // December
    let date = parse_date_boundary("2025-12", true).unwrap();
    assert_eq!(date, NaiveDate::from_ymd(2025, 12, 31));
}

#[test]
fn test_parse_date_boundary_year_only_end_false() {
    let date = parse_date_boundary("2025", false).unwrap();
    assert_eq!(date, NaiveDate::from_ymd(2025, 1, 1));

    // Compact
    let date = parse_date_boundary("2025", false).unwrap();
    assert_eq!(date, NaiveDate::from_ymd(2025, 1, 1));
}

#[test]
fn test_parse_date_boundary_year_only_end_true() {
    let date = parse_date_boundary("2025", true).unwrap();
    assert_eq!(date, NaiveDate::from_ymd(2025, 12, 31));
}

#[test]
fn test_parse_date_boundary_invalid() {
    // Invalid year length
    assert!(parse_date_boundary("202", false).is_err());

    // Invalid month
    assert!(parse_date_boundary("2025-13", false).is_err());

    // Invalid day
    assert!(parse_date_boundary("2025-05-32", false).is_err());

    // Invalid compact
    assert!(parse_date_boundary("2025052", false).is_err());

    // Too many parts
    assert!(parse_date_boundary("2025-05-27-01", false).is_err());

    // Empty
    assert!(parse_date_boundary("", false).is_err());
}

#[test]
fn test_parse_date_range_single_date() {
    let (start, end) = parse_date_range("2025-05-27").unwrap();
    assert_eq!(start, NaiveDate::from_ymd(2025, 5, 27));
    assert_eq!(end, NaiveDate::from_ymd(2025, 5, 27));

    // Compact
    let (start, end) = parse_date_range("20250527").unwrap();
    assert_eq!(start, NaiveDate::from_ymd(2025, 5, 27));
    assert_eq!(end, NaiveDate::from_ymd(2025, 5, 27));
}

#[test]
fn test_parse_date_range_with_separator() {
    let (start, end) = parse_date_range("2025-05-01..2025-05-31").unwrap();
    assert_eq!(start, NaiveDate::from_ymd(2025, 5, 1));
    assert_eq!(end, NaiveDate::from_ymd(2025, 5, 31));

    // Different separators
    let (start, end) = parse_date_range("2025/05/01..2025/05/31").unwrap();
    assert_eq!(start, NaiveDate::from_ymd(2025, 5, 1));
    assert_eq!(end, NaiveDate::from_ymd(2025, 5, 31));
}

#[test]
fn test_parse_date_range_compact() {
    let (start, end) = parse_date_range("20250501..20250531").unwrap();
    assert_eq!(start, NaiveDate::from_ymd(2025, 5, 1));
    assert_eq!(end, NaiveDate::from_ymd(2025, 5, 31));
}

#[test]
fn test_parse_date_range_month_range() {
    let (start, end) = parse_date_range("2025-05").unwrap();
    assert_eq!(start, NaiveDate::from_ymd(2025, 5, 1));
    assert_eq!(end, NaiveDate::from_ymd(2025, 5, 31));
}

#[test]
fn test_parse_date_range_year_range() {
    let (start, end) = parse_date_range("2025").unwrap();
    assert_eq!(start, NaiveDate::from_ymd(2025, 1, 1));
    assert_eq!(end, NaiveDate::from_ymd(2025, 12, 31));
}

#[test]
fn test_parse_date_range_invalid() {
    // Invalid date
    assert!(parse_date_range("invalid").is_err());

    // Too many parts
    assert!(parse_date_range("2025-05-01..2025-05-31..extra").is_err());
}