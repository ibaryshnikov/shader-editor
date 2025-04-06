use naga::valid::{Capabilities, ValidationFlags, Validator};

pub fn validate(shader: &str) -> Result<(), String> {
    let parsed =
        naga::front::wgsl::parse_str(shader).map_err(|parse_error| parse_error.to_string())?;
    let _ = Validator::new(ValidationFlags::default(), Capabilities::all())
        .validate(&parsed)
        .map_err(|e| e.to_string())?;
    Ok(())
}
