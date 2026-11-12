//! PDF import ("convert PDF to Word"): the text, its formatting and the pictures of a PDF, rebuilt
//! as editable paragraphs.
//!
//! A PDF has no paragraphs, only glyphs placed on pages. [`hayro_interpret`] walks each page's
//! content and reports every glyph (its Unicode text, position, size and font) and picture. They
//! are gathered into lines (glyphs on one baseline, spaces where the gaps are), lines into
//! paragraphs (a larger gap, a change of size, a short last line or an indented first line ends
//! one), and paragraphs set larger than the body text become headings. Bold, italic, font, size
//! and colour carry over; words broken with a hyphen at a line end are joined again; running
//! headers, footers and page numbers repeated on the pages are left out; each PDF page starts a
//! new page unless its text continues the previous page's paragraph.
//!
//! Layouts a flow can't express (columns side by side, tables, text on a curve) come out as
//! paragraphs in the order the PDF draws them. A scanned PDF has no text, only page pictures,
//! which are kept as pictures. Input is hostile: pages, glyphs and pictures are capped, numbers
//! checked, and nothing panics.

use std::collections::HashMap;
use std::sync::Arc;

use hayro_interpret::font::Glyph;
use hayro_interpret::hayro_cmap::BfString;
use hayro_interpret::hayro_syntax::Pdf;
use hayro_interpret::{
    BlendMode, ClipPath, Context, Device, GlyphDrawMode, Image, ImageData, InterpreterCache, InterpreterSettings, Paint, PathDrawMode, SoftMask,
    interpret_page,
};
use kurbo::{Affine, Point, Vec2};
use wordcraft_doc::Document;
use wordcraft_doc::props::Align;

use crate::model::{Cell, FBlock, FTable, Flow, Fmt, Img, Inline, Kind, ListInfo, Para};

/// Most pages read.
const MAX_PAGES: usize = 5_000;
/// Most glyphs read from the whole file.
const MAX_GLYPHS: usize = 5_000_000;
/// Most pictures kept, and the largest (pixels) decoded.
const MAX_PICTURES: usize = 500;
const MAX_PICTURE_PIXELS: u64 = 40_000_000;
/// Pictures smaller than this (points, either side) are decoration (bullets, rules) and left out.
const MIN_PICTURE_PT: f64 = 12.0;

/// Does `bytes` look like a PDF (`%PDF-` in the first kilobyte, as readers accept)?
pub fn is_pdf(bytes: &[u8]) -> bool {
    bytes.get(..1024.min(bytes.len())).is_some_and(|head| head.windows(5).any(|w| w == b"%PDF-"))
}

/// Convert a PDF to a document.
pub fn import(bytes: &[u8]) -> Result<Document, String> {
    let pdf = Pdf::new(bytes.to_vec()).map_err(|e| match e {
        hayro_interpret::hayro_syntax::LoadPdfError::Decryption(_) => "this PDF is password-protected; it can't be converted".to_string(),
        other => {
            log::warn!("PDF not readable: {other:?}");
            "this PDF is damaged or incomplete, so it can't be converted".to_string()
        }
    })?;
    let pages = read_pages(&pdf);
    let mut flow = build_flow(&pages);
    let meta = pdf.metadata();
    let text = |v: &Option<Vec<u8>>| v.as_deref().map(decode_pdf_text).unwrap_or_default();
    flow.meta.title = text(&meta.title);
    flow.meta.author = text(&meta.author);
    flow.meta.subject = text(&meta.subject);
    flow.meta.keywords = text(&meta.keywords);
    Ok(crate::model::to_doc(&flow))
}

/// A PDF text string: UTF-16BE with a byte order mark, else PDFDocEncoding (Latin-1 for the
/// characters that matter here).
fn decode_pdf_text(b: &[u8]) -> String {
    if let Some(rest) = b.strip_prefix(&[0xFE, 0xFF]) {
        let units: Vec<u16> = rest.chunks_exact(2).map(|c| u16::from_be_bytes([c[0], c[1]])).collect();
        return String::from_utf16_lossy(&units).trim().to_string();
    }
    if let Ok(s) = std::str::from_utf8(b.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(b)) {
        return s.trim().to_string();
    }
    b.iter().map(|&c| c as char).collect::<String>().trim().to_string()
}

// ---------------------------------------------------------------------------------------------
// Reading pages

/// One glyph as placed on the page (points, y up from the bottom of the page).
#[derive(Clone, Debug)]
struct Gl {
    text: String,
    x: f64,
    y: f64,
    /// Advance, points.
    w: f64,
    size: f64,
    style: Style,
}

/// The formatting a glyph carries over.
#[derive(Clone, Debug, Default, PartialEq)]
struct Style {
    bold: bool,
    italic: bool,
    font: Option<String>,
    color: Option<[u8; 3]>,
}

