use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::repository::VatRepository2;
use dirs_next::{config_dir, document_dir};
use std::fs;


const CONFIG_FILE_NAME: &str = "vat.config";


#[derive(Serialize, Deserialize, Debug)]
pub struct VatConfig{
    pub repository_path: Option<PathBuf>,   
    pub packages_path: Option<PathBuf>,

}

impl VatConfig {
    pub fn default() -> Self{
        let vat_config_path = Self::get_app_dir().unwrap();
        let default_repo_path = vat_config_path.join("repository");
        let default_packages_path = vat_config_path.join("packages");
        let result_repo = fs::create_dir_all(&default_repo_path);
        if result_repo.is_err(){
            println!("Failed to create repository directory: {:?}", result_repo.err());
        }
        let result_packages = fs::create_dir_all(&default_packages_path);
        if result_packages.is_err(){
            println!("Failed to create packages directory: {:?}", result_packages.err());
        }

        // initalize repository
        let result_repo =   VatRepository2::initalize_repository(&default_repo_path);
        if result_repo.is_err(){
            println!("Failed to initalize repository: {:?}", result_repo.err());
        }

        VatConfig{
            repository_path: Some(default_repo_path),
            packages_path: Some(default_packages_path),
        }
    }


    pub fn init() -> Result<Self, anyhow::Error> {

        let app_dir = VatConfig::get_app_dir();
        if let Some(app_dir) = app_dir.clone(){

            if !app_dir.exists(){   
                let result = fs::create_dir_all(&app_dir);
                if result.is_err(){
                    println!("Failed to create app directory: {:?}", result.err());
                }else{
                    let config_path = app_dir.join(CONFIG_FILE_NAME);
                    let config = VatConfig::default();
                    let config_str = serde_json::to_string(&config).unwrap();
                    fs::write(config_path, config_str).unwrap();
                    return Ok(config);
                }
            } else{
                let config_path = app_dir.join(CONFIG_FILE_NAME);
                if config_path.exists(){
                    let config = serde_json::from_str(&fs::read_to_string(config_path).unwrap()).unwrap();
                    return Ok(config);
                }else{
                    let config = VatConfig::default();
                    let config_str = serde_json::to_string(&config).unwrap();
                    fs::write(config_path, config_str).unwrap();
                    return Ok(config);
                }
            }
        }
        Err(anyhow::anyhow!("Failed to create or read config file"))
    }

    pub fn get_repository_path(&self) -> Option<PathBuf> {
        self.repository_path.clone()
    }

    pub fn set_repository_path(&mut self, path: PathBuf) {
        self.repository_path = Some(path);
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        let config_path = VatConfig::get_app_dir().unwrap().join(CONFIG_FILE_NAME);
        let config_str = serde_json::to_string(&self).unwrap();
        fs::write(config_path, config_str).unwrap();
        Ok(())
    }

    pub fn get_app_dir() -> Option<PathBuf> {
        let app_name = String::from("Vat");

        if cfg!(target_os = "macos"){
            config_dir().map(|path| path.join(app_name))
        }
        else if cfg!(target_os = "windows") {
            document_dir().map(|path| path.join(app_name))
        } else {
            config_dir().map(|path| path.join(app_name))
        }
    }
}
