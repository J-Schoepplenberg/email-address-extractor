mod file;
use env_logger::Builder;
use file::TryIntoFileType;
use log::{error, info, warn};
use std::{
    collections::HashSet,
    env,
    fs::{self, File},
    io::{self, BufReader, Read, Write},
};

/// Extracts email addresses from a list of strings using regex.
///
/// See: https://www.regular-expressions.info/email.html for a discussion about how to find an email address.
///
/// Uses a HashSet to handle deduplication inherently without sorting and deduping the list explicitly.
fn extract_emails(content: &[String]) -> Vec<String> {
    let regex = regex::Regex::new(r"(\b[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}\b)").unwrap(); // won't panic
    content
        .iter()
        .flat_map(|line| regex.captures_iter(line).map(|cap| cap[0].to_string()))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}

/// Attempts to write the extracted emails to a plain text file.
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

/// Attempts to process the file from the given path and extract email addresses.
fn process_file(input_path: &str) -> io::Result<()> {
    let metadata = fs::metadata(input_path)?;
    info!("File path: {}.", input_path);
    info!("File size: {} bytes.", metadata.len());

    let file = File::open(input_path)?;
    let mut reader = BufReader::new(file);
    let mut buffer = vec![];
    reader.read_to_end(&mut buffer)?;

    let processed = buffer.try_into_filetype()?.process()?;
    info!("File processed successfully.");

    let emails = extract_emails(&processed);
    if !emails.is_empty() {
        match write_emails_to_file(&emails) {
            Ok(path) => info!("Extracted emails written to {} successfully.", path),
            Err(e) => error!("Failed to write emails to file. {}.", e),
        }
    } else {
        warn!("No email address found.");
    }

    Ok(())
}

fn main() {
    Builder::from_default_env()
        .write_style(env_logger::WriteStyle::Always)
        .filter_level(log::LevelFilter::Trace)
        .init();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        error!("Path is missing. Usage: \"{} <path\\to\\file>\".", args[0]);
        std::process::exit(1);
    }
    let input_path = &args[1];

    if let Err(e) = process_file(input_path) {
        error!("Application error: {}.", e);
        std::process::exit(1);
    }
}