/// A picture as placed on the page.
#[derive(Clone, Debug)]
struct Pic {
    png: Arc<Vec<u8>>,
    w: f64,
    h: f64,
}

/// What a page draws, in drawing order.
#[derive(Clone, Debug)]
enum Item {
    Glyph(Gl),
    Picture(Pic),
}

struct PageContent {
    items: Vec<Item>,
    /// Page size, points.
    width: f64,
    height: f64,
}

/// Font facts per font, worked out once.
#[derive(Clone, Default)]
struct FontFacts {
    bold: bool,
    italic: bool,
    family: Option<String>,
}

struct Collector {
    items: Vec<Item>,
    glyphs: usize,
    pictures: usize,
    fonts: HashMap<u128, FontFacts>,
    /// The page's origin offset (crop box's lower left), subtracted from positions.
    origin: Vec2,
}

fn read_pages(pdf: &Pdf) -> Vec<PageContent> {
    let cache = InterpreterCache::new();
    let mut pages = Vec::new();
    let mut glyphs = 0usize;
    let mut pictures = 0usize;
    let mut fonts = HashMap::new();
    for page in pdf.pages().iter().take(MAX_PAGES) {
        let c = page.intersected_crop_box();
        let crop = kurbo::Rect::new(c.x0, c.y0, c.x1, c.y1);
        let (width, height) = (crop.width().abs(), crop.height().abs());
        if !(width.is_finite() && height.is_finite()) || width < 1.0 || height < 1.0 {
            continue;
        }
        let mut ctx = Context::new(Affine::IDENTITY, crop, &cache, pdf.xref(), InterpreterSettings::default());
        let mut c = Collector {
            items: Vec::new(),
            glyphs,
            pictures,
            fonts: std::mem::take(&mut fonts),
            origin: Vec2::new(crop.x0.min(crop.x1), crop.y0.min(crop.y1)),
        };
        interpret_page(page, &mut ctx, &mut c);
        glyphs = c.glyphs;
        pictures = c.pictures;
        fonts = c.fonts;
        pages.push(PageContent { items: c.items, width, height });
        if glyphs >= MAX_GLYPHS {
            break;
        }
    }
    pages
}

impl Collector {
    fn font_facts(&mut self, g: &hayro_interpret::font::OutlineGlyph) -> FontFacts {
        let key = g.font_cache_key();
        if let Some(f) = self.fonts.get(&key) {
            return f.clone();
        }
        let facts = match g.font_data() {
            Some(d) => {
                let name = d.postscript_name.clone().unwrap_or_default();
                let lower = name.to_ascii_lowercase();
                FontFacts {
                    bold: d.weight.is_some_and(|w| w >= 600) || ["bold", "black", "heavy", "semibold", "demi"].iter().any(|k| lower.contains(k)),
                    italic: d.is_italic || lower.contains("italic") || lower.contains("oblique"),
                    family: family_name(&name),
                }
            }
            None => FontFacts::default(),
        };
        self.fonts.insert(key, facts.clone());
        facts
    }
}

