//! Centralised font loading, caching, and styling helper.
//!
//! Avoids duplicate WOFF2-to-TTF decompression by caching decompressed bytes in OnceLocks,
//! and provides a unified representation of font sets and styles for all renderers.

use std::sync::OnceLock;

use std::collections::BTreeSet;

use ab_glyph::{Font, FontArc};
use anyhow::{Context, Result, anyhow};
use woff2_patched::convert_woff2_to_ttf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SvgFontEmbeddingPolicy {
    AllowSubsetting,
    EmbedFullFont { reason: String },
    OmitFont { reason: String },
}

struct EmbeddedFont {
    name: &'static str,
    bytes: &'static [u8],
    family_name: &'static str,
    weight: &'static str,
    style: &'static str,
    lock: OnceLock<Vec<u8>>,
}

impl EmbeddedFont {
    fn get_ttf(&self) -> &[u8] {
        self.lock.get_or_init(|| {
            convert_woff2_to_ttf(&mut std::io::Cursor::new(self.bytes)).unwrap_or_else(|err| {
                panic!(
                    "failed to decompress embedded WOFF2 face {}: {:?}",
                    self.name, err
                )
            })
        })
    }
}

// We define our embedded font records statically. Each contains a OnceLock for lazy decompression.
static EMBEDDED_FONTS: [EmbeddedFont; 10] = [
    EmbeddedFont {
        name: "JetBrainsMonoNerdFontMono-Regular.woff2",
        bytes: include_bytes!(concat!(
            env!("OUT_DIR"),
            "/JetBrainsMonoNerdFontMono-Regular.woff2"
        )),
        family_name: "JetBrainsMono Nerd Font Mono",
        weight: "normal",
        style: "normal",
        lock: OnceLock::new(),
    },
    EmbeddedFont {
        name: "JetBrainsMonoNerdFontMono-Bold.woff2",
        bytes: include_bytes!(concat!(
            env!("OUT_DIR"),
            "/JetBrainsMonoNerdFontMono-Bold.woff2"
        )),
        family_name: "JetBrainsMono Nerd Font Mono",
        weight: "bold",
        style: "normal",
        lock: OnceLock::new(),
    },
    EmbeddedFont {
        name: "JetBrainsMonoNerdFontMono-Italic.woff2",
        bytes: include_bytes!(concat!(
            env!("OUT_DIR"),
            "/JetBrainsMonoNerdFontMono-Italic.woff2"
        )),
        family_name: "JetBrainsMono Nerd Font Mono",
        weight: "normal",
        style: "italic",
        lock: OnceLock::new(),
    },
    EmbeddedFont {
        name: "JetBrainsMonoNerdFontMono-BoldItalic.woff2",
        bytes: include_bytes!(concat!(
            env!("OUT_DIR"),
            "/JetBrainsMonoNerdFontMono-BoldItalic.woff2"
        )),
        family_name: "JetBrainsMono Nerd Font Mono",
        weight: "bold",
        style: "italic",
        lock: OnceLock::new(),
    },
    EmbeddedFont {
        name: "NotoSansMono-Regular.woff2",
        bytes: include_bytes!(concat!(env!("OUT_DIR"), "/NotoSansMono-Regular.woff2")),
        family_name: "Noto Sans Mono",
        weight: "normal",
        style: "normal",
        lock: OnceLock::new(),
    },
    EmbeddedFont {
        name: "NotoEmoji-Regular.woff2",
        bytes: include_bytes!(concat!(env!("OUT_DIR"), "/NotoEmoji-Regular.woff2")),
        family_name: "Noto Emoji",
        weight: "normal",
        style: "normal",
        lock: OnceLock::new(),
    },
    EmbeddedFont {
        name: "NotoSansSymbols2-Regular.woff2",
        bytes: include_bytes!(concat!(env!("OUT_DIR"), "/NotoSansSymbols2-Regular.woff2")),
        family_name: "Noto Sans Symbols 2",
        weight: "normal",
        style: "normal",
        lock: OnceLock::new(),
    },
    EmbeddedFont {
        name: "NotoSansMonoCJKjp-Subset.woff2",
        bytes: include_bytes!(concat!(env!("OUT_DIR"), "/NotoSansMonoCJKjp-Subset.woff2")),
        family_name: "Noto Sans Mono CJK JP",
        weight: "normal",
        style: "normal",
        lock: OnceLock::new(),
    },
    EmbeddedFont {
        name: "unifont_upper-17.0.04.woff2",
        bytes: include_bytes!(concat!(env!("OUT_DIR"), "/unifont_upper-17.0.04.woff2")),
        family_name: "unifont_upper",
        weight: "normal",
        style: "normal",
        lock: OnceLock::new(),
    },
    EmbeddedFont {
        name: "unifont_csur-17.0.04.woff2",
        bytes: include_bytes!(concat!(env!("OUT_DIR"), "/unifont_csur-17.0.04.woff2")),
        family_name: "unifont_csur",
        weight: "normal",
        style: "normal",
        lock: OnceLock::new(),
    },
];

