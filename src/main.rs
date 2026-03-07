mod auth;
mod output;
mod query;

use clap::{Parser, Subcommand};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Parser)]
#[command(name = "soql-client", about = "Salesforce SOQL query CLI")]
struct Cli {
	#[command(subcommand)]
	command: Commands,

	/// Target org alias or username (passed to sf CLI)
	#[arg(short = 'o', long, global = true)]
	target_org: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
	/// Login to Salesforce via sf CLI (opens browser)
	Login,
	/// Execute a SOQL query
	Query {
		/// SOQL query string (or use --file instead)
		soql: Option<String>,
		/// Read SOQL from a file
		#[arg(short = 'i', long)]
		file: Option<std::path::PathBuf>,
		/// Output format: json, table, csv, tsv
		#[arg(short, long, default_value = "table")]
		format: String,
	},
}

#[tokio::main]
async fn main() -> Result<()> {
	let cli = Cli::parse();
	let target_org = cli.target_org.as_deref();

	match cli.command {
		Commands::Login => {
			auth::login(target_org)?;
			let creds = auth::get_credentials(target_org)?;
			println!("Logged in successfully to {}", creds.instance_url);
		}
		Commands::Query { soql, file, format } => {
			let soql = match (soql, file) {
				(Some(s), _) => s,
				(None, Some(path)) => {
					std::fs::read_to_string(&path).map_err(|e| format!("Failed to read SOQL file {}: {}", path.display(), e))?
				}
				(None, None) => {
					return Err("Provide a SOQL query as an argument or use --file / -i".into());
				}
			};
			let soql = soql.trim().to_string();
			let creds = auth::get_credentials(target_org)?;
			let format = output::OutputFormat::from_str(&format)?;
			let result = query::fetch_all(&creds, &soql).await?;
			eprintln!("Total records: {}", result.total_size);
			output::print_result(&result, &format)?;
		}
	}

	Ok(())
}
