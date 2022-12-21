use std::{process::{Command, Output, Stdio}, env::args, io::Write};
use tempfile;
use tempfile::tempdir;

use clap::{Parser, ValueEnum};


#[derive(Parser)]
#[command(author, version, about)]
struct Cli {

    /// Git url to clone - example: gitlab@steplix/frontend/kata-gilded-rose-sebas-dani.git
    git_url: String,
    /// Github repository name - example: kata-gilded-rose-sebas-dani
    /// 
    /// Beware that the slug is derived from the repository name, unless the github slug override parameter is passed
    #[arg(verbatim_doc_comment)]
    github_repo_name: String,
    
    /// Github slug override - example: KataGildedRose-SebasDani
    #[arg(short = 'o', long = "override-github-slug")]
    github_slug: Option<String>,

    /// Team name to assign once the repository is created - example: steplix/frontend
    /// 
    /// Supports adding multiple teams
    /// 
    /// Permissions may be {pull|triage|push|maintain|admin} with pull as default
    #[arg(short = 'p', long, value_name="ORGANIZATION/TEAM:PERMISSION", verbatim_doc_comment)]
    permission: Option<Vec<String>>,

    /// Description for the GitHub repository
    #[arg(short, long)]
    description: Option<String>,

    /// Topics to add to the repository
    /// 
    /// eg: -t "npm repository",package,query
    #[arg(short, long, use_value_delimiter=true, value_delimiter=',',value_name= "COMMA_SEPARATED_TOPICS", verbatim_doc_comment)]
    topics: Option<Vec<String>>,
    
    /// Flag for public
    #[arg(value_enum)]
    #[arg(default_value_t = Visibility::Private)]
    #[arg(short = 'v', long)]
    visibility: Visibility,

}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Visibility {
    /// Private github repository
    Private,
    /// Public github repository
    Public,
}

fn clone_repository(params: &Cli) -> String {
    
    let temp_repo = tempdir().expect("Could not create tmp directory");
    
    // Clone repository to tmp folder, with all branches -- and with custom tmp name
    Command::new("git").arg("clone")
        .arg(&params.git_url)
        .arg(temp_repo.path())
        .arg("--mirror")
        .stdout(Stdio::inherit())
        .print_command()
        .output()
        .expect("Failed to execute git clone")
        .print_stderr_if_error();

    let github_slug = params.github_slug.as_ref().unwrap_or(&params.github_repo_name);

    let git_url = format!("git@github.com:{}.git", github_slug);
    let github_url = format!("https://github.com/{}", github_slug);

    // Create the github repository with the parameters specified
    let mut create_repo_command = Command::new("gh");
    create_repo_command
        .arg("repo").arg("create")
        .stdout(Stdio::inherit());
    
    create_repo_command.arg(&params.github_repo_name);
    
    match &params.visibility {
        Visibility::Private => {create_repo_command.arg("--private"); },
        Visibility::Public => { create_repo_command.arg("--public"); } ,
    }
    
    // Add description
    if let Some(description) = &params.description {
        create_repo_command.args(["--description", description]);    
    }
    
    create_repo_command.print_command().output().expect("Failed to execute gh repo create").print_stderr_if_error();

    // Edit new repo

    // Add permissions
    if let Some(teams) = &params.permission {
        for team_and_permission in teams {
            
            let mut splitter = team_and_permission.split(':');
            let team = splitter.next().expect("Expected team in iterator");
            let permission = splitter.next().unwrap_or("pull");
            
            let mut child = Command::new("gh").arg("repo-collab").arg("add")
                .arg(&params.github_repo_name)
                .arg(&team)
                .args(["--permission", permission])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .print_command()
                .spawn()
                .expect(format!("Failed to execute").as_str());

            {
                let child_stdin = child.stdin.as_mut().unwrap();
                child_stdin.write_all(b"y").expect("Could not write to child");
            }

            child.wait_with_output().expect("Failed to wait for child").print_stderr_if_error();
            
        }
        
    }

    // Add topics
    if let Some(topics) = &params.topics {
        for topic in topics {
            Command::new("gh").arg("repo").arg("edit")
                .arg(&params.github_repo_name)
                .args(["--add-topic", topic])
                .stdout(Stdio::inherit())
                .print_command()
                .output().expect(format!("Failed to execute gh repo edit --add-topic {}",topic).as_str())
                .print_stderr_if_error();
        }
    }

    // Push to github

    Command::new("git").args(["-C", temp_repo.path().to_str().unwrap()]).arg("push")
        .arg("--mirror")
        .arg(&git_url)
        .stdout(Stdio::inherit()).print_command().output()
        .expect("Failed to execute git push").print_stderr_if_error();

    return github_url;
}

trait OutputExt {
    fn print_stderr_if_error(self) -> Output;
}

trait CommandExt {
    fn print_command(&mut self) -> &mut Command;
    
}

impl CommandExt for Command {
    fn print_command(&mut self) -> &mut Command {
        println!("{} {}", self.get_program().to_str().unwrap(), self.get_args().map(|s| s.to_str().unwrap()).fold("".to_string(), |cur, nxt| cur + " " + nxt));
        
        self
    }
}

impl OutputExt for Output {
    fn print_stderr_if_error(self) -> Output {
        if !self.status.success() { eprint!("{}", String::from_utf8(self.stderr).expect("Could not parse command")); std::process::exit(1); }
        self
    }
}

fn main() {
    
    let args = Cli::parse();

    println!("==== Migrating {} to GitHub. ====", &args.git_url);
    
    // Check if auth'ed in github

    let auth = Command::new("gh").args(["auth", "status"]).print_command()
        .stdout(Stdio::inherit())
        .output()
        .expect("Failed to execute gh auth status -- is gh installed?");

    if !auth.status.success() { eprintln!("You need to log in using gh auth login."); std::process::exit(1); }

    // Call github thingies
    let url = clone_repository(&args);

    println!("Migration successful for {}: {}", args.github_repo_name, url);
}