/// Detailed metadata and raw assets for a loaded font face.
#[derive(Debug, Clone)]
pub enum FontSource {
    Embedded { index: usize },
    Custom { bytes: Vec<u8> },
}

#[derive(Debug, Clone)]
pub struct FontInfo {
    /// ab_glyph FontArc for rasterization (lazily parsed).
    pub font: OnceLock<FontArc>,
    /// Source from which TTF bytes are fetched/decompressed on demand.
    pub source: FontSource,
    /// Original WOFF2 bytes if available (for efficient SVG embedding).
    pub woff2_bytes: Option<Vec<u8>>,
    /// CSS font family name.
    pub family_name: String,
    /// CSS font weight (e.g. "normal", "bold").
    pub weight: String,
    /// CSS font style (e.g. "normal", "italic").
    pub style: String,
    /// Whether this font was explicitly loaded from a custom file path.
    pub is_custom: bool,
}

impl FontInfo {
    pub fn get_ttf_bytes(&self) -> &[u8] {
        match &self.source {
            FontSource::Custom { bytes } => bytes,
            FontSource::Embedded { index } => EMBEDDED_FONTS[*index].get_ttf(),
        }
    }

    pub fn get_font(&self) -> &FontArc {
        self.font.get_or_init(|| {
            FontArc::try_from_vec(self.get_ttf_bytes().to_vec()).expect("invalid font face bytes")
        })
    }

    fn svg_embedding_policy_from_flags(
        family_name: &str,
        embedding_is_lenient: bool,
        embed_only_bitmaps: bool,
        allow_subsetting: bool,
        embedding_desc: &str,
    ) -> SvgFontEmbeddingPolicy {
        if embed_only_bitmaps {
            return SvgFontEmbeddingPolicy::OmitFont {
                reason: format!(
                    "Font permissions for '{}' allow only bitmap embedding; omitting embedded font data from this SVG.",
                    family_name
                ),
            };
        }

        if !embedding_is_lenient {
            return SvgFontEmbeddingPolicy::OmitFont {
                reason: format!(
                    "Font permissions for '{}' do not allow outline embedding ({}); omitting embedded font data from this SVG.",
                    family_name, embedding_desc
                ),
            };
        }

        if !allow_subsetting {
            return SvgFontEmbeddingPolicy::EmbedFullFont {
                reason: format!(
                    "Font permissions for '{}' disallow subsetting; embedding the full font data in this SVG.",
                    family_name
                ),
            };
        }

        SvgFontEmbeddingPolicy::AllowSubsetting
    }

    pub fn svg_embedding_policy(&self) -> Result<SvgFontEmbeddingPolicy> {
        let reader =
            font_subset::FontReader::new(self.get_ttf_bytes()).map_err(|e| anyhow!("{e:?}"))?;
        let font = reader.read().map_err(|e| anyhow!("{e:?}"))?;
        let permissions = font.permissions();

        Ok(Self::svg_embedding_policy_from_flags(
            &self.family_name,
            permissions.embedding.is_lenient(),
            permissions.embed_only_bitmaps,
            permissions.allow_subsetting,
            &format!("{:?}", permissions.embedding),
        ))
    }

    pub fn subset(&self, chars: &BTreeSet<char>) -> Result<Vec<u8>> {
        if chars.is_empty() {
            return Err(anyhow!("no characters to subset"));
        }

        let reader =
            font_subset::FontReader::new(self.get_ttf_bytes()).map_err(|e| anyhow!("{e:?}"))?;
        let font = reader.read().map_err(|e| anyhow!("{e:?}"))?;
        let subset = font.subset(chars).map_err(|e| anyhow!("{e:?}"))?;
        Ok(subset.to_woff2())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_custom_font_keeps_the_embedded_fallbacks() {
        use ab_glyph::Font;
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/fonts/NotoSansMonoCJKjp-Subset.otf"
        );
        let set = super::load_font_family(Some(path)).unwrap().font_set;
        assert_eq!(set.regular[0], 0);
        assert!(set.regular.len() > 1);
        let (idx, font) = set.select_for_char(0, '✔');
        assert_ne!(idx, 0);
        assert_ne!(font.glyph_id('✔').0, 0);
    }

