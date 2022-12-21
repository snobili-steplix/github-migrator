use std::io;
use std::process::Command;
use tempfile;
use tempfile::tempdir;

struct CliParams {
    pub gitlab_url: String,
    pub github_repository_name: String,
        
    pub team_name: Option<String>,
    pub public: Option<String>,
}

impl CliParams {
    fn from_string(input: String) -> Self {

        let inputs : Vec<_> = input.split_ascii_whitespace().collect();


        Self {
            gitlab_url : inputs[0].to_string(),
            github_repository_name : inputs[1].to_string(),
            
            public : inputs.get(4).map(|i| i.to_string()),
            team_name : inputs.get(3).map(|i| i.to_string()),
        }
    }
}

fn clone_repository(params: &CliParams) -> String {
    
    let temp_repo = tempdir().expect("Could not create tmp directory");

    // Clone repository to tmp folder, with all branches -- and with custom tmp name
    let clone = Command::new("git").arg("clone")
        .arg(&params.gitlab_url)
        .arg(temp_repo.path())
        .arg("--mirror")
        .output()
        .expect("Failed to execute git clone");

    if !clone.status.success() { eprint!("{}", String::from_utf8(clone.stderr).expect("Could not parse git clone error")); std::process::exit(1); }

    let git_url = format!("git@github.com:{}.git", &params.github_repository_name);
    let github_url = format!("https://github.com/{}", &params.github_repository_name);

    // Create the github repository with the parameters specified
    let mut create_repo_command = Command::new("gh");
    create_repo_command
        .arg("repo").arg("create");
    
    create_repo_command.arg(&params.github_repository_name);
    
    match &params.public {
        Some(val) => {
            if val.eq_ignore_ascii_case("public") {
                create_repo_command.arg("--public");
            } else {
                create_repo_command.arg("--private");
            }
        }
        None => {
            create_repo_command.arg("--private");
        },
    }

    match &params.team_name {
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
    println!("Migrating for the following repositories.");
    println!("Pattern should be\ngitlab_url github_repository_name [team_name]");
    
    // Check if auth'ed in github
    let auth = Command::new("git").args(["auth", "status"])
        .status()
        .expect("Failed to execute gh auth status");

    if !auth.success() { eprintln!("You need to log in using gh auth login."); std::process::exit(1); }

    for line_res in io::stdin().lines() {
        let params = CliParams::from_string(line_res.unwrap());

        // Call github thingies
        let url = clone_repository(&params);

        println!("Clone successful for {}: {}", params.github_repository_name, url);
    }

}
