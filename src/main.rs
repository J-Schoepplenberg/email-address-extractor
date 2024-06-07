mod file;
use env_logger::{self, Builder};
use file::Document;
use log::{error, info};
use std::{
    collections::HashSet,
    env,
    fs::File,
    io::{self, Write},
};

/// Extracts email addresses from a list of strings using regex.
///
/// See: https://www.regular-expressions.info/email.html for a discussion about how to find an email address.
///
/// Uses a HashSet to handle deduplication inherently without sorting and deduping the list explicitly.
fn extract_emails(content: Vec<String>) -> Vec<String> {
    let regex = regex::Regex::new(r"(\b[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}\b)").unwrap(); // won't panic
    content
        .iter()
        .flat_map(|line| regex.captures_iter(line).map(|cap| cap[0].to_string()))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}

/// Writes the extracted emails to a plain text file.
fn write_emails_to_file(emails: &[String]) -> io::Result<String> {
    let output_path = env::current_dir()?.join("emails.txt");
    let mut file = File::create(&output_path)?;
    for email in emails {
        writeln!(file, "{}", email)?;
    }
    output_path.to_str().map(String::from).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::Other,
            "Failed to convert output path to string.",
        )
    })
}

fn main() {
    Builder::from_default_env()
        .write_style(env_logger::WriteStyle::Always)
        .filter_level(log::LevelFilter::Trace)
        .init();
    let args = env::args().collect::<Vec<String>>();
    if args.len() != 2 {
        error!("Path is missing. Usage: \"{} <path\\to\\file>\".", args[0]);
        std::process::exit(1);
    }
    let input_path = &args[1];
    match Document::open(input_path) {
        Ok(file) => {
            info!("Document processed successfully.");
            let emails = extract_emails(file.content);
            info!("File path: {}.", input_path);
            info!("File size: {} bytes.", file.size);
            info!("Extracted emails: {}.", emails.len());
            match write_emails_to_file(&emails) {
                Ok(path) => info!("Data written to {} successfully.", path),
                Err(e) => error!("Failed to write emails to file. {}", e),
            }
            std::process::exit(0);
        }
        Err(e) => {
            error!("Failed to open document. {}", e);
            std::process::exit(1);
        }
    }
}
