use std::path::PathBuf;
use std::fs;
use crate::config::VatConfig;
use serde::{Serialize, Deserialize};
use color_print::cprintln;

#[derive(Serialize, Deserialize)]
pub struct VatRepository {
    pub packages: Vec<String>
}

impl Default for VatRepository {
    fn default() -> Self {
        VatRepository { packages: vec![] }
    }
}

impl VatRepository {
    pub fn init() -> Result<Self, anyhow::Error> {
        let currect_dir = std::env::current_dir()?;
        let mut config = VatConfig::init()?;
        if let Some(repository_path) = config.get_repository_path(){
            if repository_path.exists(){
                return Err(anyhow::anyhow!("Repository has been already initialized, {}", repository_path.display()));
            }
        }else{
            // check if current dir is empty
            if !currect_dir.is_empty(){
                return Err(anyhow::anyhow!("Current directory is not empty"));
            }else{
                let repository_config_file = currect_dir.join("Vat.repository.toml");
                if repository_config_file.exists(){
                    return Err(anyhow::anyhow!("Repository config file already exists, {}", repository_config_file.display()));
                }else{
                    let repository_config = VatRepository::default();
                    let repository_config_str = toml::to_string(&repository_config)?;
                    fs::write(repository_config_file, repository_config_str)?;
                    config.set_repository_path(currect_dir.clone());
                    config.save()?;
                    cprintln!("      <green>Repository initialized</green>");

                }
            }
        }
        Ok(VatRepository { packages: vec![] })
    }
}



trait Path {
    fn is_empty(&self) -> bool;
}

impl Path for PathBuf {
    fn is_empty(&self) -> bool {
        let files = std::fs::read_dir(self).unwrap();
        if files.count() == 0 {
            return true;
        }
        false
    }
}
