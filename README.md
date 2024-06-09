# Email Address Extractor

A blazingly fast command line tool written in pure safe Rust to automatically extract email addresses from files in a given path.

## Supported File Types

- [x] Plain text (txt, csv, sql, json, html, xml etc.)
- [x] Portable Document Format (pdf)
- [x] Microsoft Word (docx)
- [x] Microsoft Excel (xlsx)
- [x] Microsoft Power Point (pptx)
- [x] OpenOffice Writer (odt)
- [x] OpenOffice Spreadsheet (ods)
- [x] OpenDocument Presentation (odp)

## Usage

```bash
  cargo run --release /path/to/file
```

The tool writes the extracted email addresses to a plain text file called `emails.txt` in the current directory.

## Samples

Using the [sample file provided by Have I Been Pwned](https://mega.nz/file/Xk91ETzb#UYklfa84pLs5OzrysEGNFVMbFb5OC0KU7rlnugF_Aps), which contains 10 million records of typical breach data, we confirm that this tool extracts exactly 10 million email addresses successfully.

In the `\sample` folder, multiple example files are available to showcase this tool's ability to extract email addresses from various file formats.

These sample files are provided:

- file.html
- file.json
- file.odp
- file.ods
- file.odt
- file.pdf
- file.pptx
- file.sql
- file.txt
- file.xlsx
- file.xml

## Background

This project is inspired by [Have I Been Pwned](https://github.com/HaveIBeenPwned/EmailAddressExtractor) and aims to help extract email addresses from data breaches, which are commonly in plain text file formats such as csv or sql. Utilizing a `HashSet`, we ensure that the output has no duplicates.

Handling a variety of different file types requires some effort. Not all file formats use the same encoding, and some file formats are actually zip archives containing several different file types, such as xml. We use magic numbers to identify the MIME type of the file, and then try to extract the textual content based on that knowledge.

To extract email addresses from text, we use the well-known regular expression `\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b`, which matches _any_ email address sufficiently well. Formulating a _perfect_ regular expression to validate an email address is actually not trivial. [The](https://www.regular-expressions.info/email.html) [subject](https://emailregex.com/) [is](https://stackoverflow.com/a/201378) [controversial](<https://html.spec.whatwg.org/multipage/input.html#e-mail-state-(type%3Demail)>). The only way to really validate an email address is to send an email to it, which we are obviously not going to do.
