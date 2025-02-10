use clap::{Parser, Subcommand};
use colored::*;
use std::process::Command;
use vat::package::Package;
use git2::Repository as GitRepository;
use std::io::{self, Write}; 
use vat::vat_repository::VatRepo;

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
    #[command(name = "init", about = "Create a new Vat package in an existing directory")]
    Init,
    #[command(name = "new", about = "Create a new Vat package")]
    New{
        name: String,
    },
    #[command(name = "cat", about = "Read a Vat package")]
    Cat,
    #[command(name = "up", about = "Increment the version of a Vat package, commit and create a new git tag")]
    Up{
            #[arg(short = 'M', long, help = "Increment the major version")]
            major:bool,

            #[arg(short = 'm', long, help = "Increment the minor version")]
            minor:bool,
            #[arg(short = 'p', long, help = "Increment the patch version")]
            patch:bool,
        },

    #[command(name = "publish", about = "Publish a Vat package")]
    Publish{
        #[arg(short = 'm', long)]
        message: String,
        // #[arg(short, long)]
        // remote: bool,
    },
    Link,
    Run{
        #[arg(required = false)]
        subcommand:Option<String>,
        #[arg(long="append", short='a', num_args = 1..)]
        append: Option<Vec<String>>,
        #[arg(long="package", short='p')]
        package: Option<String>,
    },
    Test




}

fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Test) => {
            let current_dir = std::env::current_dir()?;
            let package = Package::read(&current_dir)?;

            dbg!(&package);
          
            Ok(())
        }


        // Testing vat Link package to repository
        Some(Commands::Link) => {

            let current_dir = std::env::current_dir()?;
            let read_package = Package::read(&current_dir)?;

            let repository = VatRepo::init();

            match repository{
                Ok(mut repository) => {
                    repository.link_package(&read_package, &current_dir)?;
                    println!("Package linked successfully to repository");
                }
                Err(e) => {
                    eprintln!("Error initializing repository: {}", e);
                }
            }
            Ok(())
        }

        Some(Commands::Init) => {
            let current_dir = std::env::current_dir()?;

            if let Err(e) = Package::init(current_dir, None) {

                let error = format!("{}", e);
                return Err(anyhow::anyhow!(error));
            }else{
                return Ok(());
            }


        }
        Some(Commands::New { name }) => {
            let current_dir = std::env::current_dir()?;
            if let Err(e) = Package::init(current_dir.clone(), Some(name.clone())) {
                let error = format!("{}", e);
                return Err(anyhow::anyhow!(error));

            }else{
                // create folder
                let package_dir = current_dir.join(name);
                std::fs::create_dir_all(package_dir).unwrap();  
                println!("{}", format!("Vat package initialized").green());
                return Ok(());

            }
        }
        Some(Commands::Cat) => {
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
   

        Some(Commands::Publish { message }) => {
            let current_dir = std::env::current_dir()?;
            let read_package = Package::read(&current_dir)?;


            let repository = VatRepo::init();
            match repository{
                Ok(mut repository) => {
                    repository.publish_package(&read_package, &current_dir, &message)?;
                    println!("Package published successfully to repository");
                }
                Err(e) => {
                    eprintln!("Error initializing repository: {}", e);
                }

            }
            Ok(())
        },

        Some(Commands::Up { major, minor, patch }) => {
            let current_dir = std::env::current_dir().unwrap();

            if Package::is_vat_package(&current_dir) {
                let mut package = Package::read(&current_dir).unwrap();
                println!("Make sure to commit before running this command");
                println!("Current version: {:?}", package.get_current_version());


                if major {
                    package.increment_version(true, false, false); 
                } else if minor {
                    package.increment_version(false, true, false);
                } else if patch {
                    package.increment_version(false, false, true);
                }else{
                    package.increment_version(false, true, false);
                }


                println!("{}", format!("New version: {}", package.get_current_version()).green());
                println!("Are you sure you want to increment the version? (y/n)");
                io::stdout().flush().unwrap();
                
                let mut input = String::new();
                io::stdin().read_line(&mut input).expect("Failed to read input");

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
                    if let Some(_latest_tag) = tags.last() {
                        // println!("Latest tag: {:?}", latest_tag.unwrap());
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

                let tag_version = package.get_current_version().to_string();

                // Commit the changes
                let commit_message = format!("Commit for version: {}", tag_version);
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
                return Err(anyhow::anyhow!("vat.toml not found in the current directory"));
            }
        },

        Some(Commands::Run { subcommand, append, package }) => {
            let current_dir = std::env::current_dir().unwrap();

            // let vat_repo = VatRepo::init()?;
            // let package = Package::resolve_package(package, true);


            Package::run(subcommand.unwrap().as_str(), package, append);



            Ok(())
        }
        None => {
            println!("No command provided");
            return Ok(());
        }
    }

}