use std::process::Command;
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
    #[arg(short = 'g', long)]
    team: Option<String>,

    /// Topics to add to the repository
    /// 
    /// eg: -t "npm repository",package,query
    #[arg(short, long, use_value_delimiter=true, value_delimiter=',',value_name= "COMMA_SEPARATED_TOPICS", verbatim_doc_comment)]
    topics: Option<Vec<String>>,
    
    /// Flag for public
    #[arg(value_enum)]
    #[arg(default_value_t = Visibility::Private)]
    #[arg(short = 'p', long)]
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
    let clone = Command::new("git").arg("clone")
        .arg(&params.git_url)
        .arg(temp_repo.path())
        .arg("--mirror")
        .output()
        .expect("Failed to execute git clone");

    if !clone.status.success() { eprint!("{}", String::from_utf8(clone.stderr).expect("Could not parse git clone error")); std::process::exit(1); }

    let github_slug = params.github_slug.as_ref().unwrap_or(&params.github_repo_name);

    let git_url = format!("git@github.com:{}.git", github_slug);
    let github_url = format!("https://github.com/{}", github_slug);

    // Create the github repository with the parameters specified
    let mut create_repo_command = Command::new("gh");
    create_repo_command
        .arg("repo").arg("create");
    
    create_repo_command.arg(&params.github_repo_name);
    
    match &params.visibility {
        Visibility::Private => {create_repo_command.arg("--private"); },
        Visibility::Public => { create_repo_command.arg("--public"); } ,
    }
        
    match &params.team {
        Some(val) => {create_repo_command.args(["--team", &val]);},
        None => {},
    }
    
    let create_repo = create_repo_command.output().expect("Failed to execute gh repo create");

    if !create_repo.status.success() { eprint!("{}", String::from_utf8(create_repo.stderr).expect("Could not parse gh repo create")); std::process::exit(1); }

    // Push to github

    let push = Command::new("git").arg("push").arg("--mirror").arg(&git_url).output().expect("Failed to execute git push");
    if !push.status.success() { eprint!("{}", String::from_utf8(push.stderr).expect("Could not parse git push")); std::process::exit(1); }


    return github_url;
}

fn main() {
    
    let args = Cli::parse();

    println!("Migrating {} to GitHub.", &args.git_url);
    
    
    // Check if auth'ed in github
    let auth = Command::new("git").args(["auth", "status"])
        .status()
        .expect("Failed to execute gh auth status -- is gh installed?");

    if !auth.success() { eprintln!("You need to log in using gh auth login."); std::process::exit(1); }

    // Call github thingies
    let url = clone_repository(&args);

    println!("Migration successful for {}: {}", args.github_repo_name, url);
}
