//! An Ecma-262 Lexer / Tokenizer
//! Prior Arts:
//!     * [jsparagus](https://github.com/mozilla-spidermonkey/jsparagus/blob/24004745a8ed4939fc0dc7332bfd1268ac52285f/crates/parser/src)
//!     * [rome](https://github.com/rome/tools/tree/lsp/v0.28.0/crates/rome_js_parser/src/lexer)
//!     * [rustc](https://github.com/rust-lang/rust/blob/1.82.0/compiler/rustc_lexer/src)
//!     * [v8](https://v8.dev/blog/scanner)

use rustc_hash::FxHashMap;

use oxc_allocator::Allocator;
use oxc_ast::ast::RegExpFlags;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{SourceType, Span};

use crate::{UniquePromise, diagnostics};

mod byte_handlers;
mod comment;
mod gperf_keywords;
mod identifier;
mod jsx;
mod kind;
mod number;
mod numeric;
mod punctuation;
mod regex;
mod search;
mod source;
mod string;
mod template;
mod token;
mod trivia_builder;
mod typescript;
mod unicode;
mod whitespace;

pub use kind::Kind;
pub use number::{parse_big_int, parse_float, parse_int};
pub use token::Token;

use source::{Source, SourcePosition};
use trivia_builder::TriviaBuilder;

#[derive(Debug, Clone, Copy)]
pub struct LexerCheckpoint<'a> {
    /// Current position in source
    position: SourcePosition<'a>,

    token: Token,

    errors_pos: usize,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LexerContext {
    Regular,
    /// Lex the next token, returns `JsxString` or any other token
    JsxAttributeValue,
}

pub struct Lexer<'a> {
    allocator: &'a Allocator,

    // Wrapper around source text. Must not be changed after initialization.
    source: Source<'a>,

    source_type: SourceType,

    token: Token,

    pub(crate) errors: Vec<OxcDiagnostic>,

    context: LexerContext,

    pub(crate) trivia_builder: TriviaBuilder,

    /// Data store for escaped strings, indexed by [Token::start] when [Token::escaped] is true
    pub escaped_strings: FxHashMap<u32, &'a str>,

    /// Data store for escaped templates, indexed by [Token::start] when [Token::escaped] is true
    /// `None` is saved when the string contains an invalid escape sequence.
    pub escaped_templates: FxHashMap<u32, Option<&'a str>>,

    /// `memchr` Finder for end of multi-line comments. Created lazily when first used.
    multi_line_comment_end_finder: Option<memchr::memmem::Finder<'static>>,
}

impl<'a> Lexer<'a> {
    /// Create new `Lexer`.
    ///
    /// Requiring a `UniquePromise` to be provided guarantees only 1 `Lexer` can exist
    /// on a single thread at one time.
    pub(super) fn new(
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: SourceType,
        unique: UniquePromise,
    ) -> Self {
        let source = Source::new(source_text, unique);

        // The first token is at the start of file, so is allows on a new line
        let token = Token::new_on_new_line();
        Self {
            allocator,
            source,
            source_type,
            token,
            errors: vec![],
            context: LexerContext::Regular,
            trivia_builder: TriviaBuilder::default(),
            escaped_strings: FxHashMap::default(),
            escaped_templates: FxHashMap::default(),
            multi_line_comment_end_finder: None,
        }
    }

    /// Backdoor to create a `Lexer` without holding a `UniquePromise`, for benchmarks.
    /// This function must NOT be exposed in public API as it breaks safety invariants.
    #[cfg(feature = "benchmarking")]
    pub fn new_for_benchmarks(
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: SourceType,
    ) -> Self {
        let unique = UniquePromise::new_for_tests_and_benchmarks();
        Self::new(allocator, source_text, source_type, unique)
    }

    /// Get errors.
    /// Only used in benchmarks.
    #[cfg(feature = "benchmarking")]
    pub fn errors(&self) -> &[OxcDiagnostic] {
        &self.errors
    }