    use super::{FontInfo, SvgFontEmbeddingPolicy};

    #[test]
    fn svg_policy_omits_bitmap_only_fonts() {
        let policy = FontInfo::svg_embedding_policy_from_flags(
            "Example Font",
            true,
            true,
            true,
            "Installable",
        );
        assert_eq!(
            policy,
            SvgFontEmbeddingPolicy::OmitFont {
                reason: "Font permissions for 'Example Font' allow only bitmap embedding; omitting embedded font data from this SVG.".to_string(),
            }
        );
    }

    #[test]
    fn svg_policy_full_embeds_when_subsetting_disallowed() {
        let policy = FontInfo::svg_embedding_policy_from_flags(
            "Example Font",
            true,
            false,
            false,
            "Installable",
        );
        assert_eq!(
            policy,
            SvgFontEmbeddingPolicy::EmbedFullFont {
                reason: "Font permissions for 'Example Font' disallow subsetting; embedding the full font data in this SVG.".to_string(),
            }
        );
    }
}

/// A collection of font faces organised by style.
#[derive(Debug, Clone)]
pub struct FontSet {
    /// All loaded font faces, indexed.
    pub fonts: Vec<FontInfo>,
    /// Font indices (into `fonts`) to try for regular text, in priority order.
    pub regular: Vec<usize>,
    /// Font indices to try for bold text.
    pub bold: Vec<usize>,
    /// Font indices to try for italic text.
    pub italic: Vec<usize>,
    /// Font indices to try for bold-italic text.
    pub bold_italic: Vec<usize>,
}

impl FontSet {
    /// Returns the ordered font-index slice appropriate for style flags.
    pub fn indices_for_flags(&self, flags: u8) -> &[usize] {
        let want_bold = flags & crate::recording::style_flags::BOLD != 0;
        let want_italic = flags & crate::recording::style_flags::ITALIC != 0;
        let list = match (want_bold, want_italic) {
            (true, true) => &self.bold_italic,
            (true, false) => &self.bold,
            (false, true) => &self.italic,
            (false, false) => &self.regular,
        };
        if list.is_empty() { &self.regular } else { list }
    }

    /// Select the best `(font_index, &FontArc)` for `ch` given cell style flags.
    pub fn select_for_char(&self, flags: u8, ch: char) -> (usize, &FontArc) {
        let indices = self.indices_for_flags(flags);
        for &idx in indices {
            let font = self.fonts[idx].get_font();
            if has_glyph(font, ch) {
                return (idx, font);
            }
        }
        let idx = indices[0];
        (idx, self.fonts[idx].get_font())
    }
}

fn has_glyph(font: &FontArc, ch: char) -> bool {
    font.glyph_id(ch).0 != 0
}

#[derive(Debug, Clone)]
pub struct LoadedFontFamily {
    pub font_set: FontSet,
    pub description: String,
}

/// Extract family name from raw TrueType font bytes using `ttf-parser`.
/// Fall back to a sanitized filename or generic name if not found.
pub fn extract_family_name(ttf_bytes: &[u8], fallback: &str) -> String {
    if let Ok(face) = ttf_parser::Face::parse(ttf_bytes, 0) {
        for name in face.names() {
            if name.name_id == 1 {
                if let Some(family) = name.to_string() {
                    return family;
                }
            }
        }
    }
    fallback.to_string()
}

/// The family a face belongs to, preferring the typographic family (name 16),
/// which every weight of a family shares, over the legacy family (name 1).
fn typographic_family(ttf_bytes: &[u8]) -> Option<String> {
    let face = ttf_parser::Face::parse(ttf_bytes, 0).ok()?;
    let named = |id| {
        face.names()
            .into_iter()
            .find(|n| n.name_id == id)
            .and_then(|n| n.to_string())
    };
    named(16).or_else(|| named(1))
}

