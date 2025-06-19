use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};
use std::ops::RangeInclusive;
use chrono::NaiveDate;

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct AnalyticsRequest {
    #[validate(range(min = 1, max = 100, message = "Referer quantity must be between 1 and 100"))]
    pub referer_quantity: Option<i64>,
    
    #[validate(range(min = 1, max = 50, message = "User agent quantity must be between 1 and 50"))]
    pub user_agent_quantity: Option<i64>,
    
    #[validate(range(min = 1, max = 365, message = "Click distribution quantity must be between 1 and 365"))]
    pub click_distribution_quantity: Option<i64>,
    
    #[validate(custom(function = "validate_date_format", message = "Start date must be in YYYY-MM-DD format"))]
    pub start_date: Option<String>,
    
    #[validate(custom(function = "validate_date_format", message = "End date must be in YYYY-MM-DD format"))]
    pub end_date: Option<String>,
}

impl AnalyticsRequest {
    // Method to validate the relationship between start and end dates
    pub fn validate_date_range(&self) -> Result<(), ValidationError> {
        if let (Some(start_str), Some(end_str)) = (&self.start_date, &self.end_date) {
            match (NaiveDate::parse_from_str(start_str, "%Y-%m-%d"), 
                   NaiveDate::parse_from_str(end_str, "%Y-%m-%d")) {
                (Ok(start_date), Ok(end_date)) => {
                    if end_date < start_date {
                        return Err(ValidationError::new("End date cannot be before start date"));
                    }
                    
                    // Also validate that the range isn't too large (e.g., limit to 1 year)
                    let max_days = 366; // Allow for leap years
                    if (end_date - start_date).num_days() > max_days {
                        return Err(ValidationError::new("Date range cannot exceed 1 year"));
                    }
                },
                _ => {
                    // If we can't parse the dates, the individual validators will catch that
                    // so we don't need to return an error here
                }
            }
        }
        
        Ok(())
    }
}

// Custom validator for date format (YYYY-MM-DD)
fn validate_date_format(date: &str) -> Result<(), ValidationError> {
    // Check if the date format is YYYY-MM-DD
    if date.len() != 10 {
        return Err(ValidationError::new("Invalid date length"));
    }
    
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return Err(ValidationError::new("Date must contain two hyphens"));
    }
    
    // Validate year
    if parts[0].len() != 4 || parts[0].parse::<i32>().is_err() {
        return Err(ValidationError::new("Year must be a 4-digit number"));
    }
    
    // Validate month
    let month_range: RangeInclusive<u8> = 1..=12;
    if parts[1].len() != 2 || parts[1].parse::<u8>().map_or(true, |m| !month_range.contains(&m)) {
        return Err(ValidationError::new("Month must be a 2-digit number between 01 and 12"));
    }
    
    // Validate day
    let day_range: RangeInclusive<u8> = 1..=31;
    if parts[2].len() != 2 || parts[2].parse::<u8>().map_or(true, |d| !day_range.contains(&d)) {
        return Err(ValidationError::new("Day must be a 2-digit number between 01 and 31"));
    }
    
    // More precise validation using chrono
    if NaiveDate::parse_from_str(date, "%Y-%m-%d").is_err() {
        return Err(ValidationError::new("Invalid date"));
    }
    
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReferrerData {
    pub referer: String,
    pub count: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserAgentData {
    pub user_agent: String,
    pub count: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClickDistributionData {
    pub date: String,
    pub count: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DateRange {
    pub start: String,
    pub end: String,
    pub days: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnalyticsData {
    pub total_clicks: i64,
    pub unique_clicks: i64,
    pub top_referrers: Vec<ReferrerData>,
    pub top_user_agents: Vec<UserAgentData>,
    pub click_distribution: Vec<ClickDistributionData>,
    pub date_range: Option<DateRange>,
}