    /// Remaining string from `Source`
    pub fn remaining(&self) -> &'a str {
        self.source.remaining()
    }

    /// Creates a checkpoint storing the current lexer state.
    /// Use `rewind` to restore the lexer to the state stored in the checkpoint.
    pub fn checkpoint(&self) -> LexerCheckpoint<'a> {
        LexerCheckpoint {
            position: self.source.position(),
            token: self.token,
            errors_pos: self.errors.len(),
        }
    }

    /// Rewinds the lexer to the same state as when the passed in `checkpoint` was created.
    pub fn rewind(&mut self, checkpoint: LexerCheckpoint<'a>) {
        self.errors.truncate(checkpoint.errors_pos);
        self.source.set_position(checkpoint.position);
        self.token = checkpoint.token;
    }

    pub fn peek_token(&mut self) -> Token {
        let checkpoint = self.checkpoint();
        let token = self.next_token();
        self.rewind(checkpoint);
        token
    }

    /// Set context
    pub fn set_context(&mut self, context: LexerContext) {
        self.context = context;
    }

    /// Main entry point
    pub fn next_token(&mut self) -> Token {
        let kind = self.read_next_token();
        self.finish_next(kind)
    }

    fn finish_next(&mut self, kind: Kind) -> Token {
        self.token.set_kind(kind);
        self.token.set_end(self.offset());
        let token = self.token;
        self.trivia_builder.handle_token(token);
        self.token = Token::default();
        token
    }

    /// Advance source cursor to end of file.
    #[inline]
    pub fn advance_to_end(&mut self) {
        self.source.advance_to_end();
    }

    // ---------- Private Methods ---------- //
    fn error(&mut self, error: OxcDiagnostic) {
        self.errors.push(error);
    }

    /// Get the length offset from the source, in UTF-8 bytes
    #[inline]
    fn offset(&self) -> u32 {
        self.source.offset()
    }

    /// Get the current unterminated token range
    fn unterminated_range(&self) -> Span {
        Span::new(self.token.start(), self.offset())
    }

    /// Consume the current char if not at EOF
    #[inline]
    fn next_char(&mut self) -> Option<char> {
        self.source.next_char()
    }

    /// Consume the current char
    #[inline]
    fn consume_char(&mut self) -> char {
        self.source.next_char().unwrap()
    }

    /// Consume the current char and the next if not at EOF
    #[inline]
    fn next_2_chars(&mut self) -> Option<[char; 2]> {
        self.source.next_2_chars()
    }

    /// Consume the current char and the next
    #[inline]
    fn consume_2_chars(&mut self) -> [char; 2] {
        self.next_2_chars().unwrap()
    }

    /// Peek the next byte without advancing the position
    #[inline]
    fn peek_byte(&self) -> Option<u8> {
        self.source.peek_byte()
    }

    /// Peek the next two bytes without advancing the position
    #[inline]
    fn peek_2_bytes(&self) -> Option<[u8; 2]> {
        self.source.peek_2_bytes()
    }

    /// Peek the next char without advancing the position
    #[inline]
    fn peek_char(&self) -> Option<char> {
        self.source.peek_char()
    }

    /// Peek the next byte, and advance the current position if it matches
    /// the given ASCII char.
    // `#[inline(always)]` to make sure the `assert!` gets optimized out.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn next_ascii_byte_eq(&mut self, b: u8) -> bool {
        // TODO: can be replaced by `std::ascii:Char` once stabilized.
        // https://github.com/rust-lang/rust/issues/110998
        assert!(b.is_ascii());
        // SAFETY: `b` is a valid ASCII char.
        unsafe { self.source.advance_if_ascii_eq(b) }
    }

    fn current_offset(&self) -> Span {
        Span::empty(self.offset())
    }

    /// Return `IllegalCharacter` Error or `UnexpectedEnd` if EOF
    fn unexpected_err(&mut self) {
        let offset = self.current_offset();
        match self.peek_char() {
            Some(c) => self.error(diagnostics::invalid_character(c, offset)),
            None => self.error(diagnostics::unexpected_end(offset)),
        }
    }

    /// Read each char and set the current token
    /// Whitespace and line terminators are skipped
    fn read_next_token(&mut self) -> Kind {
        self.trivia_builder.has_pure_comment = false;
        self.trivia_builder.has_no_side_effects_comment = false;
        loop {
            let offset = self.offset();
            self.token.set_start(offset);

            let Some(byte) = self.peek_byte() else {
                return Kind::Eof;
            };

            // SAFETY: `byte` is byte value at current position in source
            let kind = unsafe { self.handle_byte(byte) };
            if kind != Kind::Skip {
                return kind;
            }
        }
    }
}

/// Call a closure while hinting to compiler that this branch is rarely taken.
///
/// "Cold trampoline function", suggested in:
/// <https://users.rust-lang.org/t/is-cold-the-only-reliable-way-to-hint-to-branch-predictor/106509/2>
#[cold]
pub fn cold_branch<F: FnOnce() -> T, T>(f: F) -> T {
    f()
}