fn custom_face(bytes: Vec<u8>, family_name: &str, weight: &str) -> Result<FontInfo> {
    let face = FontArc::try_from_vec(bytes.clone()).context("invalid font file")?;
    Ok(FontInfo {
        font: OnceLock::from(face),
        source: FontSource::Custom { bytes },
        woff2_bytes: None,
        family_name: family_name.to_string(),
        weight: weight.to_string(),
        style: "normal".to_string(),
        is_custom: true,
    })
}

/// The system fonts a macOS terminal falls back to for symbols its font
/// lacks, such as Menlo's flag (U+2691), in the order they are tried.
const SYSTEM_FALLBACK_FAMILIES: &[&str] = &["Menlo", "Apple Symbols"];

/// The faces of `SYSTEM_FALLBACK_FAMILIES` installed on this machine. They
/// come after the embedded fallbacks, so output is the same wherever the
/// embedded fonts cover a character.
fn system_fallbacks() -> Vec<FontInfo> {
    let mut db = fontdb::Database::new();
    db.load_system_fonts();
    SYSTEM_FALLBACK_FAMILIES
        .iter()
        .filter_map(|family| {
            let query = fontdb::Query {
                families: &[fontdb::Family::Name(family)],
                ..Default::default()
            };
            let id = db.query(&query)?;
            let (bytes, index) = db.with_face_data(id, |data, index| (data.to_vec(), index))?;
            // Only the first face of a collection is read; FontArc parses index 0.
            (index == 0)
                .then(|| custom_face(bytes, family, "normal").ok())
                .flatten()
        })
        .collect()
}

/// The upright bold face of `family` in the folder of `path`, found the way a
/// terminal finds the bold face of its configured font.
fn sibling_bold(path: &std::path::Path, family: &str) -> Option<Vec<u8>> {
    let mut entries: Vec<_> = std::fs::read_dir(path.parent()?)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .collect();
    entries.sort();
    entries.into_iter().filter(|p| p != path).find_map(|p| {
        let ext = p.extension()?.to_str()?.to_ascii_lowercase();
        if !matches!(ext.as_str(), "ttf" | "otf") {
            return None;
        }
        let bytes = std::fs::read(&p).ok()?;
        let face = ttf_parser::Face::parse(&bytes, 0).ok()?;
        let bold = face.weight().to_number() == 700 && !face.is_italic();
        (bold && typographic_family(&bytes).as_deref() == Some(family)).then_some(bytes)
    })
}

/// Load the requested font set. If `path` is provided, that face is the primary
/// font, with the bold face of its family from the same folder when there is one;
/// the embedded fallbacks still cover the characters it lacks, as a terminal's
/// fallback fonts do. Otherwise, the embedded JetBrains Mono family is loaded
/// along with default fallbacks.
pub fn load_font_family(path: Option<&str>) -> Result<LoadedFontFamily> {
    if let Some(p) = path {
        let bytes = std::fs::read(p).with_context(|| format!("reading font {p}"))?;
        let family_name = extract_family_name(&bytes, "CustomFont");
        let typographic = typographic_family(&bytes);
        let bold_bytes = typographic
            .as_deref()
            .and_then(|family| sibling_bold(std::path::Path::new(p), family));

        let mut fonts = vec![custom_face(bytes, &family_name, "normal")?];
        let bold_idx = match bold_bytes {
            Some(b) => {
                fonts.push(custom_face(b, &family_name, "bold")?);
                fonts.len() - 1
            }
            None => 0,
        };

        let default = load_font_family(None)?.font_set;
        let offset = fonts.len();
        fonts.extend(default.fonts.iter().cloned());
        let system_start = fonts.len();
        fonts.extend(system_fallbacks());
        let system: Vec<usize> = (system_start..fonts.len()).collect();
        let chain = |first: usize, list: &[usize]| -> Vec<usize> {
            std::iter::once(first)
                .chain(list.iter().skip(1).map(|i| i + offset))
                .chain(system.iter().copied())
                .collect()
        };

        let font_set = FontSet {
            regular: chain(0, &default.regular),
            bold: chain(bold_idx, &default.bold),
            italic: chain(0, &default.italic),
            bold_italic: chain(bold_idx, &default.bold_italic),
            fonts,
        };

        let description = if bold_idx == 0 {
            format!("explicit path: {p} + embedded fallbacks")
        } else {
            format!("explicit path: {p} with its bold face + embedded fallbacks")
        };
        return Ok(LoadedFontFamily {
            font_set,
            description,
        });
    }

    static DEFAULT_FAMILY: OnceLock<LoadedFontFamily> = OnceLock::new();
    if let Some(cached) = DEFAULT_FAMILY.get() {
        return Ok(cached.clone());
    }

    let default_family = load_default_font_family_internal()?;
    let _ = DEFAULT_FAMILY.set(default_family.clone());
    Ok(default_family)
}

