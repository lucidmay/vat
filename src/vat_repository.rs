use std::path::PathBuf;
use std::fs;
use crate::config::VatConfig;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use anyhow::{anyhow, Result};
use crate::package::{self, Package};
use crate::package::{PackageResolver, PackageFrom};
use colored::Colorize;
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RepoPackage{
    pub versions: HashMap<semver::Version, RepoPackageInfo>,
    pub main_branch_path: PathBuf, 
    pub git_url: Option<String>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RepoPackageInfo{
    pub published_on: DateTime<Utc>,
    pub version_comment: Option<String>
}


impl RepoPackage{
    pub fn new(package_path: &PathBuf, package_repository_path: &str) -> Self{
        Self{
            versions: HashMap::new(),
            main_branch_path: package_path.clone(),
            git_url: Some(package_repository_path.to_string()),
        }
    }

    pub fn version_exists(&self, version: &semver::Version) -> bool{
        self.versions.contains_key(version)
    }


    pub fn add_version(&mut self, version: &semver::Version, version_comment: &str) -> Result<(), anyhow::Error>{
        if self.version_exists(&version){
            return Err(anyhow!("Version already exists"));
        }
        self.versions.insert(version.clone(), RepoPackageInfo{
            published_on: Utc::now(),
            version_comment: Some(version_comment.to_string()),
        });
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct VatRepo{
    pub packages: HashMap<String, RepoPackage>
}


impl VatRepo{
    pub fn new() -> Self{
        Self{packages: HashMap::new()}
    }


    pub fn init() -> Result<Self, anyhow::Error>{

        let vat_config = VatConfig::init()?;
        let repository_path = vat_config.get_repository_path();

        let repository_path = match repository_path{
            Some(path) => path,
            None => return Err(anyhow!("Repository path not found")),
        };

        if !repository_path.exists(){
            fs::create_dir_all(&repository_path)?;
        }

        let repository_config_path = repository_path.join("vat.repository.toml");
        let repository = if !repository_config_path.exists(){
            let repository = VatRepo::new();

            let repository_config_str = toml::to_string(&repository)?;
            fs::write(repository_config_path, repository_config_str)?;
            repository  
        }else{
            let repository_config_str = fs::read_to_string(repository_config_path)?;
            let repository: VatRepo = toml::from_str(&repository_config_str)?;
            repository
        }; 


        Ok(repository)

    }



    pub fn get_repo_package(&self, package_name: &str) -> Option<&RepoPackage>{
        self.packages.get(package_name)
    }



    /// Link package to repoistory does not add any versions from the package

    /// Just the package is registered in the repository
    pub fn link_package(&mut self, package: &Package, package_path: &PathBuf) -> Result<(), anyhow::Error>{
        let package_name = package.get_name();
        let package_repository_path = package.package_info.repository.clone();

        dbg!(&package_path);
        if !package_path.exists(){
            return Err(anyhow!("Package path not found"));
        }

        if self.package_exists(&package_path)?{
            return Err(anyhow!("Package already exists"));
        }


        let repo_package = RepoPackage{
            versions: HashMap::new(),
            main_branch_path: package_path.clone(),
            git_url: package_repository_path,
        };


        self.packages.insert(package_name.to_string(), repo_package);
        self.save()?;

        // create a package directory in the repository
        let vat_config = VatConfig::init()?;
        let repository_path = vat_config.get_repository_path();
        if let Some(repository_path) = repository_path{
            let repository_package_path = repository_path.join(package_name);
            if !repository_package_path.exists(){
                fs::create_dir_all(&repository_package_path)?;
            }
        }

        Ok(())

    }


    /// Publish package to repoistory
    pub fn publish_package(&mut self, package: &Package, package_path: &PathBuf, version_comment: &str) -> Result<(), anyhow::Error>{

        let package_name = package.get_name();
        let package_repository_path = package.package_info.repository.clone();

        if !package_path.exists(){
            return Err(anyhow!("Package path not found"));
        }

        // return an error if a different package exists with the same name
        let _package_exists_with_same_name = self.package_exists(&package_path)?;

        let mut repo_package = if let Some(repo_package) = self.get_repo_package(package_name){
            repo_package.clone()
        }else{
            RepoPackage{
                versions: HashMap::new(),
                main_branch_path: package_path.clone(),
                git_url: package_repository_path,
            }
        };


        // check if the current version is already published
        let current_version = package.get_version();
        if repo_package.version_exists(&current_version){
            let messsage = format!("{}: Version {} already published", package_name, current_version);
            return Err(anyhow!(messsage));
        }


        // add the version to the repository
        repo_package.add_version(current_version, version_comment)?;

        // resolves path to copy the package to the repository
        let repository_base_path = VatConfig::init()?.get_repository_path().unwrap();
        let repo_package_path = repository_base_path.join(package_name);
        let repo_package_version_path = repo_package_path.join(current_version.to_string());

        let zip_file_name = format!("{}.zip", current_version);
        let source_zip_file_path = package_path.join(&zip_file_name);

        let message = format!("Creating zip from the tag: {}", current_version);
        println!("{}", message.bright_black());

        // create zip from git version
        // "git archive --format=zip -o archive.zip 0.0.3"
        let _command = std::process::Command::new("git")
            .arg("archive")
            .arg("--format=zip")
            .arg("-o")
            .arg(&zip_file_name)
            .arg(current_version.to_string())
            .current_dir(&package_path)
            .status()
            .expect("Failed to create zip file");


        // copy zip file to repository
        let mut copy_options = fs_extra::dir::CopyOptions::new();
        copy_options.overwrite = true;
        copy_options.copy_inside = true;


        if !repo_package_version_path.exists(){
            fs::create_dir_all(&repo_package_version_path)?;
        }

        let message = format!("Copying zip file to repository");
        println!("{}", message.bright_black());

        let file = std::fs::File::open(&source_zip_file_path)?;
        let mut archive = zip::read::ZipArchive::new(file)?;

        archive.extract(repo_package_version_path)?;
        let message = format!("Version {} successfully extracted to repository", current_version);
        println!("{}", message.bright_black());

        fs::remove_file(&source_zip_file_path)?;

        // dbg!(&repo_package);
        self.packages.insert(package_name.to_string(), repo_package);
        self.save()?;

        let messsage = format!("{}: Version {} published", package_name, current_version);
        println!("{}", messsage.cyan());

        Ok(())
    }



    pub fn package_exists(&self, package_path: &PathBuf) -> Result<bool, anyhow::Error>{
        let package_name = package_path.file_name().unwrap().to_str().unwrap();
        if self.packages.contains_key(package_name){
            if self.get_repo_package(package_name).unwrap().main_branch_path == package_path.clone(){
                Ok(true)
            }else{
                return Err(anyhow!("Different package exists with the same name"));
            }
        }else{
            Ok(false)
        }
    }



    pub fn save(&self) -> Result<(), anyhow::Error>{
        let vat_config = VatConfig::init()?;
        let repository_path = vat_config.get_repository_path();

        let repository_path = match repository_path{
            Some(path) => path,
            None => return Err(anyhow!("Repository path not found")),
        };

        let repository_config_path = repository_path.join("vat.repository.toml");
        let repository_config_str = toml::to_string(self)?;
        fs::write(repository_config_path, repository_config_str)?;
        Ok(())  
    }

    pub fn get_package(&self,package_resolver: &PackageResolver) -> Result<PackageResolver, anyhow::Error> {
        let package_name = package_resolver.package_name.clone();
        if !self.packages.contains_key(&package_name){

            return Err(anyhow::anyhow!("Package {} not found", package_name));

        }

        let repo_path = VatConfig::init()?.get_repository_path().unwrap();

        let mut out_package_resolver = package_resolver.clone();

        let package_path = match &package_resolver.from{
            PackageFrom::Latest => {
                let latest_version = self.get_latest_version(&package_name);

                if latest_version.is_some(){

                    let package_path = repo_path.join(package_name).join(latest_version.unwrap().to_string());
                    package_path


                }else{
                    // fall back to main branch
                    let package_path = self.get_repo_package(&package_name).unwrap().main_branch_path.clone();
                    package_path
                }
            }

            PackageFrom::Version(version) => {
                // package_path = repo_path.join(package_name).join(version.to_string());
                if self.version_exists(&package_name, version){
                    let package_path = repo_path.join(package_name).join(version.to_string());
                    package_path
                }else{
                    // fall back to latest version

                    let latest_version = self.get_latest_version(&package_name);
                    if latest_version.is_some(){
                        let package_path = repo_path.join(package_name).join(latest_version.unwrap().to_string());
                        package_path
                    }else{
                        return Err(anyhow::anyhow!("Version not found"));
                    }

                }
            }

            PackageFrom::Main => {
                let package_path = self.get_repo_package(&package_name).unwrap().main_branch_path.clone();
                package_path
            }


        };

        // dbg!(&package_path);
        out_package_resolver.package_path = Some(package_path.clone());
        let package = Package::read(&package_path)?;
        out_package_resolver.package = Some(package);
        Ok(out_package_resolver)
    }




    pub fn get_latest_version(&self, package_name: &str) -> Option<semver::Version>{
        let repo_package = self.get_repo_package(package_name)?;
        let latest_version = repo_package.versions.keys().max();
        latest_version.cloned()
    }

    pub fn version_exists(&self, package_name: &str, version: &semver::Version) -> bool{
        let repo_package = self.get_repo_package(package_name);
        if repo_package.is_some(){
            repo_package.unwrap().version_exists(version)
        }else{
            false
        }
    }


    pub fn pretty_list(&self) {
        if !self.packages.is_empty() {
            for (package_name, package_versions) in &self.packages {
                let message = format!("Package: {}", package_name);
                println!("{}", message.green());
                let mut sorted_versions = package_versions.versions.iter()
                    .collect::<Vec<(&semver::Version, &RepoPackageInfo)>>();    

                // sort the versions in reverse order
                sorted_versions.sort_by(|a, b| b.0.cmp(a.0));
                // sorted_versions.reverse();

                for (version, package_info) in sorted_versions {
                    println!("   {} - {} - {}", version, package_info.version_comment.clone().unwrap_or_default().bright_black(),  package_info.published_on.format("%Y-%m-%d %H:%M:%S").to_string().bright_black());
                }
            }
        } else {
            println!("{}", format!("No packages found in the repository"));
        }
    }

}



