use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[validate(schema(function = "validate_time_zone", skip_on_field_errors = false))]
#[serde(rename_all = "snake_case")]
pub struct TimeZone {
    name: String,
    offset: i32,
}

fn validate_time_zone(time_zone: &TimeZone) -> Result<(), ValidationError> {
    match time_zone.name.parse::<chrono_tz::Tz>() {
        Ok(_) => Ok(()),
        Err(_) => Err(ValidationError::new("Invalid time zone name")),
    }?;

    if !(-24 * 60 <= time_zone.offset && time_zone.offset <= 24 * 60) {
        return Err(ValidationError::new(
            "Offset must be between -24*60 and 24*60 minutes",
        ));
    }

    Ok(())
}
