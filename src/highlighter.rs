// A copy of https://github.com/iced-rs/iced/blob/master/highlighter/src/lib.rs
// to load custom SyntaxSet

use std::ops::Range;
use std::sync::LazyLock;

use iced::Color;
use iced_core::font::{self, Font};
use iced_core::text::highlighter::{self, Format};

use syntect::highlighting::{self, ThemeSet};
use syntect::parsing::syntax_definition::SyntaxDefinition;
use syntect::parsing::{
    ParseState, ScopeStack, ScopeStackOp, SyntaxReference, SyntaxSet, SyntaxSetBuilder,
};

const SYNTAX_SOURCE: &str = include_str!("../wgsl.sublime-syntax");

static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(make_syntax_set);
static THEMES: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);

fn make_syntax_set() -> SyntaxSet {
    let definition = SyntaxDefinition::load_from_str(SYNTAX_SOURCE, true, None)
        .expect("Should load syntax definition");
    let mut builder = SyntaxSetBuilder::new();
    builder.add(definition);
    builder.build()
}

const LINES_PER_SNAPSHOT: usize = 50;

/// A syntax highlighter.
#[derive(Debug)]
pub struct Highlighter {
    syntax: &'static SyntaxReference,
    highlighter: highlighting::Highlighter<'static>,
    caches: Vec<(ParseState, ScopeStack)>,
    current_line: usize,
}

impl highlighter::Highlighter for Highlighter {
    type Settings = Settings;
    type Highlight = Highlight;

    type Iterator<'a> = Box<dyn Iterator<Item = (Range<usize>, Self::Highlight)> + 'a>;

    fn new(settings: &Self::Settings) -> Self {
        let syntax = SYNTAX_SET
            .find_syntax_by_extension("wgsl")
            .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

        let highlighter = highlighting::Highlighter::new(&THEMES.themes[settings.theme.key()]);

        let parser = ParseState::new(syntax);
        let stack = ScopeStack::new();

        Highlighter {
            syntax,
            highlighter,
            caches: vec![(parser, stack)],
            current_line: 0,
        }
    }

    fn update(&mut self, new_settings: &Self::Settings) {
        self.highlighter = highlighting::Highlighter::new(&THEMES.themes[new_settings.theme.key()]);
        self.change_line(0);
    }

    fn change_line(&mut self, line: usize) {
        let snapshot = line / LINES_PER_SNAPSHOT;

        if snapshot <= self.caches.len() {
            self.caches.truncate(snapshot);
            self.current_line = snapshot * LINES_PER_SNAPSHOT;
        } else {
            self.caches.truncate(1);
            self.current_line = 0;
        }

        let (parser, stack) = self
            .caches
            .last()
            .cloned()
            .unwrap_or_else(|| (ParseState::new(self.syntax), ScopeStack::new()));

        self.caches.push((parser, stack));
    }

    fn highlight_line(&mut self, line: &str) -> Self::Iterator<'_> {
        if self.current_line / LINES_PER_SNAPSHOT >= self.caches.len() {
            let (parser, stack) = self.caches.last().expect("Caches must not be empty");

            self.caches.push((parser.clone(), stack.clone()));
        }

        self.current_line += 1;

        let (parser, stack) = self.caches.last_mut().expect("Caches must not be empty");

        let ops = parser.parse_line(line, &SYNTAX_SET).unwrap_or_default();

        Box::new(scope_iterator(ops, line, stack, &self.highlighter))
    }

    fn current_line(&self) -> usize {
        self.current_line
    }
}

fn scope_iterator<'a>(
    ops: Vec<(usize, ScopeStackOp)>,
    line: &str,
    stack: &'a mut ScopeStack,
    highlighter: &'a highlighting::Highlighter<'static>,
) -> impl Iterator<Item = (Range<usize>, Highlight)> + 'a {
    ScopeRangeIterator {
        ops,
        line_length: line.len(),
        index: 0,
        last_str_index: 0,
    }
    .filter_map(move |(range, scope)| {
        let _ = stack.apply(&scope);

        if range.is_empty() {
            None
        } else {
            Some((
                range,
                Highlight(highlighter.style_mod_for_stack(&stack.scopes)),
            ))
        }
    })
}

