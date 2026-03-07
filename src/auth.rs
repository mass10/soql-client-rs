use serde::Deserialize;
use std::fmt;
use std::process::Command;

fn sf_command() -> Command {
	if cfg!(windows) {
		let mut cmd = Command::new("cmd");
		cmd.args(["/C", "sf"]);
		cmd
	} else {
		Command::new("sf")
	}
}

#[derive(Debug)]
pub enum Error {
	NotLoggedIn,
	SfNotFound,
	SfFailed(String),
	Json(serde_json::Error),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::NotLoggedIn => write!(f, "Not logged in. Run `soql-client login` first."),
			Self::SfNotFound => write!(
				f,
				"`sf` CLI not found. Install it from https://developer.salesforce.com/tools/salesforcecli"
			),
			Self::SfFailed(msg) => write!(f, "sf command failed: {}", msg),
			Self::Json(e) => write!(f, "Failed to parse sf output: {}", e),
		}
	}
}

impl std::error::Error for Error {}

impl From<serde_json::Error> for Error {
	fn from(e: serde_json::Error) -> Self {
		Self::Json(e)
	}
}

#[derive(Debug)]
pub struct Credentials {
	pub access_token: String,
	pub instance_url: String,
}

#[derive(Deserialize)]
struct SfDisplayOutput {
	result: SfDisplayResult,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SfDisplayResult {
	access_token: Option<String>,
	instance_url: Option<String>,
}

pub fn login(target_org: Option<&str>) -> Result<(), Error> {
	check_sf()?;

	let mut cmd = sf_command();
	cmd.args(["org", "login", "web"]);
	if let Some(org) = target_org {
		cmd.args(["--alias", org]);
	}

	let status = cmd.status().map_err(|_| Error::SfNotFound)?;
	if !status.success() {
		return Err(Error::SfFailed("sf org login web failed".into()));
	}
	Ok(())
}

pub fn get_credentials(target_org: Option<&str>) -> Result<Credentials, Error> {
	check_sf()?;

	match try_get_credentials(target_org) {
		Ok(creds) => Ok(creds),
		Err(_) => {
			eprintln!("No active session found. Starting sf org login web...");
			login(target_org)?;
			try_get_credentials(target_org)
		}
	}
}

fn try_get_credentials(target_org: Option<&str>) -> Result<Credentials, Error> {
	let mut cmd = sf_command();
	cmd.args(["org", "display", "--json"]);
	if let Some(org) = target_org {
		cmd.args(["--target-org", org]);
	}

	let output = cmd.output().map_err(|_| Error::SfNotFound)?;
	if !output.status.success() {
		return Err(Error::NotLoggedIn);
	}

	let parsed: SfDisplayOutput = serde_json::from_slice(&output.stdout)?;
	let access_token = parsed.result.access_token.ok_or(Error::NotLoggedIn)?;
	let instance_url = parsed.result.instance_url.ok_or(Error::NotLoggedIn)?;

	Ok(Credentials {
		access_token,
		instance_url,
	})
}

fn check_sf() -> Result<(), Error> {
	sf_command().arg("--version").output().map_err(|_| Error::SfNotFound)?;
	Ok(())
}
