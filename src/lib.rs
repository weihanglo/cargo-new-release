use anyhow::{bail, Result};
use regex::Regex;
use std::process::{Command, Stdio};

pub trait CommandExt {
    fn git(args: &str) -> Command;
    fn run_stdout(&mut self) -> Result<String>;
    fn display_args(&self) -> String;
    fn run_success(&mut self) -> Result<bool>;
}

impl CommandExt for Command {
    fn git(args: &str) -> Command {
        // TODO: verbose flag to show commands being run.
        let vargs: Vec<_> = args.split_whitespace().collect();
        let mut cmd = Command::new("git");
        cmd.args(&vargs);
        cmd
    }

    fn run_stdout(&mut self) -> Result<String> {
        self.stdout(Stdio::piped());
        match self.output() {
            Ok(output) => {
                if !output.status.success() {
                    bail!(
                        "failed to run `{} {}`: exit status {:?}",
                        self.get_program().to_str().unwrap(),
                        self.display_args(),
                        output.status
                    );
                }
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                Ok(stdout)
            }
            Err(e) => {
                bail!(
                    "failed to spawn `{}`: {}",
                    self.get_program().to_str().unwrap(),
                    e
                );
            }
        }
    }

    fn display_args(&self) -> String {
        let args: Vec<_> = self
            .get_args()
            .into_iter()
            .map(|s| s.to_str().unwrap())
            .collect();
        args.join(" ")
    }

    fn run_success(&mut self) -> Result<bool> {
        match self.status() {
            Ok(status) => {
                if status.code() != Some(0) && status.code() != Some(1) {
                    bail!(
                        "failed to run `{} {}`: exit status {:?}",
                        self.get_program().to_str().unwrap(),
                        self.display_args(),
                        status
                    );
                }
                Ok(status.success())
            }
            Err(e) => {
                bail!(
                    "failed to spawn `{}`: {}",
                    self.get_program().to_str().unwrap(),
                    e
                );
            }
        }
    }
}

/// Returns Vec of `(pr_num, pr_url, pr_description)` tuples.
pub fn commits_in_log(log: &str) -> Vec<(u32, String, String)> {
    let commit_re = Regex::new("(?m)^commit ").unwrap();
    let merge_re = Regex::new("(?:Auto merge of|Merge pull request) #([0-9]+)").unwrap();
    commit_re
        .split(&log)
        .filter(|commit| !commit.trim().is_empty())
        .filter_map(|commit| {
            let hash = commit.split_whitespace().next().expect("hash");
            let mut lines = commit
                .lines()
                .filter(|line| !line.trim().is_empty() && line.starts_with(' '))
                .map(|line| line.trim());
            let first = lines.next().expect("auto");
            let cap = match merge_re.captures(first) {
                Some(m) => m,
                None => {
                    eprintln!(
                        "could not find \"{}\" in line: {}\nhash: {}",
                        merge_re.as_str(),
                        first,
                        hash
                    );
                    return None;
                }
            };
            let num_cap = cap.get(1).expect("group").as_str();
            let pr_num: u32 = match num_cap.parse() {
                Ok(n) => n,
                Err(e) => panic!("could not parse {}: {}", num_cap, e),
            };
            let descr = lines.next().unwrap_or("").to_string();
            let url = format!("https://github.com/rust-lang/cargo/pull/{}", pr_num);
            Some((pr_num, url, descr))
        })
        .collect()
}
