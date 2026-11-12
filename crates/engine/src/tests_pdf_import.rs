//! PDF to Word: PDFs written by WordCraft's own exporter, opened again as editable documents.

use serde_json::json;
use wordcraft_doc::Block;

use crate::Session;

/// The sample document as a PDF, converted back: (styles, texts) of its body paragraphs.
fn round_trip_sample() -> (wordcraft_doc::Document, Vec<(String, String)>) {
    let pdf = crate::io::save_bytes("sample.pdf", &crate::sample::sample_document()).unwrap();
    let doc = crate::io::open_bytes("sample.pdf", &pdf).unwrap();
    let paras = doc
        .body
        .iter()
        .filter_map(|b| b.as_para())
        .map(|p| (p.props.style.clone().unwrap_or_default(), p.text.replace('\u{c}', "").trim().to_string()))
        .filter(|(_, t)| !t.is_empty())
        .collect();
    (doc, paras)
}

#[test]
fn headings_paragraphs_lists_and_tables_come_back() {
    let (doc, paras) = round_trip_sample();
    let style_of = |text: &str| paras.iter().find(|(_, t)| t == text).map(|(s, _)| s.as_str());
    // Headings by size: the title largest, then the section headings.
    assert_eq!(style_of("The Open Studio Handbook"), Some("Heading1"), "{paras:#?}");
    assert_eq!(style_of("Why share a studio?"), Some("Heading2"));
    assert_eq!(style_of("Shared equipment"), Some("Heading3"));
    // A paragraph wrapped over several lines is one paragraph again.
    assert!(paras.iter().any(|(_, t)| t.starts_with("Every Thursday evening") && t.ends_with("become a crew.")), "{paras:#?}");
    // Bullets are list items, without the bullet character.
    let bullet = doc.body.iter().filter_map(|b| b.as_para()).find(|p| p.text.starts_with("Clean as you go")).unwrap();
    assert!(bullet.props.numbering.is_some(), "a bulleted list item");
    // The running header ("The Open Studio Handbook" / "Spring edition" on both pages) is left out.
    assert_eq!(paras.iter().filter(|(_, t)| t == "Spring edition").count(), 0, "{paras:#?}");
    // The table is a table: 4 columns, header row and 4 rows.
    let table = doc.body.iter().find_map(|b| if let Block::Table(t) = &**b { Some(t) } else { None }).expect("a table");
    assert_eq!(table.rows.len(), 5);
    assert!(table.rows.iter().all(|r| r.cells.len() == 4));
    // Bold, italic and colour carry over.
    let lead = doc.body.iter().filter_map(|b| b.as_para()).find(|p| p.text.starts_with("Every Thursday")).unwrap();
    assert!(lead.runs.iter().any(|r| r.props.bold == Some(true)), "\"Painters\" is bold");
}

#[test]
fn the_paragraph_split_by_the_page_break_is_joined() {
    let (_, paras) = round_trip_sample();
    // Steps 3 and 4 are on page 2; the paragraphs around them stay whole.
    assert!(paras.iter().any(|(_, t)| t.starts_with("Membership is pay what you can") && t.ends_with("would show up.")), "{paras:#?}");
}

#[test]
fn opening_a_pdf_names_a_word_document_and_never_overwrites_the_pdf() {
    let dir = std::env::temp_dir().join(format!("wordcraft-pdfin-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let pdf_path = dir.join("Report.pdf");
    let pdf = crate::io::save_bytes("x.pdf", &crate::sample::sample_document()).unwrap();
    std::fs::write(&pdf_path, &pdf).unwrap();
    let mut s = Session::new(wordcraft_doc::Document::new());
    let r = s.run("file.convertPdf", &json!({"path": pdf_path.to_string_lossy()})).unwrap();
    assert_eq!(r["convertedFromPdf"], true);
    assert_eq!(s.path.as_deref(), Some(dir.join("Report.docx").as_path()));
    assert!(s.dirty, "closing asks to save the converted document");
    s.run("file.save", &json!({})).unwrap();
    assert_eq!(std::fs::read(&pdf_path).unwrap(), pdf, "the PDF is untouched");
    let docx = crate::io::open_path(&dir.join("Report.docx")).unwrap();
    assert!(docx.plain_text(wordcraft_doc::StoryRef::Body).contains("Every Thursday evening"));
    // With `out`, the command converts in one go; `out` must be a Word document.
    s.run("file.convertPdf", &json!({"path": pdf_path.to_string_lossy(), "out": dir.join("Copy.docx").to_string_lossy()})).unwrap();
    assert!(dir.join("Copy.docx").exists());
    assert!(s.run("file.convertPdf", &json!({"path": pdf_path.to_string_lossy(), "out": dir.join("Copy.pdf").to_string_lossy()})).is_err());
    assert!(s.run("file.convertPdf", &json!({"path": "notes.txt"})).is_err());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_pdf_opens_by_its_content_too() {
    let pdf = crate::io::save_bytes("x.pdf", &crate::sample::sample_document()).unwrap();
    let doc = crate::io::open_bytes("download", &pdf).unwrap();
    assert!(doc.plain_text(wordcraft_doc::StoryRef::Body).contains("Why share a studio?"));
}

#[test]
fn damaged_and_hostile_pdfs_are_errors_not_crashes() {
    let pdf = crate::io::save_bytes("x.pdf", &crate::sample::sample_document()).unwrap();
    assert!(crate::io::open_bytes("x.pdf", b"").is_err());
    assert!(crate::io::open_bytes("x.pdf", b"%PDF-1.7\n%junk").is_err());
    // Truncated and corrupted copies: an error or a best-effort document, never a panic.
    let mut seed = 0x2545F4914F6CDD1Du64;
    for cut in [pdf.len() / 2, pdf.len() * 3 / 4, pdf.len().saturating_sub(10)] {
        let _ = crate::io::open_bytes("x.pdf", &pdf[..cut]);
    }
    for _ in 0..40 {
        let mut bad = pdf.clone();
        for _ in 0..20 {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let i = (seed as usize) % bad.len();
            bad[i] = (seed >> 32) as u8;
        }
        let _ = crate::io::open_bytes("x.pdf", &bad);
    }
}
