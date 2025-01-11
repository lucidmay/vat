use clap::{Parser, Subcommand, Args};
use colored::*;
use std::process::Command;
use vat::config::VatConfig;
use vat::repository::VatRepository2;
use vat::package::Package;
use git2::Repository as GitRepository;
use std::io::{self, Write}; 
use vat::git::Git;
use vat::git::GitTags;
use vat::registry::{Registry, RegistryLock};

/// Simple program to demonstrate colored CLI
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to greet
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    New{
        name: String,
    },
    LocateProject,
    Publish{
        #[arg(short = 'm', long)]
        message: Option<String>,
        
        #[arg(short, long)]
        remote: bool,
    },
    Register,
    Repo,
    RepoInit,
    Up{
        #[arg(short = 'M', long)]
        major:bool,
        #[arg(short = 'm', long)]
        minor:bool,
        #[arg(short = 'p', long)]
        patch:bool,
    },
    Test{
        #[arg(required = false)]
        subcommand:Option<String>,
        #[arg(long="append", short='a', num_args = 1..)]
        append: Option<Vec<String>>
    },

}

fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init) => {
            let current_dir = std::env::current_dir()?;
            if let Err(e) = Package::init(current_dir, None) {
                eprintln!("{}", format!("Error initializing Vat: {}", e).red()); 
                return Err(anyhow::anyhow!("Error initializing Vat"));
            }else{
                return Ok(());
            }
        }
        Some(Commands::New { name }) => {
            let current_dir = std::env::current_dir()?;
            if let Err(e) = Package::init(current_dir.clone(), Some(name.clone())) {
                eprintln!("{}", format!("Error initializing Vat: {}", e).red()); 
                return Err(anyhow::anyhow!("Error initializing Vat"));
            }else{
                // create folder
                let package_dir = current_dir.join(name);
                std::fs::create_dir_all(package_dir).unwrap();  
                println!("{}", format!("Vat package initialized").green());
                return Ok(());
            }
        }
        Some(Commands::LocateProject) => {
            let current_dir = std::env::current_dir()?;
            let vat = Package::read(&current_dir);
            match vat {
                Ok(package) => {
                    println!("Package: {:?}", package);
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("{}", format!("Error reading Vat: {}", e).red());
                    return Err(anyhow::anyhow!("Error reading Vat"));
                }
            }
        },
   

        Some(Commands::Publish { message, remote }) => {
            let current_dir = std::env::current_dir()?;
            let repo = GitRepository::open(&current_dir)?;
            let tags = repo.get_tags()?;
            let git_tags = GitTags::new(tags);
            if !git_tags.tags.is_empty() {
                if let Some(latest_tag) = git_tags.get_latest() {
                    if remote{
                        println!("{}", format!("Publishing to remote").green());
                        // git push 
                        let cmd = Command::new("git").args(&["push"]).current_dir(&current_dir).output().unwrap();
                        //git push origin v1.0.0
                        let cmd = Command::new("git").args(&["push", "origin", latest_tag.to_string().as_str()]).current_dir(&current_dir).output().unwrap();
                        if cmd.status.success() {
                            println!("{}", format!("Git push successful").green());
                            return Ok(());
                        }else{
                            eprintln!("{}", format!("Git push failed: {}", cmd.status).red());
                            return Err(anyhow::anyhow!("Git push failed"));
                        }

                    }
                }
            }

            if remote {
                println!("{}", format!("Publishing to remote").green());
                let cmd = Command::new("git").args(&["push", "--tags"]).current_dir(&current_dir).output().unwrap();
                if cmd.status.success() {
                    println!("{}", format!("Git push successful").green());
                    return Ok(());
                }else{
                    eprintln!("{}", format!("Git push failed: {}", cmd.status).red());
                    return Err(anyhow::anyhow!("Git push failed"));
                }
            }else{

                let package = Package::read(&current_dir);
                match package{
                    Ok(package) => {

                    }
                    Err(e) => {
                        eprintln!("{}", format!("Vat package error: {}: {}", e, current_dir.display()).red());
                        return Err(anyhow::anyhow!("Vat package error"));
                    }
                }

            }



            if !remote {
                let tags = repo.get_tags()?;
                if !tags.is_empty() {
                    if let Some(latest_tag) = tags.last() {
                        println!("Latest tag: {:?}", latest_tag);
                    } else {
                        println!("No tags found");
                    }
                } else {
                    println!("No tags found");
                }


                // check if remote repository is set
                let remote = repo.get_remotes()?;
                if !remote.is_empty() {
                    println!("Remote git repository: {:?}", remote.first().unwrap());
                } else {
                    println!("No remote git repository set");
                }

                let package = Package::read(&current_dir)?;

                let mut vat_repository = VatRepository2::read_repository()?;
                let result = vat_repository.add_package(package.clone(), message.clone(), current_dir.clone());
                match result {
                    Ok(_) => {
                        let result = vat_repository.package_data_update(&package, current_dir.clone());
                        match result {
                            Ok(_) => {
                                let result = vat_repository.save();
                                match result {
                                    Ok(_) => {
                                        println!("Package added successfully");
                                        return Ok(());
                                    }
                                    Err(e) => {
                                        eprintln!("Error saving repository: {}", e);
                                        return Err(anyhow::anyhow!("Error saving repository"));
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Error updating package data: {}", e);
                                return Err(anyhow::anyhow!("Error updating package data"));
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error adding package: {}", e);
                        return Err(anyhow::anyhow!("Error adding package"));
                    }
                }
            } else {
                return Err(anyhow::anyhow!("Remote repository not set"));
            }   
            // dbg!(&vat_repository);
            
        },
        Some(Commands::Register) => {
            let registry_lock = RegistryLock::init();
            match registry_lock{
                Ok(mut registry_lock) => {
                    if registry_lock.is_read_locked(){
                        return Err(anyhow::anyhow!("Registry is locked by another process"));
                    }else{
                        registry_lock.lock_write()?;
                        let registry = Registry::init();
                        match registry{ 
                            Ok(mut registry) => {
                                let current_dir = std::env::current_dir()?;
                                let package = Package::read(&current_dir)?;
                                let result = registry.add_package(package, current_dir.clone());
                                match result{
                                    Ok(_) => {
                                        println!("Package added successfully");
                                        registry_lock.unlock_write()?;
                                        return Ok(());
                                    }
                                    Err(e) => {
                                        eprintln!("Error registering package: {}", e);
                                        registry_lock.unlock_write()?;
                                        return Err(anyhow::anyhow!("Error registering package"));
                                    }
                                }

                                registry_lock.unlock_write()?;
                                return Ok(());
                            }
                            Err(e) => {
                                return Err(anyhow::anyhow!("Error initializing registry: {}", e));
                            }
                        }
                    }
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Error initializing registry lock: {}", e));
                }
            }
        },
        Some(Commands::Repo) => {
            let config = VatConfig::init().unwrap();
            println!("Repository path: {:?}", config.get_repository_path());
            return Ok(());
        },
        Some(Commands::RepoInit) => {
            let repository = VatRepository2::init();
            match repository {
                Ok(_) => {
                    println!("Repository initialized");
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("Error initializing repository: {}", e);
                    return Err(anyhow::anyhow!("Error initializing repository"));
                }
            }
        },
        Some(Commands::Up { major, minor, patch }) => {
            let current_dir = std::env::current_dir().unwrap();
            if Package::is_vat_package(&current_dir) {
                let mut package = Package::read(&current_dir).unwrap();
                println!("Make sure you have committed before running this command");
                println!("Current version: {:?}", package.get_current_version());


                if major {
                    package.increment_version(true, false, false); 
                } else if minor {
                    package.increment_version(false, true, false);
                } else {
                    package.increment_version(false, false, true);
                }

                println!("{}", format!("New version: {}", package.get_current_version()).green());

                println!("Are you sure you want to continue? (y/n)");
                io::stdout().flush().unwrap();
                
                let mut input = String::new();
                io::stdin().read_line(&mut input).expect("Failed to read line");

                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Version increment canceled.");
                    return Ok(());
                }

                println!("Tag message:");
                io::stdout().flush().unwrap();
                
                let mut tag_message = String::new();
                io::stdin().read_line(&mut tag_message).expect("Failed to read line");
                package.set_version_message(tag_message.trim().to_string());

                let repo = GitRepository::open(&current_dir).unwrap();
                let tags = &repo.tag_names(None).unwrap();
                let tags = tags.iter().collect::<Vec<_>>();
                if !tags.is_empty() {
                    if let Some(latest_tag) = tags.last() {
                        println!("Latest tag: {:?}", latest_tag.unwrap());
                    } else {
                        println!("No git tags found to publish");
                    }
                } else {
                    println!("No git tags found to publish");
                }

                package.save(&current_dir).unwrap();

                // Open the repository
                let repo = GitRepository::open(&current_dir).unwrap();

                // Stage all changes in the current directory
                let status = Command::new("git")
                    .arg("add")
                    .arg(".")
                    .current_dir(&current_dir)
                    .status()
                    .expect("Failed to execute git add");

                // Check if the command was successful
                if !status.success() {
                    eprintln!("Failed to stage changes.");
                    return Err(anyhow::anyhow!("Failed to stage changes"));
                }

                // Commit the changes
                let commit_message = "New version";
                let status = Command::new("git")
                    .arg("commit")
                    .arg("-m")
                    .arg(commit_message)
                    .current_dir(&current_dir)
                    .status()
                    .expect("Failed to execute git commit");

                // Check if the command was successful
                if !status.success() {
                    eprintln!("Failed to commit changes.");
                    return Err(anyhow::anyhow!("Failed to commit changes"));
                }


                let mut revwalk = repo.revwalk().unwrap();
                revwalk.push_head().unwrap();
                let target_commit_oid = revwalk.next().unwrap().unwrap();
                let target_commit = repo.find_object(target_commit_oid, None).unwrap();

                repo.tag(&package.get_current_version().to_string(), &target_commit, &repo.signature().unwrap(), tag_message.trim(), true).unwrap();
                println!("Tag created successfully");
                return Ok(());


            } else {
                println!("Vat package not found");
            }
            return Err(anyhow::anyhow!("Vat package not found"));
        },
        Some(Commands::Test { subcommand, append }) => {
            let current_dir = std::env::current_dir().unwrap();
            let package = Package::read(&current_dir).unwrap();
            if subcommand.is_some() {

                if append.is_some() {
                    let vat_repository = VatRepository2::read_repository().unwrap();
                    for name in append.unwrap().iter() {
                        let package = vat_repository.get_latest_package(&name);
                        let latest_package_path = vat_repository.get_latest_package_path(&name).unwrap();
                        if package.is_some() {
                            let package = package.unwrap();
                            println!("{}", format!("Appending package: {}: {}", name, &package.get_current_version()).green());
                            package.load_all_environments(&latest_package_path).unwrap();
                        }else{
                            eprintln!("{}", format!("Package failed to append: {}", name).red());
                        }
                    }
                }

                let subcommand = subcommand.unwrap();
                package.command_load_env(&subcommand, &current_dir).unwrap();
                let result = package.run_only_command(&subcommand);
                match result {
                    Ok(_) => {
                        println!("Command executed successfully");
                        return Ok(());
                    }
                    Err(e) => {
                        eprintln!("Error executing command: {}", e);
                        return Err(anyhow::anyhow!("Error executing command"));
                    }
                }
                // package.run_command(&subcommand, &current_dir).unwrap();
            } else {
                println!("No subcommand provided");
                return Err(anyhow::anyhow!("No subcommand provided"));
            }

            // std::process::Command::new("explorer").status().unwrap();
        },
        None => {
            println!("Vat");
            println!("      Have you watched the Vat of Acid episode? https://en.wikipedia.org/wiki/The_Vat_of_Acid_Episode");
            return Err(anyhow::anyhow!("No command provided"));
        },
    }
}