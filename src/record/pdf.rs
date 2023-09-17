use std::{sync::Arc, vec};

use super::{
    error::RecordError,
    record::{Content, Record},
    spin::Spin,
};
use pdf::{
    any::AnySync,
    file::{File, FileOptions, NoLog, SyncCache},
    object::PlainRef,
    PdfError,
};

pub struct PDF {
    file: File<Vec<u8>, Arc<SyncCache<PlainRef, Result<AnySync, Arc<PdfError>>>>, Arc<SyncCache<PlainRef, Result<Arc<[u8]>, Arc<PdfError>>>>, NoLog>,
    split: bool,
}

impl PDF {
    /// Create a new PDF record from a buffer
    /// When calling this function, specify the PDF generic type as a slice of bytes
    /// ```
    /// use resume::record::pdf::PDF;
    ///
    /// let record = PDF::<&[u8]>::from_buffer(include_bytes!("../../tests/records/pdf.in"), false).unwrap();
    /// ```
    pub fn from_buffer(buffer: &[u8], split: bool) -> PDF {
        // convert buffer into file object
        PDF {
            file: FileOptions::cached().load(buffer.to_vec()).unwrap(),
            split,
        }
    }

    /// Create a new PDF record from a file
    /// When calling this function, specify the PDF generic type as a vector of bytes
    /// ```
    /// use resume::record::pdf::PDF;
    ///
    /// let record = PDF::<Vec<u8>>::from_file("test/test.pdf", false).unwrap();
    /// ```
    pub fn from_file(path: &str, split: bool) -> PDF {
        // convert buffer into file object
        PDF {
            file: FileOptions::cached().open(&path).unwrap(),
            split,
        }
    }
}

pub enum PDFOutput {
    String(String),
    Vec(Vec<String>),
}

impl PDFOutput {
    pub fn to_string(self) -> String {
        match self {
            PDFOutput::String(string) => string.to_string(),
            PDFOutput::Vec(vec) => vec.join("\n******************\n"),
        }
    }

    pub fn to_vec(self) -> Vec<String> {
        match self {
            PDFOutput::String(string) => vec![string],
            PDFOutput::Vec(vec) => vec,
        }
    }
}

impl Spin for PDF {
    fn spin(&self) -> Result<Record, RecordError> {
        let resolver = self.file.resolver();
        return if self.split {
            let mut content = Vec::new();
            for page in self.file.pages() {
                let page = page?;
                let mut page_content = String::new();
                let flow = pdf_text::run(&self.file, &page, &resolver)?;
                for run in flow.runs {
                    for line in run.lines {
                        for word in line.words {
                            page_content.push_str(&word.text);
                            page_content.push(' ');
                        }
                        page_content.push('\n');
                    }
                }
                content.push(page_content);
            }
            Ok(Record::new(Content::Vec(content)))
        } else {
            let resolver = self.file.resolver();
            let mut content = String::new();
            for page in self.file.pages() {
                let page = page?;
                let flow = pdf_text::run(&self.file, &page, &resolver)?;
                for run in flow.runs {
                    for line in run.lines {
                        for word in line.words {
                            content.push_str(&word.text);
                            content.push(' ');
                        }
                        content.push('\n');
                    }
                }
            }
            Ok(Record::new(Content::String(content)))
        };
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_from_buffer() {
        let record = PDF::from_buffer(include_bytes!("../../tests/pdf.in"), false);
        assert_eq!(record.split, false);
    }

    #[test]
    fn test_from_file() {
        let record = PDF::from_file("test/test.pdf", false);
        assert_eq!(record.split, false);
    }

    #[test]
    fn test_spin() {
        std::env::set_var("STANDARD_FONTS", "./assets/pdf_fonts");
        let record = PDF::from_file("./tests/sample-resume.pdf", false).spin().unwrap();
        let correct_content = include_str!("../../tests/out/sample-resume.out");
        assert_eq!(record.content.to_string(), correct_content);
    }
}