use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::package::Package;
use crate::config::VatConfig;
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PackageRegistry{
    pub path: PathBuf,
    pub description: String,
    pub repository: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Registry{
    pub registry: HashMap<String, PackageRegistry>,
}

impl Default for Registry{
    fn default() -> Self{
        Registry{registry: HashMap::new()}
    }
}

impl Registry{

    pub fn init() -> Result<Self, anyhow::Error>{
        let registry_path = Self::registry_path();
        if !registry_path.exists(){
            let registry = Registry::default();
            let registry_str = serde_json::to_string(&registry).unwrap();
            fs::write(&registry_path, registry_str).unwrap();
            return Ok(registry);
        }

        let registry_str = fs::read_to_string(&registry_path).unwrap();
        let registry: Registry = serde_json::from_str(&registry_str).unwrap();
        Ok(registry)
    }

    pub fn registry_path() -> PathBuf{
        let app_dir = VatConfig::get_app_dir().unwrap();
        let registry_path = app_dir.join("registry.toml");
        registry_path
    }


    pub fn add_package(&mut self, package: Package, path: PathBuf) -> Result<(), anyhow::Error>{
        if self.registry.contains_key(&package.package_info.name){
            return Err(anyhow::anyhow!("Package already exists"));
        }
        if package.package_info.description.is_none(){
            return Err(anyhow::anyhow!("Package description is required to register"));
        }

        let package_registry = PackageRegistry{
            path: path,
            description: package.package_info.description.unwrap(),
            repository: package.package_info.repository.clone(),
        };
        self.registry.insert(package.package_info.name, package_registry);
        self.save()?;
        Ok(())
    }

    pub fn remove_package(&mut self, package_name: &str) -> Result<(), anyhow::Error>{
        if !self.registry.contains_key(package_name){
            return Err(anyhow::anyhow!("Package does not exist"));
        }
        self.registry.remove(package_name);
        self.save()?;
        Ok(())
    }   

    pub fn save(&self) -> Result<(), anyhow::Error>{
        let registry_path = Self::registry_path();
        let registry_str = serde_json::to_string(&self).unwrap();
        fs::write(&registry_path, registry_str).unwrap();
        Ok(())
    }   

    pub fn get_package(&self, package_name: &str) -> Option<PackageRegistry>{
        if !self.registry.contains_key(package_name){
            return None;
        }
        self.registry.get(package_name).cloned()
    }

    pub fn read_package(&self, package_name: &str) -> Result<Package, anyhow::Error>{
        let package_register = self.get_package(package_name);
        if package_register.is_none(){
            return Err(anyhow::anyhow!("Package does not exist: {}", package_name));
        }
        let package_path = package_register.unwrap().path;
        if !package_path.exists(){
            return Err(anyhow::anyhow!("Package path does not exist: {}", package_path.display()));
        }
        let package = Package::read(&package_path);
        match package{  
            Ok(package) => {
                dbg!(&package);
                Ok(package)
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to read package: {}", e));
            }
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegistryLock{
    pub read: bool,
    pub write: bool,
}

impl Default for RegistryLock{
    fn default() -> Self{
        RegistryLock{read: false, write: false}
    }
}


impl RegistryLock{
    pub fn new() -> Self{
        RegistryLock{read: false, write: false}
    }

    pub fn init() -> Result<Self, anyhow::Error>{
        let registry_lock_path = Self::lock_file_path();
        if !registry_lock_path.exists(){
            let registry_lock = Self::new();
            let registry_lock_str = serde_json::to_string(&registry_lock).unwrap();
            fs::write(&registry_lock_path, registry_lock_str).unwrap();
            return Ok(registry_lock);
        }
        let registry_lock_str = fs::read_to_string(&registry_lock_path).unwrap();
        let registry_lock: RegistryLock = serde_json::from_str(&registry_lock_str).unwrap();
        Ok(registry_lock)
    }   


    pub fn is_read_locked(&self) -> bool{
        self.read
    }

    pub fn is_write_locked(&self) -> bool{
        self.write
    }

    pub fn lock_read(&mut self) -> Result<(), anyhow::Error>{
        self.read = true;
        self.save()?;
        Ok(())
    }

    pub fn lock_write(&mut self) -> Result<(), anyhow::Error>{
        self.write = true;
        self.read = true;
        self.save()?;
        Ok(())
    }

    pub fn unlock_read(&mut self) -> Result<(), anyhow::Error>{
        self.read = false;
        self.save()?;
        Ok(())
    }

    pub fn unlock_write(&mut self) -> Result<(), anyhow::Error>{
        self.write = false;
        self.read = false;
        self.save()?;
        Ok(())
    }

    pub fn lock_file_path() -> PathBuf{
        let app_dir = VatConfig::get_app_dir().unwrap();
        let registry_lock_path = app_dir.join("registry.lock");
        registry_lock_path
    }

    pub fn save(&self) -> Result<(), anyhow::Error>{
        let registry_lock_path = Self::lock_file_path();
        let registry_lock_str = serde_json::to_string(&self).unwrap();
        fs::write(&registry_lock_path, registry_lock_str).unwrap();
        Ok(())
    }
}

// impl Drop for RegistryLock{
//     fn drop(&mut self){
//         self.read = true;
//         self.write = true;
//         let registry_lock_path = Self::lock_file_path();
//         let registry_lock_str = serde_json::to_string(&self).unwrap();
//         fs::write(&registry_lock_path, registry_lock_str).unwrap();
//     }
// }
