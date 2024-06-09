use pdf_extract::extract_text_from_mem;
use std::io::{self, Cursor, Read};
use zip::ZipArchive;

/// Represents different file types that can be processed.
pub enum FileType<'a> {
    Zip(ZipFile<'a>),
    Text(TextFile<'a>),
    Pdf(PdfFile<'a>),
}

/// Represents a zip file as a byte slice reference.
pub struct ZipFile<'a>(&'a [u8]);

/// Represents a text file as a byte slice reference.
pub struct TextFile<'a>(&'a [u8]);

/// Represents a pdf file as a byte slice reference.
pub struct PdfFile<'a>(&'a [u8]);

impl<'a> AsRef<[u8]> for ZipFile<'a> {
    /// Converts a `ZipFile` to its byte slice reference.
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Trait for processing different file types.
pub trait ProcessFile<'a> {
    /// Attempts to process the given byte slice and return a vector of strings containing the extracted text.
    fn process(&'a self) -> io::Result<Vec<String>>;
}

impl<'a> ProcessFile<'a> for ZipFile<'a> {
    /// Attempts to parse a given byte slice as a zip archive and extracts the content of its xml files as strings.
    fn process(&'a self) -> io::Result<Vec<String>> {
        // Makes the byte slice readable by wrapping it with Cursor.
        let reader = Cursor::new(self.0);
        let mut archive = ZipArchive::new(reader)?;
        let mut xml = Vec::new();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            // Ensures we only read valid UTF-8 streams.
            if file.name().ends_with(".xml") {
                let mut buffer = String::new();
                file.read_to_string(&mut buffer)?;
                xml.push(buffer);
            }
        }
        Ok(xml)
    }
}

impl<'a> ProcessFile<'a> for TextFile<'a> {
    /// Converts a given byte slice to a string and splits it into lines.
    ///
    /// Invalid UTF-8 sequences get replaced with �.
    fn process(&'a self) -> io::Result<Vec<String>> {
        Ok(String::from_utf8_lossy(&self.0)
            .lines()
            .map(String::from)
            .collect())
    }
}

impl<'a> ProcessFile<'a> for PdfFile<'a> {
    /// Attempts to parse the given byte slice as a pdf file and extract its text.
    fn process(&'a self) -> io::Result<Vec<String>> {
        extract_text_from_mem(&self.0)
            .map(|text| vec![text])
            .map_err(|err| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to extract PDF text. {}.", err),
                )
            })
    }
}

// Implementing `TryFrom` provides an equivalent `TryInto` implementation for free.
impl<'a> TryFrom<&'a [u8]> for FileType<'a> {
    type Error = io::Error;

    /// Attempts to convert the given byte buffer into a supported `FileType` without panicking.
    ///
    /// Many file formats are actually zip archives containing other files such as xml.
    /// 
    /// For unknown MIME types we default to plain text assuming they contain valid UTF-8.
    ///
    /// Supported:
    ///     - plain text (e.g. txt, csv, sql, json, xml, html)
    ///     - zip archives containing xml (e.g. odp, ods, odt, docx)
    ///     - pdf files
    fn try_from(bytes: &'a [u8]) -> io::Result<FileType<'a>> {
        if let Some(t) = infer::get(bytes) {
            match t.mime_type() {
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" // xlsl
                | "application/vnd.oasis.opendocument.presentation" // odp
                | "application/vnd.oasis.opendocument.spreadsheet" // ods
                | "application/vnd.oasis.opendocument.text" // odt
                | "application/msword" // docx
                | "application/zip" => Ok(FileType::Zip(ZipFile(bytes))),
                "application/pdf" => Ok(FileType::Pdf(PdfFile(bytes))),
                "text/html" | "text/xml" => Ok(FileType::Text(TextFile(bytes))),
                mime_type => Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    format!("Unsupported file type: {}", mime_type),
                )),
            }
        } else {
            Ok(FileType::Text(TextFile(bytes)))
        }
    }
}

impl<'a> FileType<'a> {
    /// Attempts to process the different `FileType` variants.
    ///
    /// Extracts available text content from the files.
    pub fn process(self) -> io::Result<Vec<String>> {
        match self {
            FileType::Text(text_file) => text_file.process(),
            FileType::Zip(zip_file) => zip_file.process(),
            FileType::Pdf(pdf_file) => pdf_file.process(),
        }
    }
}

/// Helper trait to avoid type annotations and allow function chaining.
pub trait TryIntoFileType<'a> {
    /// Attempts to convert the given byte buffer into a `FileType` using `try_into()`.
    ///
    /// Avoids panicking if the conversion fails.
    fn try_into_filetype(self) -> io::Result<FileType<'a>>;
}

impl<'a> TryIntoFileType<'a> for &'a [u8] {
    fn try_into_filetype(self) -> io::Result<FileType<'a>> {
        self.try_into()
    }
}
