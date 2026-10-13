<p align="center">
  <img alt="Wordpadx app icon" src="assets/app-icon/wordcraft.svg" width="96">
</p>

<h1 align="center">Wordpadx</h1>

<p align="center">
  <b>A fast, open-source word processor in pure Rust for Android tablets, iPad, the web and the desktop.</b>
</p>

<p align="center">
  The Word workflow you already know (ribbon, styles, tables, track changes, references and mail
  merge) in one Rust codebase. It reads and writes .docx, installs on tablets as an app or a
  Progressive Web App, and runs natively on macOS, Windows, Linux and BSD.<br>
  <i>Wordpadx is a fork of <a href="https://github.com/storytold/wordcraft">WordCraft</a> with a focus on tablets.</i>
</p>

<p align="center">
  <img alt="Written in Rust" src="https://img.shields.io/badge/written%20in-Rust-2b47b5?style=flat-square&logo=rust&logoColor=white">
  <img alt="Runs on Android tablet, iPad, web, macOS, Windows, Linux and BSD" src="https://img.shields.io/badge/runs%20on-Android%20%C2%B7%20iPad%20%C2%B7%20Web%20%C2%B7%20Desktop-3b5bdb?style=flat-square">
  <img alt="License: MIT OR Apache-2.0" src="https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-2b47b5?style=flat-square">
  <img alt="Agent-drivable over MCP" src="https://img.shields.io/badge/agents-MCP%20%C2%B7%20CLI-3b5bdb?style=flat-square">
</p>

<p align="center">
  <a href="https://github.com/abahvictor360-sketch/Wordpadx/releases"><b>Download</b></a> ·
  <a href="https://abahvictor360-sketch.github.io/Wordpadx/">Website</a> ·
  <a href="TABLET_GUIDE.md">Tablet guide</a> ·
  <a href="https://github.com/abahvictor360-sketch/Wordpadx/issues">Report a problem</a>
</p>

<br>

<p align="center">
  <img src="docs/images/hero.png" alt="The Home tab of the ribbon open over a two-page document titled The Open Studio Handbook. The Navigation pane on the left lists the document's headings, the Styles gallery shows live previews of Normal, Heading 1, Title and Subtitle, and a word in the first paragraph is selected." width="100%">
  <br><sub><b>The Open Studio Handbook</b>, the built-in sample: the ribbon, live Styles gallery, rulers and the Navigation pane.</sub>
</p>

<p align="center">
  <a href="#get-wordpadx">Get Wordpadx</a> ·
  <a href="#a-tour">A tour</a> ·
  <a href="#why-wordpadx">Why Wordpadx</a> ·
  <a href="#what-works-today">What works today</a> ·
  <a href="#build-from-source">Build from source</a> ·
  <a href="#for-agents-cli-and-mcp">For agents</a> ·
  <a href="#architecture">Architecture</a> ·
  <a href="#status">Status</a> ·
  <a href="#license-and-credits">License and credits</a>
</p>

## Get Wordpadx

