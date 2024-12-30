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
    Read,
    Publish{
        #[arg(short = 'm', long)]
        message: Option<String>,
        
        #[arg(short, long)]
        remote: bool,
    },
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

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init) => {
            let current_dir = std::env::current_dir().unwrap();
            if let Err(e) = Package::init(current_dir) {
                eprintln!("{}", format!("Error initializing Vat: {}", e).red()); 
            }
        }
        Some(Commands::Read) => {
            let current_dir = std::env::current_dir().unwrap();
            let vat = Package::read(&current_dir);
            match vat {
                Ok(package) => {
                    println!("Package: {:?}", package);
                }
                Err(e) => {
                    eprintln!("{}", format!("Error reading Vat: {}", e).red());
                }
            }
        },
   

        Some(Commands::Publish { message, remote }) => {
            let current_dir = std::env::current_dir().unwrap();
            let repo = GitRepository::open(&current_dir).unwrap();
            let tags = repo.get_tags().unwrap();
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
                            return;
                        }else{
                            eprintln!("{}", format!("Git push failed: {}", cmd.status).red());
                        }

                    }
                }
            }

            if remote {
                println!("{}", format!("Publishing to remote").green());
                let cmd = Command::new("git").args(&["push", "--tags"]).current_dir(&current_dir).output().unwrap();
                if cmd.status.success() {
                    println!("{}", format!("Git push successful").green());
                    return;
                }else{
                    eprintln!("{}", format!("Git push failed: {}", cmd.status).red());
                }
            }else{

                let package = Package::read(&current_dir);
                match package{
                    Ok(package) => {

                    }
                    Err(e) => {
                        eprintln!("{}", format!("Vat package error: {}: {}", e, current_dir.display()).red());
                        return;
                    }
                }

            }



            if !remote {
                let tags = repo.get_tags().unwrap();
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
                let remote = repo.get_remotes().unwrap();
                if !remote.is_empty() {
                    println!("Remote git repository: {:?}", remote.first().unwrap());
                } else {
                    println!("No remote git repository set");
                }

                let package = Package::read(&current_dir).unwrap();

                let mut vat_repository = VatRepository2::read_repository().unwrap();
                let result = vat_repository.add_package(package.clone(), message.unwrap_or("".to_string()), current_dir.clone());
                match result {
                    Ok(_) => {
                        let result = vat_repository.package_data_update(&package, current_dir.clone());
                        match result {
                            Ok(_) => {
                                let result = vat_repository.save();
                                match result {
                                    Ok(_) => {
                                        println!("Package added successfully");
                                    }
                                    Err(e) => {
                                        eprintln!("Error saving repository: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Error updating package data: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error adding package: {}", e);
                    }
                }
        }
            // dbg!(&vat_repository);
            
        },
        Some(Commands::Repo) => {
            let config = VatConfig::init().unwrap();
            println!("Repository path: {:?}", config.get_repository_path());
        },
        Some(Commands::RepoInit) => {
            let repository = VatRepository2::init();
            match repository {
                Ok(_) => {
                    println!("Repository initialized");
                }
                Err(e) => {
                    eprintln!("Error initializing repository: {}", e);
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
                    return;
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
                    return;
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
                    return;
                }


                let mut revwalk = repo.revwalk().unwrap();
                revwalk.push_head().unwrap();
                let target_commit_oid = revwalk.next().unwrap().unwrap();
                let target_commit = repo.find_object(target_commit_oid, None).unwrap();

                repo.tag(&package.get_current_version().to_string(), &target_commit, &repo.signature().unwrap(), tag_message.trim(), true).unwrap();
                println!("Tag created successfully");

            } else {
                println!("Vat package not found");
            }
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
                    }
                    Err(e) => {
                        eprintln!("Error executing command: {}", e);
                    }
                }
                // package.run_command(&subcommand, &current_dir).unwrap();
            } else {
                println!("No subcommand provided");
            }

            // std::process::Command::new("explorer").status().unwrap();
        },
        None => {
            println!("Vat");
            println!("      Have you watched the Vat of Acid episode? https://en.wikipedia.org/wiki/The_Vat_of_Acid_Episode");
        },
    }
}