impl<'a> Device<'a> for Collector {
    fn set_soft_mask(&mut self, _: Option<SoftMask<'a>>) {}
    fn set_blend_mode(&mut self, _: BlendMode) {}
    fn draw_path(&mut self, _: &kurbo::BezPath, _: Affine, _: &Paint<'a>, _: &PathDrawMode) {}
    fn push_clip_path(&mut self, _: &ClipPath) {}
    fn push_transparency_group(&mut self, _: f32, _: Option<SoftMask<'a>>, _: BlendMode) {}
    fn pop_clip_path(&mut self) {}
    fn pop_transparency_group(&mut self) {}

    fn draw_glyph(&mut self, glyph: &Glyph<'a>, transform: Affine, glyph_transform: Affine, paint: &Paint<'a>, mode: &GlyphDrawMode) {
        // Synthetic bold fills and then strokes each glyph: one character, two draws. Invisible
        // text (an OCR layer over a scan) has no stroke or fill, and is the text we want.
        if self.glyphs >= MAX_GLYPHS || matches!(mode, GlyphDrawMode::Stroke(_)) {
            return;
        }
        let text = match glyph.as_unicode() {
            Some(BfString::Char(c)) => c.to_string(),
            Some(BfString::String(s)) => s,
            None => return,
        };
        let text: String = text.chars().filter(|c| !c.is_control() || *c == '\t').map(|c| if c == '\t' || c == '\u{a0}' { ' ' } else { c }).collect();
        if text.is_empty() {
            return;
        }
        let full = transform * glyph_transform;
        let origin = full * Point::ORIGIN;
        // The glyph space is 1000 units to the em (the interpreter scales by 1/1000).
        let up = full * Point::new(0.0, 1000.0) - origin;
        let size = up.length();
        let advance = match glyph {
            Glyph::Outline(o) => o.advance_width().map(f64::from),
            Glyph::Type3(_) => None,
        }
        .filter(|a| a.is_finite() && *a > 0.0)
        .unwrap_or(500.0);
        let w = (full * Point::new(advance, 0.0) - origin).length();
        if !(size.is_finite() && w.is_finite() && origin.x.is_finite() && origin.y.is_finite()) || !(1.0..=1000.0).contains(&size) {
            return;
        }
        // Text not running left to right (rotated, vertical) is read as it comes.
        let facts = match glyph {
            Glyph::Outline(o) => self.font_facts(o),
            Glyph::Type3(_) => FontFacts::default(),
        };
        let color = match paint {
            Paint::Color(c) => {
                let [r, g, b, _] = c.to_rgba().to_rgba8();
                // Black and near-black are the default colour.
                (u16::from(r) + u16::from(g) + u16::from(b) > 60).then_some([r, g, b])
            }
            Paint::Pattern(_) => None,
        };
        self.glyphs += 1;
        let p = origin - self.origin;
        self.items.push(Item::Glyph(Gl {
            text,
            x: p.x,
            y: p.y,
            w,
            size,
            style: Style { bold: facts.bold, italic: facts.italic, font: facts.family, color },
        }));
    }

    fn draw_image(&mut self, image: Image<'a, '_>, transform: Affine) {
        if self.pictures >= MAX_PICTURES {
            return;
        }
        let Image::Raster(raster) = image else { return };
        // `transform` places the image's pixels (width × height) on the page.
        let (pw, ph) = (f64::from(raster.width()), f64::from(raster.height()));
        let corners = [Point::new(0.0, 0.0), Point::new(pw, 0.0), Point::new(0.0, ph), Point::new(pw, ph)].map(|p| transform * p - self.origin);
        let (mut x0, mut x1, mut y0, mut y1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
        for c in corners {
            x0 = x0.min(c.x);
            x1 = x1.max(c.x);
            y0 = y0.min(c.y);
            y1 = y1.max(c.y);
        }
        let (w, h) = (x1 - x0, y1 - y0);
        if !(w.is_finite() && h.is_finite()) || w < MIN_PICTURE_PT || h < MIN_PICTURE_PT {
            return;
        }
        if u64::from(raster.width()) * u64::from(raster.height()) > MAX_PICTURE_PIXELS {
            return;
        }
        let mut png = None;
        raster.with_rgba(|data, alpha| png = encode_png(data, alpha), None);
        let Some(png) = png else { return };
        self.pictures += 1;
        self.items.push(Item::Picture(Pic { png: Arc::new(png), w, h }));
    }
}

/// A decoded picture as PNG (RGB, or RGBA when it has transparency).
fn encode_png(data: ImageData, alpha: Option<hayro_interpret::LumaData>) -> Option<Vec<u8>> {
    let (w, h) = (data.width(), data.height());
    let n = (w as usize).checked_mul(h as usize)?;
    let rgb: Vec<u8> = match data {
        ImageData::Rgb(d) => d.data,
        ImageData::Luma(d) => d.data.iter().flat_map(|&v| [v, v, v]).collect(),
    };
    if rgb.len() < n * 3 {
        return None;
    }
    let mut out = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut out);
    use image::ImageEncoder as _;
    match alpha.filter(|a| a.width == w && a.height == h && a.data.len() >= n && a.data.iter().any(|&v| v < 255)) {
        Some(a) => {
            let rgba: Vec<u8> = rgb.chunks_exact(3).zip(a.data.iter()).take(n).flat_map(|(c, &al)| [c[0], c[1], c[2], al]).collect();
            encoder.write_image(&rgba, w, h, image::ExtendedColorType::Rgba8).ok()?;
        }
        None => encoder.write_image(rgb.get(..n * 3)?, w, h, image::ExtendedColorType::Rgb8).ok()?,
    }
    Some(out)
}

/// A family name from a PostScript font name: `ABCDEF+TimesNewRomanPS-BoldItalicMT` → Times New
/// Roman. The standard 14 fonts get the families documents use for them.
fn family_name(ps: &str) -> Option<String> {
    let name = ps.split_once('+').filter(|(tag, _)| tag.len() == 6 && tag.chars().all(|c| c.is_ascii_uppercase())).map_or(ps, |(_, n)| n);
    let base = name.split(['-', ',']).next().unwrap_or(name);
    let base = ["PSMT", "MT", "PS", "Std", "Pro", "LT"].iter().fold(base.to_string(), |b, s| b.strip_suffix(s).map(str::to_string).unwrap_or(b));
    let base = ["Bold", "Italic", "Regular", "Oblique", "Medium", "Light", "Semibold", "SemiBold", "Black"]
        .iter()
        .fold(base, |b, s| if b.len() > s.len() + 2 { b.strip_suffix(s).map(str::to_string).unwrap_or(b) } else { b });
    let standard = match base.as_str() {
        "Times" => Some("Times New Roman"),
        "Helvetica" | "Arial" => Some("Arial"),
        "Courier" => Some("Courier New"),
        "Symbol" => Some("Symbol"),
        _ => None,
    };
    if let Some(s) = standard {
        return Some(s.to_string());
    }
    if base.is_empty() || !base.chars().all(|c| c.is_ascii_alphanumeric() || c == ' ' || c == '_') {
        return None;
    }
    // "TimesNewRoman" → "Times New Roman", keeping runs of capitals ("PTSerif" stays).
    let mut out = String::new();
    let chars: Vec<char> = base.replace('_', " ").chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        let prev = i.checked_sub(1).and_then(|j| chars.get(j)).copied();
        if c.is_ascii_uppercase() && prev.is_some_and(|p| p.is_ascii_lowercase()) {
            out.push(' ');
        }
        out.push(c);
    }
    Some(out.split_whitespace().collect::<Vec<_>>().join(" "))
}

// ---------------------------------------------------------------------------------------------
// Lines and paragraphs

/// A run of text in one style within a line.
#[derive(Clone, Debug)]
struct Span {
    text: String,
    size: f64,
    style: Style,
}

#[derive(Clone, Debug)]
struct Line {
    spans: Vec<Span>,
    /// Baseline, points from the bottom.
    y: f64,
    left: f64,
    right: f64,
    /// Largest glyph size on the line.
    size: f64,
    /// Where each cell starts: (x, index of its first span). Text separated by a gap much wider
    /// than a space (table columns, tab stops) starts a new cell.
    cells: Vec<(f64, usize)>,
}

impl Line {
    fn text(&self) -> String {
        self.spans.iter().map(|s| s.text.as_str()).collect()
    }

    /// The spans of cell `i`.
    fn cell_spans(&self, i: usize) -> &[Span] {
        let start = self.cells.get(i).map_or(self.spans.len(), |c| c.1);
        let end = self.cells.get(i + 1).map_or(self.spans.len(), |c| c.1);
        self.spans.get(start..end.max(start)).unwrap_or(&[])
    }
}

/// A page's lines, pictures and tables, in drawing order.
enum Piece {
    Line(Line),
    Picture(Pic),
    /// Rows of a table: lines whose cells line up in columns.
    Table(Vec<Line>),
}

/// Group a page's glyphs into lines (one baseline each), with spaces where the gaps are.
fn lines_of(items: &[Item]) -> Vec<Piece> {
    let mut out: Vec<Piece> = Vec::new();
    let mut cur: Option<Line> = None;
    let mut last_end = 0.0f64;
    let flush = |cur: &mut Option<Line>, out: &mut Vec<Piece>| {
        if let Some(mut l) = cur.take() {
            while l.spans.last().is_some_and(|s| s.text.trim().is_empty()) {
                l.spans.pop();
            }
            if let Some(s) = l.spans.last_mut() {
                let t = s.text.trim_end().len();
                s.text.truncate(t);
            }
            if !l.text().trim().is_empty() {
                out.push(Piece::Line(l));
            }
        }
    };
    for item in items {
        let g = match item {
            Item::Glyph(g) => g,
            Item::Picture(p) => {
                flush(&mut cur, &mut out);
                out.push(Piece::Picture(p.clone()));
                continue;
            }
        };
        let same_line = cur.as_ref().is_some_and(|l| {
            let tol = 0.45 * l.size.min(g.size);
            // Same baseline (superscripts and subscripts sit within it), moving right.
            (g.y - l.y).abs() <= tol.max(0.5 * l.size) && g.x >= last_end - 0.6 * g.size && g.x <= last_end + 12.0 * g.size
        });
        if !same_line {
            flush(&mut cur, &mut out);
            cur = Some(Line { spans: Vec::new(), y: g.y, left: g.x, right: g.x + g.w, size: g.size, cells: vec![(g.x, 0)] });
            last_end = g.x;
        }
        let Some(line) = cur.as_mut() else { continue };
        let gap = g.x - last_end;
        // Much wider than a space: a new cell (column, tab stop).
        let new_cell = !line.spans.is_empty() && gap > (1.2 * g.size).max(6.0) && !g.text.trim().is_empty();
        let needs_space =
            gap > 0.18 * g.size && !line.spans.last().is_some_and(|s| s.text.ends_with(' ')) && !g.text.starts_with(' ') && !line.spans.is_empty();
        if needs_space && let Some(s) = line.spans.last_mut() {
            s.text.push(' ');
        }
        // A glyph that is only a space starts no line and adds no double spaces.
        if g.text.trim().is_empty() && (line.spans.is_empty() || line.spans.last().is_some_and(|s| s.text.ends_with(' '))) {
            last_end = last_end.max(g.x + g.w);
            continue;
        }
        let size = (g.size * 2.0).round() / 2.0;
        if new_cell {
            line.cells.push((g.x, line.spans.len()));
            line.spans.push(Span { text: g.text.clone(), size, style: g.style.clone() });
        } else {
            match line.spans.last_mut() {
                Some(s) if s.style == g.style && (s.size - size).abs() < 0.6 => s.text.push_str(&g.text),
                _ => line.spans.push(Span { text: g.text.clone(), size, style: g.style.clone() }),
            }
        }
        line.size = line.size.max(g.size);
        line.left = line.left.min(g.x);
        line.right = line.right.max(g.x + g.w);
        // Baseline of the main text: superscripts don't move it.
        if g.size >= line.size - 0.01 {
            line.y = g.y;
        }
        last_end = last_end.max(g.x + g.w);
    }
    flush(&mut cur, &mut out);
    out
}

/// Text with the digits taken out, for spotting running headers and footers ("Page 3 of 9").
fn header_key(s: &str) -> String {
    s.chars().filter(|c| !c.is_ascii_digit()).collect::<String>().split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

/// Is `s` a page number ("7", "- 7 -", "Page 7", "7 of 12", "vii")?
fn is_page_number(s: &str) -> bool {
    let t = s.trim().trim_matches(|c: char| c == '-' || c == '–' || c == '—' || c.is_whitespace()).to_lowercase();
    let t = t.strip_prefix("page").map_or(t.as_str(), str::trim);
    let digits = |x: &str| !x.is_empty() && x.len() <= 5 && x.chars().all(|c| c.is_ascii_digit());
    let roman = |x: &str| !x.is_empty() && x.len() <= 6 && x.chars().all(|c| "ivxlc".contains(c));
    digits(t) || roman(t) || t.split_once(" of ").is_some_and(|(a, b)| digits(a.trim()) && digits(b.trim()))
}

/// A paragraph being built.
struct ParaAcc {
    lines: Vec<Line>,
    page_break: bool,
}

fn build_flow(pages: &[PageContent]) -> Flow {
    let mut page_pieces: Vec<Vec<Piece>> = pages.iter().map(|p| lines_of(&p.items)).collect();

    // Running headers and footers: lines in the top or bottom tenth of the page that are page
    // numbers, or whose text (digits aside) recurs there on at least half the pages.
    let mut margin_count: HashMap<String, usize> = HashMap::new();
    for (pieces, page) in page_pieces.iter().zip(pages) {
        let mut seen = std::collections::HashSet::new();
        for piece in pieces {
            if let Piece::Line(l) = piece
                && in_margin(l, page.height)
                && seen.insert(header_key(&l.text()))
            {
                *margin_count.entry(header_key(&l.text())).or_default() += 1;
            }
        }
    }
    let repeated = |l: &Line| {
        let n = margin_count.get(&header_key(&l.text())).copied().unwrap_or(0);
        !header_key(&l.text()).is_empty() && ((pages.len() >= 3 && n * 2 >= pages.len()) || (pages.len() == 2 && n == 2))
    };
    for (pieces, page) in page_pieces.iter_mut().zip(pages) {
        pieces.retain(|piece| match piece {
            Piece::Line(l) => !(in_margin(l, page.height) && (is_page_number(&l.text()) || repeated(l))),
            Piece::Picture(_) | Piece::Table(_) => true,
        });
        *pieces = find_tables(std::mem::take(pieces));
    }

    // The body text size: the size most characters are set in.
    let mut by_size: HashMap<i64, usize> = HashMap::new();
    for pieces in &page_pieces {
        for piece in pieces {
            let lines: &[Line] = match piece {
                Piece::Line(l) => std::slice::from_ref(l),
                Piece::Table(rows) => rows,
                Piece::Picture(_) => &[],
            };
            for l in lines {
                for s in &l.spans {
                    *by_size.entry((s.size * 2.0).round() as i64).or_default() += s.text.chars().count();
                }
            }
        }
    }
    let body = by_size.iter().max_by_key(|(k, n)| (**n, -**k)).map_or(11.0, |(k, _)| *k as f64 / 2.0);

    // Paragraphs.
    let mut blocks: Vec<FBlock> = Vec::new();
    let mut heading_sizes: Vec<f64> = Vec::new();
    let mut paras: Vec<Out> = Vec::new();
    for (pi, (pieces, page)) in page_pieces.iter().zip(pages).enumerate() {
        let text_left = pieces.iter().filter_map(|p| if let Piece::Line(l) = p { Some(l.left) } else { None }).fold(f64::MAX, f64::min);
        let text_right = pieces.iter().filter_map(|p| if let Piece::Line(l) = p { Some(l.right) } else { None }).fold(f64::MIN, f64::max);
        let full = (text_right - text_left).max(1.0);
        let mut first_on_page = true;
        let mut prev: Option<Line> = None;
        for piece in pieces {
            match piece {
                Piece::Picture(pic) => {
                    // A picture covering the page (a scan) is kept at the width of the text.
                    let mut pic = pic.clone();
                    let max_w = (page.width - 144.0).max(144.0);
                    if pic.w > max_w {
                        let k = max_w / pic.w;
                        pic.w *= k;
                        pic.h *= k;
                    }
                    paras.push(Out::Picture(pic, first_on_page && pi > 0));
                    first_on_page = false;
                    prev = None;
                }
                Piece::Table(rows) => {
                    paras.push(Out::Table(rows.clone(), first_on_page && pi > 0));
                    first_on_page = false;
                    prev = None;
                }
                Piece::Line(line) => {
                    let continues = prev.as_ref().is_some_and(|p| continues_paragraph(p, line, text_left, full, body))
                        || (first_on_page && pi > 0 && continues_across_pages(paras.last(), line, body));
                    match paras.last_mut() {
                        Some(Out::Para(acc)) if continues => acc.lines.push(line.clone()),
                        _ => paras.push(Out::Para(ParaAcc { lines: vec![line.clone()], page_break: first_on_page && pi > 0 })),
                    }
                    first_on_page = false;
                    prev = Some(line.clone());
                }
            }
        }
    }
    // Heading levels: the distinct sizes of short paragraphs set clearly larger than the body.
    for acc in paras.iter().filter_map(|o| if let Out::Para(a) = o { Some(a) } else { None }) {
        if let Some(size) = heading_size(acc, body)
            && !heading_sizes.iter().any(|s| (s - size).abs() < 0.75)
        {
            heading_sizes.push(size);
        }
    }
    heading_sizes.sort_by(|a, b| b.total_cmp(a));
    for out in paras {
        let acc = match out {
            Out::Para(acc) => acc,
            Out::Picture(pic, page_break) => {
                let mut p = Para { page_break, ..Default::default() };
                p.inlines.push(Inline::Image(Img { data: pic.png.clone(), ext: "png".into(), w: pic.w as f32, h: pic.h as f32, alt: String::new() }));
                blocks.push(FBlock::Para(p));
                continue;
            }
            Out::Table(rows, page_break) => {
                if page_break {
                    blocks.push(FBlock::Para(Para { page_break: true, ..Default::default() }));
                }
                blocks.push(FBlock::Table(table_of(&rows)));
                continue;
            }
        };
        let kind = match heading_size(&acc, body) {
            Some(size) => Kind::Heading(heading_sizes.iter().position(|s| (s - size).abs() < 0.75).map_or(3, |i| (i + 1).min(3) as u8)),
            None => Kind::Normal,
        };
        let mut p = Para::new(kind);
        p.page_break = acc.page_break;
        p.align = alignment(&acc.lines);
        // "• text" is a bulleted list item: the bullet becomes the list's own.
        let mut lines = acc.lines.clone();
        if kind == Kind::Normal && strip_bullet(&mut lines) {
            p.list = Some(ListInfo { ordered: false, level: 0 });
        }
        let acc = ParaAcc { lines, page_break: acc.page_break };
        let n = acc.lines.len();
        for (i, line) in acc.lines.iter().enumerate() {
            let spans = &line.spans;
            for (j, span) in spans.iter().enumerate() {
                let mut text = span.text.clone();
                let last_of_line = j + 1 == spans.len();
                if last_of_line && i + 1 < n {
                    let next_starts_lower = acc.lines.get(i + 1).and_then(|l| l.text().trim_start().chars().next()).is_some_and(char::is_lowercase);
                    // "exam-" + "ple" → "example"; a hyphen before a capital (or a dash) stays.
                    // PDFs hyphenate with "-", the Unicode hyphen (U+2010) or a soft hyphen.
                    if next_starts_lower && text.ends_with(['-', '\u{2010}', '\u{ad}']) && text.chars().rev().nth(1).is_some_and(char::is_alphabetic)
                    {
                        text.pop();
                    } else if !text.ends_with(['-', '\u{2010}', '/']) {
                        text.push(' ');
                    }
                }
                let f = span_fmt(span, kind);
                p.push_text(&text, &f);
            }
        }
        p.trim();
        if !p.is_empty() {
            blocks.push(FBlock::Para(p));
        }
    }
    Flow { blocks, meta: Default::default() }
}

/// What the page sequence becomes before formatting: paragraphs, pictures and tables.
enum Out {
    Para(ParaAcc),
    /// A picture, and whether it starts a page.
    Picture(Pic, bool),
    Table(Vec<Line>, bool),
}

fn span_fmt(span: &Span, kind: Kind) -> Fmt {
    Fmt {
        bold: span.style.bold && kind == Kind::Normal,
        italic: span.style.italic,
        font: span.style.font.clone(),
        color: span.style.color.map(|[r, g, b]| wordcraft_doc::Rgb(r, g, b)),
        // Headings take their size from the heading style.
        size: (kind == Kind::Normal).then_some(span.size as f32),
        ..Default::default()
    }
}

/// Take a leading bullet ("•", "◦", "▪", "–" followed by a space) off the first line; whether
/// there was one.
fn strip_bullet(lines: &mut [Line]) -> bool {
    let Some(first) = lines.first_mut() else { return false };
    // The bullet is often a cell of its own (a tab after it).
    let Some(span) = first.spans.first_mut() else { return false };
    let t = span.text.trim_start();
    let Some(c) = t.chars().next().filter(|c| matches!(c, '•' | '◦' | '▪' | '‣' | '·' | '●' | '○' | '■' | '–')) else {
        return false;
    };
    let rest = &t[c.len_utf8()..];
    if !(rest.is_empty() || rest.starts_with(' ')) {
        return false;
    }
    span.text = rest.trim_start().to_string();
    if span.text.is_empty() {
        first.spans.remove(0);
        for cell in first.cells.iter_mut() {
            cell.1 = cell.1.saturating_sub(1);
        }
    }
    !first.spans.is_empty()
}

/// Runs of at least two lines whose cells line up in the same columns become tables.
fn find_tables(pieces: Vec<Piece>) -> Vec<Piece> {
    let mut out: Vec<Piece> = Vec::new();
    let mut run: Vec<Line> = Vec::new();
    let flush = |run: &mut Vec<Line>, out: &mut Vec<Piece>| {
        if run.len() >= 2 {
            out.push(Piece::Table(std::mem::take(run)));
        } else {
            out.extend(run.drain(..).map(Piece::Line));
        }
    };
    for piece in pieces {
        match piece {
            Piece::Line(l) if l.cells.len() >= 2 && l.cells.len() <= 20 => {
                let fits = run.first().is_some_and(|first| {
                    let prev = run.last().unwrap_or(first);
                    first.cells.len() == l.cells.len()
                        && first.cells.iter().zip(&l.cells).all(|(a, b)| (a.0 - b.0).abs() <= 1.5 * l.size.max(first.size))
                        && prev.y - l.y > 0.0
                        && prev.y - l.y < 3.0 * l.size.max(prev.size)
                });
                if !fits {
                    flush(&mut run, &mut out);
                }
                run.push(l);
            }
            other => {
                flush(&mut run, &mut out);
                out.push(other);
            }
        }
    }
    flush(&mut run, &mut out);
    out
}

/// A table from rows of lined-up cells: column widths from where the columns start, the first
/// row a header row when it is all bold.
fn table_of(rows: &[Line]) -> FTable {
    let Some(first) = rows.first() else { return FTable::default() };
    let cols = first.cells.len();
    let right = rows.iter().map(|r| r.right).fold(f64::MIN, f64::max);
    let mut widths: Vec<f32> = Vec::with_capacity(cols);
    for i in 0..cols {
        let start = first.cells.get(i).map_or(0.0, |c| c.0);
        let end = first.cells.get(i + 1).map_or(right + 6.0, |c| c.0);
        widths.push(((end - start).max(18.0)) as f32);
    }
    let header = first.spans.iter().all(|s| s.style.bold);
    let rows = rows
        .iter()
        .enumerate()
        .map(|(ri, line)| {
            (0..cols)
                .map(|ci| {
                    let mut p = Para::new(Kind::Normal);
                    for span in line.cell_spans(ci) {
                        p.push_text(&span.text, &span_fmt(span, Kind::Normal));
                    }
                    p.trim();
                    Cell { blocks: vec![FBlock::Para(p)], header: header && ri == 0, ..Default::default() }
                })
                .collect()
        })
        .collect();
    FTable { rows, widths, borderless: false }
}

fn in_margin(l: &Line, page_h: f64) -> bool {
    l.y > page_h * 0.9 || l.y < page_h * 0.1
}

/// The size a paragraph would be a heading at: short, and set clearly larger than the body.
fn heading_size(acc: &ParaAcc, body: f64) -> Option<f64> {
    let size = acc.lines.iter().map(|l| l.size).fold(0.0, f64::max);
    let chars: usize = acc.lines.iter().map(|l| l.text().chars().count()).sum();
    (size >= body * 1.15 + 0.5 && chars <= 200 && acc.lines.len() <= 3).then_some((size * 2.0).round() / 2.0)
}

/// Does `next` carry on the paragraph `prev` is the last line of?
fn continues_paragraph(prev: &Line, next: &Line, text_left: f64, full: f64, body: f64) -> bool {
    let gap = prev.y - next.y;
    let size = prev.size.max(next.size);
    // A different size (a heading and its text), a line above the previous one (a new column),
    // or more space than line spacing ends the paragraph.
    if (prev.size - next.size).abs() > 1.0 || gap <= 0.0 || gap > size * 1.75 {
        return false;
    }
    // A short last line: the paragraph ended there (unless it's all short lines, as in a list
    // of names, which stay separate anyway).
    let prev_short = prev.right < text_left + full * 0.75;
    if prev_short {
        return false;
    }
    // An indented first line starts a new paragraph.
    let indented = next.left > prev.left + body * 1.2;
    !indented && !starts_list_item(&next.text())
}

/// Does the first line of a page carry on the last paragraph of the page before?
fn continues_across_pages(last: Option<&Out>, line: &Line, body: f64) -> bool {
    let Some(Out::Para(acc)) = last else { return false };
    let Some(prev) = acc.lines.last() else { return false };
    let prev_text = prev.text();
    let ends_sentence = prev_text.trim_end().ends_with(['.', '!', '?', ':', '"', '”', ')']);
    let starts_lower = line.text().trim_start().chars().next().is_some_and(char::is_lowercase);
    (prev.size - line.size).abs() <= 1.0 && (prev.size - body).abs() <= 1.0 && !ends_sentence && starts_lower
}

/// Does `s` start like a list item ("•", "1.", "a)")?
fn starts_list_item(s: &str) -> bool {
    let t = s.trim_start();
    if t.starts_with(['•', '◦', '▪', '‣', '–', '-', '*', '·']) {
        return true;
    }
    let lead: String = t.chars().take_while(|c| c.is_ascii_alphanumeric()).collect();
    !lead.is_empty() && lead.len() <= 3 && t[lead.len()..].starts_with(['.', ')']) && (lead.chars().all(|c| c.is_ascii_digit()) || lead.len() == 1)
}

/// Centred when every line's middle sits at the same place and the lines don't fill the width.
fn alignment(lines: &[Line]) -> Option<Align> {
    let first = lines.first()?;
    let mid = (first.left + first.right) / 2.0;
    let centred = lines.len() <= 4 && lines.iter().all(|l| ((l.left + l.right) / 2.0 - mid).abs() < 3.0) && first.left > 100.0;
    centred.then_some(Align::Center)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn family_names_from_postscript_names() {
        assert_eq!(family_name("ABCDEF+TimesNewRomanPS-BoldItalicMT").as_deref(), Some("Times New Roman"));
        assert_eq!(family_name("Times-Roman").as_deref(), Some("Times New Roman"));
        assert_eq!(family_name("Helvetica-Bold").as_deref(), Some("Arial"));
        assert_eq!(family_name("Calibri,Bold").as_deref(), Some("Calibri"));
        assert_eq!(family_name("XYZABC+SourceSans3-Regular").as_deref(), Some("Source Sans3"));
        assert_eq!(family_name(""), None);
    }

    #[test]
    fn page_numbers_and_running_headers() {
        for s in ["7", "- 7 -", "Page 7", "page 3 of 12", "vii", "— 12 —"] {
            assert!(is_page_number(s), "{s}");
        }
        for s in ["7 apples", "Introduction", "Chapter 1"] {
            assert!(!is_page_number(s), "{s}");
        }
        assert_eq!(header_key("Annual Report 2024 — Page 3"), "annual report — page");
    }

    #[test]
    fn list_items() {
        assert!(starts_list_item("• Apples"));
        assert!(starts_list_item("1. First"));
        assert!(starts_list_item("a) Option"));
        assert!(!starts_list_item("The 1990s were"));
    }

    #[test]
    fn junk_is_an_error_not_a_panic() {
        assert!(import(b"").is_err());
        assert!(import(b"%PDF-1.7\n garbage").is_err() || import(b"%PDF-1.7\n garbage").is_ok());
        assert!(!is_pdf(b"hello"));
        assert!(is_pdf(b"%PDF-1.4\n"));
    }
}