| Platform | How | Notes |
|---|---|---|
| **Android tablet** | `Wordpadx-Android-Tablet.apk` (or `app-debug.apk` in older releases) from [Releases](https://github.com/abahvictor360-sketch/Wordpadx/releases) | Debug-signed build: allow installs from unknown sources. Opens .docx, .odt and .txt with the system file picker; exports go to Downloads. |
| **iPad** | `Wordpadx-iPad.zip` from [Releases](https://github.com/abahvictor360-sketch/Wordpadx/releases) | Built without code signing, so it has to be signed (Xcode, or a sideloading tool) before it installs. Building it yourself in Xcode is the simplest route; see below. |
| **Any tablet browser** | Install the web app ([PWA](TABLET_GUIDE.md#1-instant-pwa-installation-zero-build-required)): Safari → Share → *Add to Home Screen*, or Chrome → ⋮ → *Install app* | Needs WebGPU or WebGL2. Runs full screen and keeps working offline once loaded. |
| **macOS, Windows, Linux, BSD** | [Build from source](#build-from-source), or install upstream [WordCraft releases](https://github.com/storytold/wordcraft/releases/latest) | The desktop app is the same engine and interface. |

Every push to `main` builds the web app, the Android APK and the iPad app
([`build-mobile.yml`](.github/workflows/build-mobile.yml)); a successful build is published as a
`v0.5.0-build.N` prerelease ([`publish-mobile-build.yml`](.github/workflows/publish-mobile-build.yml)).
Step-by-step tablet instructions, including Split View, Stage Manager, Samsung DeX and stylus
support: **[TABLET_GUIDE.md](TABLET_GUIDE.md)**.

## A tour

Every screenshot below is the app itself, rendered offscreen by its own UI test harness
(`cargo run -p wordcraft-ui-egui --example ui_shot`).

<table>
<tr>
<td width="50%" valign="top"><img src="docs/images/review.png" alt="The Review tab with Track Changes on: the word forty is inserted in magenta and thirty struck through; three commented phrases are shaded and joined by dashed leader lines to comment balloons in a grey markup area to the right of the page" width="100%"><p align="center"><sub><b>Review.</b> Track changes, comment balloons in the margin, accept and reject, spelling and grammar as you type.</sub></p></td>
<td width="50%" valign="top"><img src="docs/images/references.png" alt="The References tab with two pages side by side: a table of contents with dotted leaders and page numbers on page one, and a styled table, numbered list and hyperlink on page two" width="100%"><p align="center"><sub><b>References.</b> Tables of contents, footnotes, citations in APA, MLA, Chicago or IEEE, index and captions.</sub></p></td>
</tr>
<tr>
<td width="50%" valign="top"><img src="docs/images/design.png" alt="The Design tab showing style-set previews; the document is set in a serif theme with plum headings underlined by thin rules and a pale diagonal DRAFT watermark behind the text" width="100%"><p align="center"><sub><b>Design.</b> Themes, style sets, paragraph spacing, watermarks, page colour and borders.</sub></p></td>
<td width="50%" valign="top"><img src="docs/images/dark.png" alt="Dark mode with the Insert tab open and formatting marks shown: pilcrows at paragraph ends and dots for spaces" width="100%"><p align="center"><sub><b>Dark mode</b> with formatting marks, and the Insert tab: tables, pictures, shapes, links, headers, footers, fields and symbols.</sub></p></td>
</tr>
<tr>
<td colspan="2"><img src="docs/images/layout.png" alt="The Layout tab at 80% zoom: the first paragraph opens with a three-line drop cap E, a dark blue circle sits beside the second section with its paragraph wrapping around it and hyphenating art-ist at the line end, every line is numbered in the left margin and a thin blue border surrounds the page" width="100%"><p align="center"><sub><b>Layout.</b> Drop caps, text wrapping around pictures and shapes, automatic hyphenation, line numbers and page borders.</sub></p></td>
</tr>
<tr>
<td colspan="2"><img src="docs/images/backstage.png" alt="The File tab start page with a Good morning greeting, thumbnails of the Blank document, Studio handbook, Letter, Résumé and Report templates, and a list of recent documents" width="100%"><p align="center"><sub><b>File.</b> Start from a template, open recent documents, edit properties, export to PDF and other formats.</sub></p></td>
</tr>
</table>

## Why Wordpadx

- **Made for tablets.** A native Android app, an iPad app and an installable web app, all from
  the same build. The ribbon's tab strip scrolls sideways on narrow screens, and the on-screen
  keyboard comes up when you tap the page.
- **Familiar.** Word's ribbon tabs, groups, shortcuts and behaviour: Enter continues a list,
  Tab demotes it, Ctrl/⌘+B bolds the word under the caret, the Styles gallery previews styles live,
  F4 repeats, F7 checks spelling, F8 extends the selection.
- **Your files.** Opens and saves .docx (OOXML), and also .odt, .rtf, .html, .md, .tex (LaTeX), .txt;
  opens Word 97-2003 .doc; exports PDF with real, selectable text, links and bookmarks.
- **Fast.** Paragraph layout is cached, so typing in a 188-page document re-lays it out in about
  1.4 ms; pages render on demand.
- **One codebase.** Pure Rust with native [egui](https://github.com/emilk/egui) on the GPU, compiled
  to WebAssembly for the web and tablets. No Electron, no Tauri.
- **Built for agents.** Every action is a command with an id. The same 600+ commands drive the
  ribbon, keyboard shortcuts, the command search, a command-line tool, a JSON control channel and
  an MCP server.
- **Private.** Spelling, grammar and everything else work offline.
- **Open.** MIT OR Apache-2.0. Clean-room: built from public specifications and observation, with
  every asset original or openly licensed.

## What works today

| Area | Highlights |
|---|---|
| **Writing** | Fast typing with IME, smart quotes, AutoCorrect, list autoformat (`* `, `1. `), dashes; word, sentence, paragraph selection; drag-select; clipboard with formatting; undo/redo; find and replace with regex |
| **Formatting** | Fonts, sizes, bold/italic/underline styles, strike, sub/superscript, caps, highlight, colours, character spacing, Format Painter, Change Case, Clear Formatting |
| **Paragraphs** | Alignment, indents (draggable on the ruler), spacing, line spacing, tabs with leaders, borders, shading, keep with next, widow/orphan control |
| **Styles** | Built-in style set, live gallery, Styles pane, create/modify/update styles, style sets, themes |
| **Lists** | Bullets, numbering, multilevel, restart, set value, custom formats |
| **Tables** | Insert by grid, merge/split, styles with banded rows, borders, shading, header rows repeated across pages, rows that split across pages, sort, formulas, text ↔ table |
| **Pages** | Margins, orientation, size, columns, page/column/section breaks, headers and footers (first page, odd/even), page numbers, watermark, page borders, line numbers, vertical alignment, drop caps, automatic hyphenation |
| **Objects** | Pictures (resize, crop, recolour, brightness/contrast, transparency, background removal, picture styles, rotate), shapes, text boxes, floating position with text wrapping (square, top and bottom, behind or in front of text) |
| **References** | Table of contents, footnotes and endnotes, citations and bibliography (APA, MLA, Chicago, IEEE), captions, table of figures, cross-references, index, table of authorities |
| **Review** | Spelling and grammar with suggestions, thesaurus, word count, comments in margin balloons or a pane, track changes, accept/reject, compare documents, restrict editing, accessibility checker, document inspector |
| **Mailings** | Mail merge from CSV, merge fields, address block, greeting line, rules, preview, finish to a document; envelopes and labels |
| **View** | Print layout, web layout, draft, read mode, focus, zoom, one/multiple pages, page width, Navigation pane, rulers, gridlines, dark mode; interface in English, 简体中文, 繁體中文, 日本語, Português (Brasil), Español, Українська, Српски / Srpski, Eesti, Türkçe, Русский or Italiano (follows the system language by default) |
| **Files** | .docx read/write (opens in Word), Word 97-2003 .doc import, PDF export, .odt, .rtf, .html, .md, .tex, .txt import/export, page images |

The detailed picture, area by area, is in [ROADMAP.md](ROADMAP.md), the
[parity assessment](docs/target-app-parity.md) and the generated
[feature checklist](docs/parity-checklist.md).

## Build from source

You need a recent stable Rust toolchain (1.90 or newer).

```sh
git clone https://github.com/abahvictor360-sketch/Wordpadx
cd Wordpadx
cargo run --release -p wordcraft -- --sample        # the desktop app with the sample document
cargo run --release -p wordcraft -- report.docx     # open a document
```

The crates and binaries keep their upstream names (`wordcraft`, `wordcraft-cli`,
`wordcraft-web`), so merging upstream changes stays easy.

**Web app / PWA.** Needs [Trunk](https://trunkrs.dev) and the wasm target:

```sh
rustup target add wasm32-unknown-unknown
cd apps/wordcraft-web
trunk serve                                         # then open http://127.0.0.1:8771/?sample
trunk build --release                               # static site in dist/web, ready to host
```

**Android tablet.** `./packaging/android/build.sh` (Windows: `.\packaging\android\build.ps1`)
builds the web app with Trunk, copies it into the Android project and runs Gradle; `--release`
builds a release APK, `--skip-web` reuses an existing `dist/web`. Or open `mobile/android` in
Android Studio after copying `dist/web` into `mobile/android/app/src/main/assets/www`.

**iPad.** On a Mac with Xcode, `./packaging/ios/build.sh` builds the web app and the iPad app.
Or open `mobile/ios/Wordpadx.xcodeproj` and pick your iPad or a simulator.

**Command line:**

```sh
wordcraft-cli convert report.docx report.pdf        # docx, pdf, odt, rtf, html, md, tex, txt, png
wordcraft-cli text report.docx                      # plain text
wordcraft-cli inspect report.docx                   # structure as JSON
wordcraft-cli run --template sample \
  --cmd 'select.text={"text":"Membership"}' --cmd format.bold --save out.docx
```

## For agents: CLI and MCP

The editor is designed to be driven by people *and* by AI agents.

```sh
claude mcp add wordpadx -- wordcraft-cli mcp                   # headless documents
wordcraft --control 7981 &                                     # or drive the running app…
claude mcp add wordpadx-app -- wordcraft-cli mcp --connect 127.0.0.1:7981
```

Tools include `list_commands`, `execute`, `batch`, `type_text`, `select_text`, `inspect_document`,
`render_page`, `save_document`, and, with a running app, `screenshot`, `click` and `key`. Agents
can check their work through `inspect_document` without screenshots. See [docs/mcp.md](docs/mcp.md)
and the [control protocol](docs/control-protocol.md). Macros record any sequence of commands and
play it back (`tools.recordMacro`, `tools.macros`).

## Logs

The desktop app writes its log records to standard error and to `logs/wordcraft.log` next to its
preferences: on Linux `$XDG_CONFIG_HOME/wordcraft/logs/` (by default `~/.config/wordcraft/logs/`),
on macOS `~/Library/Application Support/WordCraft/logs/`, on Windows `%APPDATA%\WordCraft\logs\`.
Attach this file to a bug report: unreadable parts of a .docx, pictures or fonts a PDF export had
to leave out, and panics the command guard recovered from are recorded there. Each launch moves the
previous log to `wordcraft.1.log` (and that one to `wordcraft.2.log`), so the log of a run that
crashed survives the next start; a log stops growing at 16 MiB. Document text is never logged.

`RUST_LOG` overrides the default levels with env_logger-style directives, for example
`RUST_LOG=debug` or `RUST_LOG=warn,wordcraft_docx=trace`. The logger is
`apps/wordcraft/src/logging.rs`.

**Windows graphics troubleshooting.** The desktop app uses DirectX 12 and falls back to OpenGL on
PCs without a hardware DirectX 12 driver. If it fails to open, try another backend from PowerShell:
`$env:WGPU_BACKEND = "vulkan"; & .\wordcraft.exe`.

## Architecture

| Layer | Crate | Job |
|---|---|---|
| L0 | `wordcraft-geom` | units and measurements |
| L1 | `wordcraft-doc`, `wordcraft-fonts`, `wordcraft-proof` | document model and editing; fonts and shaping; spelling, grammar, hyphenation |
| L2 | `wordcraft-layout`, `wordcraft-docx`, `wordcraft-formats` | line breaking, pagination, tables, notes, hit testing; OOXML; ODT/RTF/HTML/Markdown/TXT |
| L3 | `wordcraft-render`, `wordcraft-pdf` | rasteriser (vello_cpu); PDF (krilla) |
| L4 | `wordcraft-engine` | session, undo, commands, Word feature catalog |
| L5 | `wordcraft-mcp` | MCP server |
| L6 | `wordcraft-ui-egui` | the Word-style front end (swappable) |
| apps | `wordcraft`, `wordcraft-cli`, `wordcraft-web` | desktop, command line, browser and PWA |
| mobile | `mobile/android`, `mobile/ios` | native Android tablet and iPad shells that host the web build |

`cargo xtask ci` runs formatting, clippy, the test suite (about 1,500 tests), the
asset-attribution check, the layering check and the wasm build. Contributor and agent
instructions: [AGENTS.md](AGENTS.md) and [CLAUDE.md](CLAUDE.md).

## Status

Wordpadx is in **alpha**. Commands cover 97% of Word's ribbon features (483 of 498, measured by
`cargo xtask parity`). Counting depth, file fidelity and stability as well, it is roughly 60% of
the way to replacing Word for real work. The tablet builds are new: the Android APK is a debug
build and the iPad build is unsigned. Details: [ROADMAP.md](ROADMAP.md),
[where we fall short](docs/gaps.md) and [TABLET_GUIDE.md](TABLET_GUIDE.md).

## License and credits

Wordpadx is dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
It is a fork of [WordCraft](https://github.com/storytold/wordcraft), Copyright (c) 2026 ArtCraft
Team and the WordCraft contributors; required notices are in [NOTICE](NOTICE).

Bundled fonts, icons, images and other assets keep their own open licenses; each one is listed
with its author, source and license in [ATTRIBUTION.md](ATTRIBUTION.md). The spelling dictionary
and hyphenation come from Grady Ward's public-domain Moby Hyphenator II word list. The sample
documents and templates are original text.

Wordpadx is not affiliated with or endorsed by the ArtCraft team.

<sub>Microsoft and Microsoft Word are trademarks of the Microsoft group of companies. Wordpadx is an independent, open-source project and is not affiliated with, sponsored by or endorsed by Microsoft Corporation; the name Word is used only to describe the workflows it is compatible with.</sub>
