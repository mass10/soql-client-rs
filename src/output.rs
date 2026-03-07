use comfy_table::{ContentArrangement, Table};
use serde_json::Value;

use crate::query::QueryResult;

pub enum OutputFormat {
	Json,
	Table,
	Csv,
	Tsv,
}

impl OutputFormat {
	pub fn from_str(s: &str) -> Result<Self, String> {
		match s.to_lowercase().as_str() {
			"json" => Ok(Self::Json),
			"table" => Ok(Self::Table),
			"csv" => Ok(Self::Csv),
			"tsv" => Ok(Self::Tsv),
			_ => Err(format!("Unknown output format: {}. Use json, table, csv, or tsv.", s)),
		}
	}
}

pub fn print_result(result: &QueryResult, format: &OutputFormat) -> Result<(), Box<dyn std::error::Error>> {
	if result.records.is_empty() {
		eprintln!("No records found. (totalSize: {})", result.total_size);
		return Ok(());
	}

	let columns = collect_columns(&result.records);

	match format {
		OutputFormat::Json => print_json(result),
		OutputFormat::Table => print_table(&result.records, &columns),
		OutputFormat::Csv => print_csv(&result.records, &columns),
		OutputFormat::Tsv => print_tsv(&result.records, &columns),
	}
}

fn collect_columns(records: &[Value]) -> Vec<String> {
	let mut columns = Vec::new();
	if let Some(first) = records.first() {
		if let Some(obj) = first.as_object() {
			for key in obj.keys() {
				if key != "attributes" {
					columns.push(key.clone());
				}
			}
		}
	}
	columns.sort();
	columns
}

fn value_to_string(val: &Value) -> String {
	match val {
		Value::Null => String::new(),
		Value::String(s) => s.clone(),
		Value::Bool(b) => b.to_string(),
		Value::Number(n) => n.to_string(),
		Value::Object(_) | Value::Array(_) => serde_json::to_string(val).unwrap_or_default(),
	}
}

fn print_json(result: &QueryResult) -> Result<(), Box<dyn std::error::Error>> {
	let output = serde_json::to_string_pretty(&result.records)?;
	println!("{}", output);
	Ok(())
}

fn print_table(records: &[Value], columns: &[String]) -> Result<(), Box<dyn std::error::Error>> {
	let mut table = Table::new();
	table.set_content_arrangement(ContentArrangement::Dynamic);
	table.set_header(columns);

	for record in records {
		let row: Vec<String> = columns
			.iter()
			.map(|col| record.get(col).map(value_to_string).unwrap_or_default())
			.collect();
		table.add_row(row);
	}

	println!("{table}");
	Ok(())
}

fn print_csv(records: &[Value], columns: &[String]) -> Result<(), Box<dyn std::error::Error>> {
	let stdout = std::io::stdout();
	let mut writer = csv::Writer::from_writer(stdout.lock());

	writer.write_record(columns)?;

	for record in records {
		let row: Vec<String> = columns
			.iter()
			.map(|col| record.get(col).map(value_to_string).unwrap_or_default())
			.collect();
		writer.write_record(&row)?;
	}

	writer.flush()?;
	Ok(())
}

fn print_tsv(records: &[Value], columns: &[String]) -> Result<(), Box<dyn std::error::Error>> {
	let stdout = std::io::stdout();
	let mut writer = csv::WriterBuilder::new().delimiter(b'\t').from_writer(stdout.lock());

	writer.write_record(columns)?;

	for record in records {
		let row: Vec<String> = columns
			.iter()
			.map(|col| record.get(col).map(value_to_string).unwrap_or_default())
			.collect();
		writer.write_record(&row)?;
	}

	writer.flush()?;
	Ok(())
}
