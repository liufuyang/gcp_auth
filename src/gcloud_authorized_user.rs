use std::path::{Path, PathBuf};
use std::process::Command;

use async_trait::async_trait;
use which::which;

use crate::authentication_manager::ServiceAccount;
use crate::error::Error;
use crate::error::Error::{GCloudError, GCloudNotFound, GCloudParseError};
use crate::types::HyperClient;
use crate::Token;

#[derive(Debug)]
pub(crate) struct GCloudAuthorizedUser {
    gcloud: PathBuf,
    project_id: Option<String>,
}

impl GCloudAuthorizedUser {
    pub(crate) fn new() -> Result<Self, Error> {
        let gcloud = which("gcloud").map_err(|_| GCloudNotFound)?;
        let project_id = run(&gcloud, &["config", "get-value", "project"]).ok();
        Ok(Self { gcloud, project_id })
    }

    fn token(&self) -> Result<Token, Error> {
        Ok(Token::from_string(run(
            &self.gcloud,
            &["auth", "print-access-token", "--quiet"],
        )?))
    }
}

#[async_trait]
impl ServiceAccount for GCloudAuthorizedUser {
    async fn project_id(&self, _: &HyperClient) -> Result<String, Error> {
        self.project_id.clone().ok_or(Error::NoProjectId)
    }

    fn get_token(&self, _scopes: &[&str]) -> Option<Token> {
        None
    }

    async fn refresh_token(&self, _client: &HyperClient, _scopes: &[&str]) -> Result<Token, Error> {
        self.token()
    }
}

fn run(gcloud: &Path, cmd: &[&str]) -> Result<String, Error> {
    let mut command = Command::new(gcloud);
    command.args(cmd);

    let mut stdout = match command.output() {
        Ok(output) if output.status.success() => output.stdout,
        _ => return Err(GCloudError),
    };

    while let Some(b' ' | b'\r' | b'\n') = stdout.last() {
        stdout.pop();
    }

    String::from_utf8(stdout).map_err(|_| GCloudParseError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn gcloud() {
        let gcloud = GCloudAuthorizedUser::new().unwrap();
        println!("{:?}", gcloud.project_id);
        println!("{:?}", gcloud.token());
    }
}
