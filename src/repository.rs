use std::path::PathBuf;
use std::fs;
use crate::config::VatConfig;
use serde::{Serialize, Deserialize};
use color_print::cprintln;
use std::collections::HashMap;
use crate::package::{Package, PackageVersions};
use std::io::Write;
use fs_extra::dir::{copy, CopyOptions};
use crate::package::RepositoryType;

#[derive(Serialize, Deserialize, Debug)]
pub struct VatRepository {
    pub packages: HashMap<String, PackageVersions>
}

impl Default for VatRepository {
    fn default() -> Self {
        VatRepository { packages: HashMap::new() }
    }
}

impl VatRepository {
    pub fn init() -> Result<Self, anyhow::Error> {
        let mut config = VatConfig::init()?;
        let repository_path = config.get_repository_path();
        if repository_path.is_none(){
            let current_dir = std::env::current_dir()?;
            let repository_config_file = current_dir.join("vat.repository.toml");
            if !current_dir.is_empty(){
                if repository_config_file.exists(){
                    return Err(anyhow::anyhow!("Repository already exists, {}", repository_path.unwrap().display()));
                }else{
                    return Err(anyhow::anyhow!("Current directory is not empty to initialize the repository"));
                }
            }else{
                let repository_config = VatRepository::default();
                let repository_config_str = toml::to_string(&repository_config)?;
                fs::write(repository_config_file, repository_config_str)?;
                config.set_repository_path(current_dir.clone());
                config.save()?;
                cprintln!("      <green>Repository initialized</green>");
            }
        }else{
            let repository_path = config.get_repository_path().unwrap();
            let repository_config_file = repository_path.join("vat.repository.toml");
            if repository_config_file.exists(){
                return Err(anyhow::anyhow!("Repository config file already exists, {}", repository_config_file.display()));
            }else{
                let repository_config = VatRepository::default();
                let repository_config_str = toml::to_string(&repository_config)?;
                fs::write(repository_config_file, repository_config_str)?;
                config.set_repository_path(repository_path.clone());
                config.save()?;
                cprintln!("      <green>Repository initialized</green>");
            }
        }
    
        Ok(VatRepository::default())
    }

    pub fn read_repository() -> Result<Self, anyhow::Error> {
        let vat_config = VatConfig::init()?;
        let repository_path = vat_config.get_repository_path();
        let repository_path = if let Some(repository_path) = repository_path {
            repository_path
        } else {
            return Err(anyhow::anyhow!("Repository path not found, Run vat repo-init to initialize the repository"));
        };
        let repository_config_file = repository_path.join("vat.repository.toml");
        let repository_config_str = fs::read_to_string(repository_config_file)?;
        let repository_config: VatRepository = toml::from_str(&repository_config_str)?;
        Ok(repository_config)
    }

    pub fn add_package(&mut self, package: Package) -> Result<Package, anyhow::Error> {
        if self.packages.contains_key(&package.get_name()){
            let package_versions = self.packages.get_mut(&package.get_name()).unwrap();
            if package_versions.publishes.contains_key(&package.get_version()){
                return Err(anyhow::anyhow!("Package version has already been published, {}", package.get_version()));
            }
            package_versions.append_version(package.clone());
            self.save()?;
            Ok(package)
        }else{
            let package_name = package.get_name();
            let package_versions = PackageVersions::from(package.clone());
            self.packages.insert(package_name, package_versions);
            self.save()?;
            Ok(package)
        }

    }

    pub fn get_repository_path() -> Option<PathBuf> {
        let vat_config = VatConfig::init().unwrap();
        let repository_path = vat_config.get_repository_path();
        repository_path
    }   

    pub fn get_package_version_path(package: &Package) -> Option<PathBuf> {
        let version_path = Self::get_repository_path().unwrap().join("packages").join(&package.get_name()).join(&package.get_version());
        Some(version_path)
    }

    pub fn package_data_update(package: &Package) -> Result<(), anyhow::Error> {
        match &package.package_info.repository {
            RepositoryType::Local(path) => {
                println!("Local repository: {:?}", path);
                let current_dir = path.clone();
                let version_path = Self::get_package_version_path(package).unwrap();
                let mut options = CopyOptions::new();
                options.overwrite = true;
                options.copy_inside = true;  
                copy(&current_dir, &version_path, &options)?;

                // remove .git folder #TODO: This is a temporary solution, need to find a better way to handle this
                let git_folder = version_path.join(".git");
                if git_folder.exists() {
                    fs::remove_dir_all(git_folder)?;
                }
            }
            RepositoryType::Remote(url) => {
                println!("Remote repository: {}", url);
            }
        }
        Ok(())
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        let vat_config = VatConfig::init()?;
        let repository_path = vat_config.get_repository_path();
        let repository_path = if let Some(repository_path) = repository_path {
            repository_path
        } else {
            return Err(anyhow::anyhow!("Repository path not found, Run vat repo-init to initialize the repository"));
        };
        let toml_string = toml::to_string(self)?;
        let mut toml_file = std::fs::File::create(repository_path.join("vat.repository.toml"))?;
        toml_file.write_all(toml_string.as_bytes())?;
        Ok(())
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
