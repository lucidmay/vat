use clap::{Parser, Subcommand, Args};
use colored::*;
use std::process::Command;
use vat::config::VatConfig;
use vat::repository::VatRepository;
use vat::package::Package;
use git2::Repository as GitRepository;
use std::io::{self, Write}; 
use std::path::PathBuf;
use fs_extra::dir::{copy, CopyOptions};
use vat::git::Git;

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
    Houdini,
    Publish,
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
        Some(Commands::Houdini) => {
            // append path env variable
            // let houdini_path = "C:\\Program Files\\Side Effects Software\\Houdini 20.5.314\\bin";
            // let current_path = std::env::var("PATH").unwrap_or_default();
            // let new_path = format!("{};{}", current_path, houdini_path);
            // std::env::set_var("PATH", new_path);

            //  let output = Command::new("houdini")
            //     .status()
            //     .expect("Failed to execute Houdini command");

            // if output.success() {
            //     println!("Houdini launched successfully.");
            // } else {
            //     eprintln!("Failed to launch Houdini.");
            // }

            // },
            // let current_dir = std::env::current_dir().unwrap();
            // let package = Package::read(&current_dir).unwrap();
            // // package.load_environments("houdinibin").unwrap();
            // package.run_command("houdini20").unwrap();
            println!("houdini");

            // let output = Command::new("cmd")
            //     .arg("/C")
            //     .arg("where houdini")
            //     .output()
            //     .expect("Failed to check Houdini command availability");

            // println!("Houdini command output: {:?}", String::from_utf8_lossy(&output.stdout));

            // std::process::Command::new("houdini").status().unwrap();

        }

        Some(Commands::Publish) => {
            let current_dir = std::env::current_dir().unwrap();
            let repo = GitRepository::open(&current_dir).unwrap();
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

            // raw git checkout
            let status = Command::new("git")
                .arg("checkout")
                .arg(package.get_current_version())
                .current_dir(&current_dir)
                .status()
                .expect("Failed to execute git checkout");

            if !status.success() {
                eprintln!("Failed to checkout version");
                return;
            }

            let mut repository = VatRepository::read_repository().unwrap();
            let result = repository.add_package(package.clone());
            
            match result {
                Ok(_) => {
                    // repository
                    dbg!(&repository);
                    let result =  VatRepository::package_data_update(&package);
                    match result {
                        Ok(_) => {
                            println!("Package data updated successfully");
                        }
                        Err(e) => {
                            eprintln!("Error updating package data: {}", e);
                        }
                    }

                },
                Err(e) => eprintln!("Error adding package: {}", e),
            }

            // Checkout the master branch in the destination directory
            let status = Command::new("git")
                .arg("checkout")
                .arg("master")
                .current_dir(current_dir) // Ensure you're checking out in the correct directory
                .status()
                .expect("Failed to execute git checkout");

            if !status.success() {
                eprintln!("Failed to checkout master. Exit code: {}", status.code().unwrap_or(-1));
                return;
            }

            println!("Directory copied successfully");
            
        },
        Some(Commands::Repo) => {
            let config = VatConfig::init().unwrap();
            println!("Repository path: {:?}", config.get_repository_path());
        },
        Some(Commands::RepoInit) => {
            let repository = VatRepository::init();
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
                // let signature = repo.signature().unwrap();
                // println!("Tagger Name: {}", signature.name().unwrap());
                // println!("Tagger Email: {}", signature.email().unwrap());
                // println!("Tagger Time: {}", signature.when().seconds());

                repo.tag(&package.get_current_version(), &target_commit, &repo.signature().unwrap(), "New version", true).unwrap();
                println!("Tag created successfully");

            } else {
                println!("Vat package not found");
            }
        },
        Some(Commands::Test { subcommand }) => {
            let current_dir = std::env::current_dir().unwrap();
            let package = Package::read(&current_dir).unwrap();
            if subcommand.is_some() {
                package.run_command(&subcommand.unwrap()).unwrap();
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