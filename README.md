<p align="center">
  <img alt="Wordpadx app icon" src="assets/app-icon/wordcraft.svg" width="96">
</p>

<h1 align="center">Wordpadx</h1>

<p align="center">
  <b>Write, design and share documents on your tablet, in the browser and on the desktop.</b>
</p>

<p align="center">
  Wordpadx is a fast, private, open-source word processor written in Rust. It has the ribbon,
  styles, tables, track changes, references and mail merge you already know, opens and saves
  .docx, and works offline on Android tablets, iPad, the web, macOS, Windows, Linux and BSD.
</p>

<p align="center">
  <img alt="Written in Rust" src="https://img.shields.io/badge/written%20in-Rust-2f54eb?style=flat-square&logo=rust&logoColor=white">
  <img alt="Runs on Android tablet, iPad, web and desktop" src="https://img.shields.io/badge/runs%20on-Android%20%C2%B7%20iPad%20%C2%B7%20Web%20%C2%B7%20Desktop-f5a524?style=flat-square">
  <img alt="License: MIT OR Apache-2.0" src="https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-2f54eb?style=flat-square">
</p>

<p align="center">
  <a href="https://github.com/abahvictor360-sketch/Wordpadx/releases"><b>Download</b></a> ·
  <a href="https://abahvictor360-sketch.github.io/Wordpadx/">Website</a> ·
  <a href="TABLET_GUIDE.md">Tablet guide</a> ·
  <a href="https://github.com/abahvictor360-sketch/Wordpadx/issues">Report a problem</a>
</p>

<p align="center">
  <img src="docs/images/hero.png" alt="Wordpadx with the Home tab of the ribbon open over a two-page document. The Navigation pane on the left lists the headings and the Styles gallery shows live previews." width="100%">
</p>

## Contents

