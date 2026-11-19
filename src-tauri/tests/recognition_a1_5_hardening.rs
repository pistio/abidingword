//! Phase A1.5: documents the readers must refuse rather than read in part
//! (independent security review). Invented text only.

use std::io::Write;

use abiding_word_lib::scripture::documents;

fn docx(document_xml: &str) -> std::path::PathBuf {
    let mut bytes = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut bytes));
        let o = zip::write::SimpleFileOptions::default();
        zip.start_file("[Content_Types].xml", o).unwrap();
        zip.write_all(br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#).unwrap();
        zip.start_file("word/document.xml", o).unwrap();
        zip.write_all(document_xml.as_bytes()).unwrap();
        zip.finish().unwrap();
    }
    let dir = std::env::temp_dir().join(format!("aw-a15-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{}.docx", bytes.len()));
    std::fs::write(&path, bytes).unwrap();
    path
}

const HEAD: &str = r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Invented words of a verse</w:t></w:r></w:p>"#;

#[test]
fn a_word_body_that_is_not_well_formed_is_refused_whole() {
    // Read as far as its first error, it looked complete (finding 11).
    let damaged = format!("{HEAD}<w:p><w:r><w:t>more invented words <!-- never closed");
    assert!(documents::read(&docx(&damaged), 0).is_err());
    // The control: the same body, well formed, is read.
    let whole = format!("{HEAD}</w:body></w:document>");
    assert!(documents::read(&docx(&whole), 0).is_ok());
}
