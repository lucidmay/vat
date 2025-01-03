use git2::Repository as GitRepository;
use semver;

pub trait Git {
    fn get_tags(&self) -> Result<Vec<String>, anyhow::Error>;
    fn get_remotes(&self) -> Result<Vec<String>, anyhow::Error>;
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