- [Get Wordpadx](#get-wordpadx)
- [Features](#features)
- [A tour](#a-tour)
- [Build from source](#build-from-source)
- [Command line and AI agents](#command-line-and-ai-agents)
- [Troubleshooting](#troubleshooting)
- [Project layout](#project-layout)
- [Status](#status)
- [Contributing](#contributing)
- [License](#license)

## Get Wordpadx

| Device | How to install | Notes |
|---|---|---|
| **Android tablet** | Download `app-debug.apk` from the [latest release](https://github.com/abahvictor360-sketch/Wordpadx/releases/latest) | Debug-signed: allow installs from unknown sources. Opens .docx, .odt and .txt; exports go to Downloads. Works with Samsung DeX, split screen and a stylus. |
| **iPad** | Install the web app (below), or download `Wordpadx-iPad.zip` from the [latest release](https://github.com/abahvictor360-sketch/Wordpadx/releases/latest) | The zip is unsigned and must be signed before it installs; building it in Xcode is simpler. Supports Split View, Stage Manager and Apple Pencil. |
| **Any tablet browser** | Open the website, then Safari → Share → *Add to Home Screen*, or Chrome → ⋮ → *Install app* | Needs WebGPU or WebGL2. Runs full screen and keeps working offline once loaded. |
| **macOS, Windows, Linux, BSD** | [Build from source](#build-from-source) | Native app on the GPU. |

Every push to `main` builds the web app, the Android APK and the iPad app
([`build-mobile.yml`](.github/workflows/build-mobile.yml)), and each successful build is published
as a `v0.5.0-build.N` prerelease. Step-by-step tablet instructions: **[TABLET_GUIDE.md](TABLET_GUIDE.md)**.

## Features

- **Familiar.** The ribbon tabs, groups and shortcuts you know from Word: Enter continues a list,
  Tab demotes it, Ctrl/⌘+B bolds the word under the caret, the Styles gallery previews live,
  F4 repeats, F7 checks spelling.
- **Your files.** Opens and saves .docx, .odt, .rtf, .html, .md, .tex and .txt; opens Word 97-2003
  .doc; exports PDF with selectable text, links and bookmarks.
- **Made for tablets.** A native Android app, an iPad app and an installable web app from the same
  build. The ribbon's tab strip scrolls on narrow screens, and the keyboard opens when you tap the page.
- **Fast.** Layout is cached per paragraph, so typing in a 188-page document re-lays it out in
  about 1.4 ms.
- **Private.** Spelling, grammar and everything else run on your device. No account, no tracking.
- **Scriptable.** Every action is a command with an id. The same 600+ commands drive the ribbon,
  shortcuts, command search, a command-line tool and an MCP server for AI agents.
- **Open.** MIT OR Apache-2.0, built from public file-format specifications, with every asset
  original or openly licensed.

| Area | Highlights |
|---|---|
| **Writing** | Typing with IME, smart quotes, AutoCorrect, list autoformat, find and replace with regex, undo/redo |
| **Formatting** | Fonts, sizes, bold/italic/underline, strike, sub/superscript, highlight, colours, character spacing, Format Painter, Change Case |
| **Paragraphs** | Alignment, indents on the ruler, spacing, line spacing, tabs with leaders, borders, shading, keep with next, widow/orphan control |
| **Styles** | Built-in styles, live gallery, Styles pane, create and modify styles, style sets, themes |
| **Lists** | Bullets, numbering, multilevel, restart, custom formats |
| **Tables** | Insert by grid, merge/split, table styles, borders, shading, repeated header rows, sort, formulas, text ↔ table |
| **Pages** | Margins, orientation, size, columns, section breaks, headers and footers, page numbers, watermarks, page borders, line numbers, drop caps, hyphenation |
| **Objects** | Pictures (crop, recolour, transparency, background removal, styles, rotate), shapes, text boxes, text wrapping |
| **References** | Table of contents, footnotes and endnotes, citations and bibliography (APA, MLA, Chicago, IEEE), captions, cross-references, index |
| **Review** | Spelling and grammar, thesaurus, word count, comments, track changes, compare documents, restrict editing, accessibility checker |
| **Mailings** | Mail merge from CSV, merge fields, address block, greeting line, preview, envelopes and labels |
| **View** | Print layout, web layout, draft, read mode, focus, zoom, Navigation pane, rulers, dark mode; interface in 12 languages |

## A tour

<table>
<tr>
<td width="50%" valign="top"><img src="docs/images/review.png" alt="The Review tab with tracked insertions and deletions and comment balloons in the margin" width="100%"><p align="center"><sub><b>Review.</b> Track changes, comments in the margin, spelling and grammar as you type.</sub></p></td>
<td width="50%" valign="top"><img src="docs/images/references.png" alt="The References tab with a table of contents and a styled table" width="100%"><p align="center"><sub><b>References.</b> Tables of contents, footnotes, citations and an index.</sub></p></td>
</tr>
<tr>
<td width="50%" valign="top"><img src="docs/images/design.png" alt="The Design tab with style-set previews and a DRAFT watermark" width="100%"><p align="center"><sub><b>Design.</b> Themes, style sets, watermarks, page colour and borders.</sub></p></td>
<td width="50%" valign="top"><img src="docs/images/dark.png" alt="Dark mode with the Insert tab open and formatting marks shown" width="100%"><p align="center"><sub><b>Dark mode</b> with formatting marks and the Insert tab.</sub></p></td>
</tr>
<tr>
<td width="50%" valign="top"><img src="docs/images/layout.png" alt="The Layout tab with a drop cap, text wrapping around a shape and line numbers" width="100%"><p align="center"><sub><b>Layout.</b> Drop caps, text wrapping, hyphenation and line numbers.</sub></p></td>
<td width="50%" valign="top"><img src="docs/images/backstage.png" alt="The File start page with templates and recent documents" width="100%"><p align="center"><sub><b>File.</b> Templates, recent documents, export to PDF and other formats.</sub></p></td>
</tr>
</table>

## Build from source

You need Rust 1.90 or newer.

```sh
git clone https://github.com/abahvictor360-sketch/Wordpadx
cd Wordpadx
cargo run --release -p wordcraft -- --sample        # desktop app with the sample document
cargo run --release -p wordcraft -- report.docx     # open a document
```

The crates and binaries are named `wordcraft`, `wordcraft-cli` and `wordcraft-web` internally;
the app itself is Wordpadx.

**Web app.** Needs [Trunk](https://trunkrs.dev) and the wasm target:

```sh
rustup target add wasm32-unknown-unknown
cd apps/wordcraft-web
trunk serve                     # open http://127.0.0.1:8771/?sample
trunk build --release           # static site in dist/web
```

**Android tablet.** `./packaging/android/build.sh` (Windows: `.\packaging\android\build.ps1`)
builds the web app, copies it into `mobile/android` and runs Gradle. Add `--release` for a release
APK or `--skip-web` to reuse an existing `dist/web`. You can also open `mobile/android` in Android
Studio.

**iPad.** On a Mac with Xcode, `./packaging/ios/build.sh` builds the web app and the iPad app, or
open `mobile/ios/Wordpadx.xcodeproj` and choose your iPad or a simulator.

## Command line and AI agents

```sh
wordcraft-cli convert report.docx report.pdf        # docx, pdf, odt, rtf, html, md, tex, txt, png
wordcraft-cli text report.docx                      # plain text
wordcraft-cli inspect report.docx                   # structure as JSON
wordcraft-cli run --template sample \
  --cmd 'select.text={"text":"Membership"}' --cmd format.bold --save out.docx
```

AI agents can edit documents over MCP, either headless or by driving the running app:

```sh
claude mcp add wordpadx -- wordcraft-cli mcp
wordcraft --control 7981 &
claude mcp add wordpadx-app -- wordcraft-cli mcp --connect 127.0.0.1:7981
```

Tools include `list_commands`, `execute`, `batch`, `type_text`, `select_text`, `inspect_document`,
`render_page` and `save_document`, plus `screenshot`, `click` and `key` with a running app. See
[docs/mcp.md](docs/mcp.md) and the [control protocol](docs/control-protocol.md).

## Troubleshooting

**Logs.** The desktop app logs to standard error and to `logs/wordcraft.log` in its settings folder:
`~/.config/wordcraft/logs/` on Linux, `~/Library/Application Support/WordCraft/logs/` on macOS,
`%APPDATA%\WordCraft\logs\` on Windows. Attach it to bug reports. The previous two runs are kept
as `wordcraft.1.log` and `wordcraft.2.log`. Document text is never logged. `RUST_LOG` changes the
detail, for example `RUST_LOG=debug`.

**Windows graphics.** The app uses DirectX 12 and falls back to OpenGL on PCs without a hardware
DirectX 12 driver. If it doesn't open, try another backend from PowerShell:
`$env:WGPU_BACKEND = "vulkan"; & .\wordcraft.exe`.

**Web and tablets.** The web app needs WebGPU or WebGL2. If it stays on "Loading", update the
browser or try Chrome or Safari.

## Project layout

| Layer | Crate | Job |
|---|---|---|
| L0 | `wordcraft-geom` | units and measurements |
| L1 | `wordcraft-doc`, `wordcraft-fonts`, `wordcraft-proof` | document model; fonts and shaping; spelling, grammar, hyphenation |
| L2 | `wordcraft-layout`, `wordcraft-docx`, `wordcraft-formats` | line breaking and pagination; .docx; ODT/RTF/HTML/Markdown/TXT |
| L3 | `wordcraft-render`, `wordcraft-pdf` | page rendering; PDF export |
| L4 | `wordcraft-engine` | session, undo, commands |
| L5 | `wordcraft-mcp` | MCP server |
| L6 | `wordcraft-ui-egui` | the user interface |
| apps | `apps/wordcraft`, `apps/wordcraft-cli`, `apps/wordcraft-web` | desktop, command line, web app |
| mobile | `mobile/android`, `mobile/ios` | Android tablet and iPad apps that host the web build |
| website | `website/` | the landing page |

## Status

Wordpadx is in **alpha**. Commands cover 97% of Word's ribbon features (483 of 498, measured by
`cargo xtask parity`). Real-world .docx fidelity, pagination and stability are still being
improved, and the tablet builds are new: the Android APK is debug-signed and the iPad build is
unsigned. Details: [ROADMAP.md](ROADMAP.md) and [docs/gaps.md](docs/gaps.md).

## Contributing

Bug reports and pull requests are welcome. Before you commit, run:

```sh
cargo xtask ci     # formatting, clippy, ~1,500 tests, asset check, layering check, wasm build
```

Coding rules (no panics, every asset listed in [ATTRIBUTION.md](ATTRIBUTION.md), no Microsoft
assets) are in [CLAUDE.md](CLAUDE.md) and [AGENTS.md](AGENTS.md).

## License

Wordpadx is dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
Copyright notices are in [LICENSE-MIT](LICENSE-MIT) and [NOTICE](NOTICE). Fonts, icons and other
assets keep their own open licenses, listed in [ATTRIBUTION.md](ATTRIBUTION.md).

<sub>Microsoft and Microsoft Word are trademarks of the Microsoft group of companies. Wordpadx is an independent open-source project and is not affiliated with or endorsed by Microsoft.</sub>