/// The settings of a [`Highlighter`].
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    /// The [`Theme`] of the [`Highlighter`].
    ///
    /// It dictates the color scheme that will be used for highlighting.
    pub theme: Theme,
    /// The extension of the file or the name of the language to highlight.
    ///
    /// The [`Highlighter`] will use the token to automatically determine
    /// the grammar to use for highlighting.
    pub token: String,
}

/// A highlight produced by a [`Highlighter`].
#[derive(Debug)]
pub struct Highlight(highlighting::StyleModifier);

impl Highlight {
    /// Returns the color of this [`Highlight`].
    ///
    /// If `None`, the original text color should be unchanged.
    pub fn color(&self) -> Option<Color> {
        self.0
            .foreground
            .map(|color| Color::from_rgba8(color.r, color.g, color.b, color.a as f32 / 255.0))
    }

    /// Returns the font of this [`Highlight`].
    ///
    /// If `None`, the original font should be unchanged.
    pub fn font(&self) -> Option<Font> {
        self.0.font_style.and_then(|style| {
            let bold = style.contains(highlighting::FontStyle::BOLD);
            let italic = style.contains(highlighting::FontStyle::ITALIC);

            if bold || italic {
                Some(Font {
                    weight: if bold {
                        font::Weight::Bold
                    } else {
                        font::Weight::Normal
                    },
                    style: if italic {
                        font::Style::Italic
                    } else {
                        font::Style::Normal
                    },
                    ..Font::MONOSPACE
                })
            } else {
                None
            }
        })
    }

    /// Returns the [`Format`] of the [`Highlight`].
    ///
    /// It contains both the [`color`] and the [`font`].
    ///
    /// [`color`]: Self::color
    /// [`font`]: Self::font
    pub fn to_format(&self) -> Format<Font> {
        Format {
            color: self.color(),
            font: self.font(),
        }
    }
}

/// A highlighting theme.
#[allow(missing_docs, unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    SolarizedDark,
    Base16Mocha,
    Base16Ocean,
    Base16Eighties,
    InspiredGitHub,
}

impl Theme {
    fn key(self) -> &'static str {
        match self {
            Theme::SolarizedDark => "Solarized (dark)",
            Theme::Base16Mocha => "base16-mocha.dark",
            Theme::Base16Ocean => "base16-ocean.dark",
            Theme::Base16Eighties => "base16-eighties.dark",
            Theme::InspiredGitHub => "InspiredGitHub",
        }
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Theme::SolarizedDark => write!(f, "Solarized Dark"),
            Theme::Base16Mocha => write!(f, "Mocha"),
            Theme::Base16Ocean => write!(f, "Ocean"),
            Theme::Base16Eighties => write!(f, "Eighties"),
            Theme::InspiredGitHub => write!(f, "Inspired GitHub"),
        }
    }
}

struct ScopeRangeIterator {
    ops: Vec<(usize, ScopeStackOp)>,
    line_length: usize,
    index: usize,
    last_str_index: usize,
}

impl Iterator for ScopeRangeIterator {
    type Item = (std::ops::Range<usize>, ScopeStackOp);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index > self.ops.len() {
            return None;
        }

        let next_str_i = if self.index == self.ops.len() {
            self.line_length
        } else {
            self.ops[self.index].0
        };

        let range = self.last_str_index..next_str_i;
        self.last_str_index = next_str_i;

        let op = if self.index == 0 {
            ScopeStackOp::Noop
        } else {
            self.ops[self.index - 1].1.clone()
        };

        self.index += 1;
        Some((range, op))
    }
}
