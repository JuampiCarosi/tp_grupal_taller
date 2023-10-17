pub struct GitVersion;

impl GitVersion {
    pub fn from(_args: Vec<String>) -> Result<GitVersion, String> {
        Ok(GitVersion)
    }

    pub fn ejecutar(&self) -> Result<(), String> {
        Self::imprimir_version();
        Ok(())
    }

    fn imprimir_version() {
        println!("git version 0.0.1");
    }
}
