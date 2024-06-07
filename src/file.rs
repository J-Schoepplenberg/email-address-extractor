use pdf_extract::extract_text_from_mem;
use std::fs::{self, File};
use std::io::{self, BufReader, Read};
use std::path::Path;
use zip::ZipArchive;

/// Represents the mime type of a file.
#[derive(Debug)]
pub enum FileType {
    PlainText,
    Pdf,
    Xlsx,
    Odt,
    Docx,
    Pptx,
    Zip,
    Unsupported,
}

/// Represents a file with its content and other metadata.
#[derive(Debug)]
pub struct Document {
    pub content: Vec<String>,
    pub metadata: fs::Metadata,
    pub size: u64,
    pub mime_type: FileType,
}

impl Document {
    /// Opens the file given by the specified path.
    ///
    /// Returns a File struct containing the already extracted content and other metadata.
    ///
    /// May fail if the path does not exist or if the file cannot be read.
    pub fn open<P: AsRef<Path>>(file: P) -> io::Result<Self> {
        let path = file.as_ref();
        let file = fs::File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut buffer = vec![];
        reader.read_to_end(&mut buffer)?;
        let mime_type = Self::identify_mime_type(&buffer);
        let content = Self::process_file(&mime_type, reader, &buffer)?;
        let metadata = fs::metadata(path)?;
        let size = metadata.len();
        Ok(Document {
            content,
            metadata,
            size,
            mime_type,
        })
    }

    /// Identifies file types using file signatures, also known as magic numbers.
    ///
    /// See: https://en.wikipedia.org/wiki/List_of_file_signatures.
    ///
    /// Plain text files (e.g. txt, csv, json) do not have magic numbers.
    fn identify_mime_type(buffer: &[u8]) -> FileType {
        match buffer {
            _ if infer::doc::is_docx(buffer) => FileType::Docx,
            _ if infer::doc::is_pptx(buffer) => FileType::Pptx,
            _ if infer::doc::is_xlsx(buffer) => FileType::Xlsx,
            _ if infer::odf::is_odt(buffer) => FileType::Odt,
            _ if infer::archive::is_pdf(buffer) => FileType::Pdf,
            _ if infer::archive::is_zip(buffer) => FileType::Zip,
            _ if infer::get(buffer).is_none() => FileType::PlainText,
            _ => FileType::Unsupported,
        }
    }

    /// Extracts the content of the file based on its mime type.
    ///
    /// Returns a vector of strings, where each string represents a line of text.
    fn process_file(
        mime_type: &FileType,
        reader: BufReader<File>,
        buffer: &[u8],
    ) -> io::Result<Vec<String>> {
        match mime_type {
            FileType::Zip | FileType::Xlsx | FileType::Odt | FileType::Docx | FileType::Pptx => {
                Self::process_zip(reader)
            }
            FileType::PlainText => Self::process_plain_text(buffer),
            FileType::Pdf => Self::process_pdf(buffer),
            FileType::Unsupported => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unsupported file type.",
            )),
        }
    }

    /// Extracts plain text from a given buffer.
    fn process_plain_text(buffer: &[u8]) -> io::Result<Vec<String>> {
        Ok(String::from_utf8_lossy(buffer)
            .lines()
            .map(String::from)
            .collect())
    }

    /// Extracts text from a PDF file.
    ///
    /// May fail if the PDF is encrypted and due to other reasons.
    fn process_pdf(buffer: &[u8]) -> io::Result<Vec<String>> {
        extract_text_from_mem(buffer)
            .map(|text| vec![text])
            .map_err(|err| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to extract PDF text. {}.", err),
                )
            })
    }

    /// Extracts text from a zip file.
    ///
    /// Many file types are actually zip archives (e.g. ODT, DOCX, PPTX) containing xml files.
    fn process_zip(reader: BufReader<File>) -> io::Result<Vec<String>> {
        let file = reader.into_inner();
        let mut archive = ZipArchive::new(file).map_err(|err| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to read ZIP archive. {}.", err),
            )
        })?;
        let mut result = Vec::new();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|err| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to access file in archive. {}.", err),
                )
            })?;
            let mut buf = String::new();
            file.read_to_string(&mut buf)?;
            result.push(buf);
        }
        Ok(result)
    }
}