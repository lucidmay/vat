use git2::Repository as GitRepository;
use semver;
use std::{io::Write, process::Command};
use std::path::PathBuf;

pub trait Git {
    fn get_tags(&self) -> Result<Vec<String>, anyhow::Error>;
    fn get_remotes(&self) -> Result<Vec<String>, anyhow::Error>;
    fn create_main_branch(&self) -> Result<String, anyhow::Error>;
    fn git_ignore(&self, path: &PathBuf) -> Result<(), anyhow::Error>;  
}

impl Git for GitRepository {
    fn get_tags(&self) -> Result<Vec<String>, anyhow::Error> {
        let tags = self.tag_names(None)?;
        let tags = tags.iter().collect::<Vec<_>>();
        let tags = tags.iter().map(|tag| tag.unwrap().to_string()).collect::<Vec<_>>();
        Ok(tags)
    }

    fn get_remotes(&self) -> Result<Vec<String>, anyhow::Error> {
        let remote = self.remotes().unwrap();
        let remote = remote.iter().collect::<Vec<_>>();
        let remote = remote.iter().map(|remote| remote.unwrap().to_string()).collect::<Vec<_>>();
        Ok(remote)
    }

    fn create_main_branch(&self) -> Result<String, anyhow::Error> {
        // git checkout -b main
        let status = Command::new("git")
            .arg("checkout")
            .arg("-b")
            .arg("main")
            .current_dir(&self.path())
            .status()
            .expect("Failed to execute git checkout");
        Ok(status.to_string())
    }

    fn git_ignore(&self, path: &PathBuf) -> Result<(), anyhow::Error> {
        // create .gitignore file
        let mut file = std::fs::File::create(path.join(".gitignore"))?;
        let ignore_raw_stirng ="";
        file.write_all(ignore_raw_stirng.as_bytes())?;
        Ok(())
    }
}

pub struct GitTags{
    pub tags: Vec<semver::Version>,
}

impl GitTags{
    pub fn new(tags: Vec<String>) -> Self{
        let tags = tags.iter().map(|tag| semver::Version::parse(tag).unwrap()).collect::<Vec<_>>();
        Self{tags}
    }

    pub fn get_latest(&self) -> Option<semver::Version> {
        self.tags.iter().max().cloned()
    }
}
