pub struct Version;

impl Version {
    pub fn from(_args: Vec<String>) -> Result<Version, String> {
        Ok(Version)
    }

    pub fn ejecutar(&self) -> Result<String, String> {
        Ok("git version 0.0.1".to_string())
    }
}
