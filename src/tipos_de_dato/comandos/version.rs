pub struct Version;

impl Version {
    pub fn from(_args: Vec<String>) -> Result<Version, String> {
        Ok(Version)
    }

    pub fn ejecutar(&self) -> Result<(), String> {
        Self::imprimir_version();
        Ok(())
    }

    fn imprimir_version() {
        println!("git version 0.0.1");
    }
}