fn load_default_font_family_internal() -> Result<LoadedFontFamily> {
    let _timer = crate::telemetry::ScopeTimer::new("font_init");
    // Default embedded fonts.
    let mut fonts = Vec::new();
    // Regular, Bold, Italic, BoldItalic variants of JetBrains Mono.
    fonts.push(FontInfo {
        font: OnceLock::new(),
        source: FontSource::Embedded { index: 0 },
        woff2_bytes: Some(EMBEDDED_FONTS[0].bytes.to_vec()),
        family_name: EMBEDDED_FONTS[0].family_name.to_string(),
        weight: EMBEDDED_FONTS[0].weight.to_string(),
        style: EMBEDDED_FONTS[0].style.to_string(),
        is_custom: false,
    });

    let idx_bold = {
        let i = fonts.len();
        fonts.push(FontInfo {
            font: OnceLock::new(),
            source: FontSource::Embedded { index: 1 },
            woff2_bytes: Some(EMBEDDED_FONTS[1].bytes.to_vec()),
            family_name: EMBEDDED_FONTS[1].family_name.to_string(),
            weight: EMBEDDED_FONTS[1].weight.to_string(),
            style: EMBEDDED_FONTS[1].style.to_string(),
            is_custom: false,
        });
        Some(i)
    };

    let idx_italic = {
        let i = fonts.len();
        fonts.push(FontInfo {
            font: OnceLock::new(),
            source: FontSource::Embedded { index: 2 },
            woff2_bytes: Some(EMBEDDED_FONTS[2].bytes.to_vec()),
            family_name: EMBEDDED_FONTS[2].family_name.to_string(),
            weight: EMBEDDED_FONTS[2].weight.to_string(),
            style: EMBEDDED_FONTS[2].style.to_string(),
            is_custom: false,
        });
        Some(i)
    };

    let idx_bold_italic = {
        let i = fonts.len();
        fonts.push(FontInfo {
            font: OnceLock::new(),
            source: FontSource::Embedded { index: 3 },
            woff2_bytes: Some(EMBEDDED_FONTS[3].bytes.to_vec()),
            family_name: EMBEDDED_FONTS[3].family_name.to_string(),
            weight: EMBEDDED_FONTS[3].weight.to_string(),
            style: EMBEDDED_FONTS[3].style.to_string(),
            is_custom: false,
        });
        Some(i)
    };

    // Fallbacks
    let fallback_start = fonts.len();
    let mut fallback_names = Vec::new();

    for i in 4..10 {
        let emb = &EMBEDDED_FONTS[i];
        fonts.push(FontInfo {
            font: OnceLock::new(),
            source: FontSource::Embedded { index: i },
            woff2_bytes: Some(emb.bytes.to_vec()),
            family_name: emb.family_name.to_string(),
            weight: emb.weight.to_string(),
            style: emb.style.to_string(),
            is_custom: false,
        });
        fallback_names.push(format!("{} (embedded)", emb.family_name));
    }

    let fallback_indices: Vec<usize> = (fallback_start..fonts.len()).collect();

    let regular: Vec<usize> = std::iter::once(0)
        .chain(fallback_indices.iter().copied())
        .collect();
    let bold: Vec<usize> = idx_bold
        .into_iter()
        .chain(fallback_indices.iter().copied())
        .collect();
    let italic: Vec<usize> = idx_italic
        .into_iter()
        .chain(fallback_indices.iter().copied())
        .collect();
    let bold_italic: Vec<usize> = idx_bold_italic
        .into_iter()
        .chain(fallback_indices.iter().copied())
        .collect();

    let description = if fallback_names.is_empty() {
        "embedded default: JetBrainsMono Nerd Font Mono family".to_string()
    } else {
        format!(
            "embedded default: JetBrainsMono Nerd Font Mono family + fallbacks [{}]",
            fallback_names.join(" -> ")
        )
    };

    Ok(LoadedFontFamily {
        font_set: FontSet {
            fonts,
            regular,
            bold,
            italic,
            bold_italic,
        },
        description,
    })
}
