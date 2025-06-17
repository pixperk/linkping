pub fn validate_scheme(url: &str) -> Result<(), validator::ValidationError> {
    let parsed = url::Url::parse(url);
    match parsed {
        Ok(u) if u.scheme() == "http" || u.scheme() == "https" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_scheme")),
    }
}

pub fn validate_expiry(value : &str) -> Result<(), validator::ValidationError> {
    match humantime::parse_duration(value) {
        Ok(_) => Ok(()),
        Err(_) => {
            let mut err = validator::ValidationError::new("invalid_expiry");
            err.message = Some("Expiry must be a valid duration like '1d', '6h', '30m'".into());
            Err(err)
        }
    }
}