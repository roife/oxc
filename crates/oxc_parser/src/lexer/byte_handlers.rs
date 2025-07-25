use crate::diagnostics;

use super::{Kind, Lexer, gperf_keywords};

impl Lexer<'_> {
    /// Handle next byte of source.
    ///
    /// # SAFETY
    ///
    /// * Lexer must not be at end of file.
    /// * `byte` must be next byte of source code, corresponding to current position of `lexer.source`.
    /// * Only `BYTE_HANDLERS` for ASCII characters may use the `ascii_byte_handler!()` macro.
    pub(super) unsafe fn handle_byte(&mut self, byte: u8) -> Kind {
        // SAFETY: Caller guarantees to uphold safety invariants
        unsafe { BYTE_HANDLERS[byte as usize](self) }
    }
}

type ByteHandler = unsafe fn(&mut Lexer<'_>) -> Kind;

/// Lookup table mapping any incoming byte to a handler function defined below.
/// <https://github.com/ratel-rust/ratel-core/blob/v0.7.0/ratel/src/lexer/mod.rs>
#[rustfmt::skip]
static BYTE_HANDLERS: [ByteHandler; 256] = [
//  0    1    2    3    4    5    6    7    8    9    A    B    C    D    E    F    //
    ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, SPS, LIN, ISP, ISP, LIN, ERR, ERR, // 0
    ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, // 1
    SPS, EXL, QOD, HAS, IDT, PRC, AMP, QOS, PNO, PNC, ATR, PLS, COM, MIN, PRD, SLH, // 2
    ZER, DIG, DIG, DIG, DIG, DIG, DIG, DIG, DIG, DIG, COL, SEM, LSS, EQL, GTR, QST, // 3
    AT_, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, // 4
    IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, BTO, ESC, BTC, CRT, IDT, // 5
    TPL, UNI_IDT, UNI_IDT, UNI_IDT, UNI_IDT, UNI_IDT, UNI_IDT, UNI_IDT, IDT, UNI_IDT, IDT, UNI_IDT, UNI_IDT, UNI_IDT, UNI_IDT, UNI_IDT, // 6
    UNI_IDT, IDT, UNI_IDT, UNI_IDT, UNI_IDT, UNI_IDT, UNI_IDT, UNI_IDT, IDT, UNI_IDT, IDT, BEO, PIP, BEC, TLD, ERR, // 7
    UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, // 8
    UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, // 9
    UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, // A
    UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, // B
    UER, UER, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // C
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // D
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // E
    UNI, UNI, UNI, UNI, UNI, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, UER, // F
];

/// Macro for defining a byte handler.
///
/// Use `ascii_byte_handler!` macro for ASCII characters, which adds optimizations for ASCII.
///
/// Handlers are defined as functions instead of closures, so they have names in flame graphs.
///
/// ```
/// byte_handler!(UNI(lexer) {
///   lexer.unicode_char_handler()
/// });
/// ```
///
/// expands to:
///
/// ```
/// const UNI: ByteHandler = {
///   #[expect(non_snake_case)]
///   fn UNI(lexer: &mut Lexer) -> Kind {
///     lexer.unicode_char_handler()
///   }
///   UNI
/// };
/// ```
macro_rules! byte_handler {
    ($id:ident($lex:ident) $body:expr) => {
        const $id: ByteHandler = {
            #[expect(non_snake_case)]
            fn $id($lex: &mut Lexer) -> Kind {
                $body
            }
            $id
        };
    };
}

/// Macro for defining byte handler for an ASCII character.
///
/// In addition to defining a `const` for the handler, it also asserts that lexer
/// is not at end of file, and that next char is ASCII.
/// Where the handler is for an ASCII character, these assertions are self-evidently true.
///
/// These assertions produce no runtime code, but hint to the compiler that it can assume that
/// next char is ASCII, and it uses that information to optimize the rest of the handler.
/// e.g. `lexer.consume_char()` becomes just a single assembler instruction.
/// Without the assertions, the compiler is unable to deduce the next char is ASCII, due to
/// the indirection of the `BYTE_HANDLERS` jump table.
///
/// These assertions are unchecked (i.e. won't panic) and will cause UB if they're incorrect.
///
/// # SAFETY
/// Only use this macro to define byte handlers for ASCII characters.
///
/// ```
/// ascii_byte_handler!(SPS(lexer) {
///   lexer.consume_char();
///   Kind::WhiteSpace
/// });
/// ```
///
/// expands to:
///
/// ```
/// const SPS: ByteHandler = {
///   #[expect(non_snake_case)]
///   fn SPS(lexer: &mut Lexer) {
///     // SAFETY: This macro is only used for ASCII characters
///     unsafe {
///       use oxc_data_structures::assert_unchecked;
///       assert_unchecked!(!lexer.source.is_eof());
///       assert_unchecked!(lexer.source.peek_byte_unchecked() < 128);
///     }
///     {
///       lexer.consume_char();
///       Kind::WhiteSpace
///     }
///   }
///   SPS
/// };
/// ```
macro_rules! ascii_byte_handler {
    ($id:ident($lex:ident) $body:expr) => {
        byte_handler!($id($lex) {
            // SAFETY: This macro is only used for ASCII characters
            unsafe {
                use oxc_data_structures::assert_unchecked;
                assert_unchecked!(!$lex.source.is_eof());
                assert_unchecked!($lex.source.peek_byte_unchecked() < 128);
            }
            $body
        });
    };
}

/// Macro for defining byte handler for an ASCII character which is start of an identifier
/// (`a`-`z`, `A`-`Z`, `$` or `_`).
///
/// Macro calls `Lexer::identifier_name_handler` to get the text of the identifier,
/// minus its first character.
///
/// `Lexer::identifier_name_handler` is an unsafe function, but if byte being consumed is ASCII,
/// its requirements are met.
///
/// # SAFETY
/// Only use this macro to define byte handlers for ASCII characters.
///
/// ```
/// ascii_identifier_handler!(L_G(id_without_first_char) match id_without_first_char {
///   "et" => Kind::Get,
///   "lobal" => Kind::Global,
///   _ => Kind::Ident,
/// });
/// ```
///
/// expands to:
///
/// ```
/// const L_G: ByteHandler = {
///   #[expect(non_snake_case)]
///   fn L_G(lexer: &mut Lexer) -> Kind {
///     // SAFETY: This macro is only used for ASCII characters
///     let id_without_first_char = unsafe { lexer.identifier_name_handler() };
///     match id_without_first_char {
///       "et" => Kind::Get,
///       "lobal" => Kind::Global,
///       _ => Kind::Ident,
///     }
///   }
///   L_G
/// };
/// ```
macro_rules! ascii_identifier_handler {
    ($id:ident($str:ident) $body:expr) => {
        byte_handler!($id(lexer) {
            // SAFETY: This macro is only used for ASCII characters
            let $str = unsafe { lexer.identifier_name_handler() };
            $body
        });
    };
}

// `\0` `\1` etc
ascii_byte_handler!(ERR(lexer) {
    let c = lexer.consume_char();
    lexer.error(diagnostics::invalid_character(c, lexer.unterminated_range()));
    Kind::Undetermined
});

// <SPACE> <TAB> Normal Whitespace
ascii_byte_handler!(SPS(lexer) {
    lexer.consume_char();
    Kind::Skip
});

// <VT> <FF> Irregular Whitespace
ascii_byte_handler!(ISP(lexer) {
    lexer.consume_char();
    lexer.trivia_builder.add_irregular_whitespace(lexer.token.start(), lexer.offset());
    Kind::Skip
});

// '\r' '\n'
ascii_byte_handler!(LIN(lexer) {
    lexer.consume_char();
    lexer.line_break_handler()
});

// !
ascii_byte_handler!(EXL(lexer) {
    lexer.consume_char();
    if let Some(next_2_bytes) = lexer.peek_2_bytes() {
        match next_2_bytes[0] {
            b'=' => {
                if next_2_bytes[1] == b'=' {
                    lexer.consume_2_chars();
                    Kind::Neq2
                } else {
                    lexer.consume_char();
                    Kind::Neq
                }
            }
            _ => Kind::Bang
        }
    } else {
        // At EOF, or only 1 byte left
        match lexer.peek_byte() {
            Some(b'=') => {
                lexer.consume_char();
                Kind::Neq
            }
            _ => Kind::Bang
        }
    }
});

// "
ascii_byte_handler!(QOD(lexer) {
    // SAFETY: This function is only called for `"`
    unsafe { lexer.read_string_literal_double_quote() }
});

// '
ascii_byte_handler!(QOS(lexer) {
    // SAFETY: This function is only called for `'`
    unsafe { lexer.read_string_literal_single_quote() }
});

// #
ascii_byte_handler!(HAS(lexer) {
    lexer.consume_char();
    // HashbangComment ::
    //     `#!` SingleLineCommentChars?
    if lexer.token.start() == 0 && lexer.next_ascii_byte_eq(b'!') {
        lexer.read_hashbang_comment()
    } else {
        lexer.private_identifier()
    }
});

// `A..=Z`, `a..=z` (except special cases below), `_`, `$`
ascii_identifier_handler!(IDT(_id_without_first_char) {
    Kind::Ident
});

// Unified identifier handler using gperf perfect hash function
byte_handler!(UNI_IDT(lexer) {
    // SAFETY: This function is only called for ASCII lowercase letters
    let identifier = unsafe { lexer.identifier_name_handler() };
    gperf_keywords::lookup_keyword(identifier)
});

// %
ascii_byte_handler!(PRC(lexer) {
    lexer.consume_char();
    if lexer.next_ascii_byte_eq(b'=') {
        Kind::PercentEq
    } else {
        Kind::Percent
    }
});

// &
ascii_byte_handler!(AMP(lexer) {
    lexer.consume_char();
    if lexer.next_ascii_byte_eq(b'&') {
        if lexer.next_ascii_byte_eq(b'=') {
            Kind::Amp2Eq
        } else {
            Kind::Amp2
        }
    } else if lexer.next_ascii_byte_eq(b'=') {
        Kind::AmpEq
    } else {
        Kind::Amp
    }
});

// (
ascii_byte_handler!(PNO(lexer) {
    lexer.consume_char();
    Kind::LParen
});

// )
ascii_byte_handler!(PNC(lexer) {
    lexer.consume_char();
    Kind::RParen
});

// *
ascii_byte_handler!(ATR(lexer) {
    lexer.consume_char();
    if lexer.next_ascii_byte_eq(b'*') {
        if lexer.next_ascii_byte_eq(b'=') {
            Kind::Star2Eq
        } else {
            Kind::Star2
        }
    } else if lexer.next_ascii_byte_eq(b'=') {
        Kind::StarEq
    } else {
        Kind::Star
    }
});

// +
ascii_byte_handler!(PLS(lexer) {
    lexer.consume_char();
    if lexer.next_ascii_byte_eq(b'+') {
        Kind::Plus2
    } else if lexer.next_ascii_byte_eq(b'=') {
        Kind::PlusEq
    } else {
        Kind::Plus
    }
});

// ,
ascii_byte_handler!(COM(lexer) {
    lexer.consume_char();
    Kind::Comma
});

// -
ascii_byte_handler!(MIN(lexer) {
    lexer.consume_char();
    lexer.read_minus().unwrap_or_else(|| lexer.skip_single_line_comment())
});

// .
ascii_byte_handler!(PRD(lexer) {
    lexer.consume_char();
    lexer.read_dot()
});

// /
ascii_byte_handler!(SLH(lexer) {
    lexer.consume_char();
    match lexer.peek_byte() {
        Some(b'/') => {
            lexer.consume_char();
            lexer.skip_single_line_comment()
        }
        Some(b'*') => {
            lexer.consume_char();
            lexer.skip_multi_line_comment()
        }
        _ => {
            // regex is handled separately, see `next_regex`
            if lexer.next_ascii_byte_eq(b'=') {
                Kind::SlashEq
            } else {
                Kind::Slash
            }
        }
    }
});

// 0
ascii_byte_handler!(ZER(lexer) {
    lexer.consume_char();
    lexer.read_zero()
});

// 1 to 9
ascii_byte_handler!(DIG(lexer) {
    lexer.consume_char();
    lexer.decimal_literal_after_first_digit()
});

// :
ascii_byte_handler!(COL(lexer) {
    lexer.consume_char();
    Kind::Colon
});

// ;
ascii_byte_handler!(SEM(lexer) {
    lexer.consume_char();
    Kind::Semicolon
});

// <
ascii_byte_handler!(LSS(lexer) {
    lexer.consume_char();
    lexer.read_left_angle().unwrap_or_else(|| lexer.skip_single_line_comment())
});

// =
ascii_byte_handler!(EQL(lexer) {
    lexer.consume_char();
    if lexer.next_ascii_byte_eq(b'=') {
        if lexer.next_ascii_byte_eq(b'=') {
            Kind::Eq3
        } else {
            Kind::Eq2
        }
    } else if lexer.next_ascii_byte_eq(b'>') {
        Kind::Arrow
    } else {
        Kind::Eq
    }
});

// >
ascii_byte_handler!(GTR(lexer) {
    lexer.consume_char();
    // `>=` is re-lexed with [Lexer::next_jsx_child]
    Kind::RAngle
});

// ?
ascii_byte_handler!(QST(lexer) {
    lexer.consume_char();

    if let Some(next_2_bytes) = lexer.peek_2_bytes() {
        match next_2_bytes[0] {
            b'?' => {
                if next_2_bytes[1] == b'=' {
                    lexer.consume_2_chars();
                    Kind::Question2Eq
                } else {
                    lexer.consume_char();
                    Kind::Question2
                }
            }
            // parse `?.1` as `?` `.1`
            b'.' if !next_2_bytes[1].is_ascii_digit() => {
                lexer.consume_char();
                Kind::QuestionDot
            }
            _ => Kind::Question,
        }
    } else {
        // At EOF, or only 1 byte left
        match lexer.peek_byte() {
            Some(b'?') => {
                lexer.consume_char();
                Kind::Question2
            }
            Some(b'.') => {
                lexer.consume_char();
                Kind::QuestionDot
            }
            _ => Kind::Question,
        }
    }
});

// @
ascii_byte_handler!(AT_(lexer) {
    lexer.consume_char();
    Kind::At
});

// [
ascii_byte_handler!(BTO(lexer) {
    lexer.consume_char();
    Kind::LBrack
});

// \
ascii_byte_handler!(ESC(lexer) {
    lexer.identifier_backslash_handler()
});

// ]
ascii_byte_handler!(BTC(lexer) {
    lexer.consume_char();
    Kind::RBrack
});

// ^
ascii_byte_handler!(CRT(lexer) {
    lexer.consume_char();
    if lexer.next_ascii_byte_eq(b'=') {
        Kind::CaretEq
    } else {
        Kind::Caret
    }
});

// `
ascii_byte_handler!(TPL(lexer) {
    lexer.consume_char();
    lexer.read_template_literal(Kind::TemplateHead, Kind::NoSubstitutionTemplate)
});

// {
ascii_byte_handler!(BEO(lexer) {
    lexer.consume_char();
    Kind::LCurly
});

// |
ascii_byte_handler!(PIP(lexer) {
    lexer.consume_char();

    match lexer.peek_byte() {
        Some(b'|') => {
            lexer.consume_char();
            if lexer.next_ascii_byte_eq(b'=') {
                Kind::Pipe2Eq
            } else {
                Kind::Pipe2
            }
        }
        Some(b'=') => {
            lexer.consume_char();
            Kind::PipeEq
        }
        _ => Kind::Pipe
    }
});

// }
ascii_byte_handler!(BEC(lexer) {
    lexer.consume_char();
    Kind::RCurly
});

// ~
ascii_byte_handler!(TLD(lexer) {
    lexer.consume_char();
    Kind::Tilde
});

// All L_A to L_Y handlers have been replaced by the unified UNI_IDT handler using gperf perfect hash function

// Non-ASCII characters.
// NB: Must not use `ascii_byte_handler!` macro, as this handler is for non-ASCII chars.
byte_handler!(UNI(lexer) {
    lexer.unicode_char_handler()
});

// UTF-8 continuation bytes (0x80 - 0xBF) (i.e. middle of a multi-byte UTF-8 sequence)
// + and byte values which are not legal in UTF-8 strings (0xC0, 0xC1, 0xF5 - 0xFF).
// `handle_byte()` should only be called with 1st byte of a valid UTF-8 character,
// so something has gone wrong if we get here.
// https://datatracker.ietf.org/doc/html/rfc3629
// NB: Must not use `ascii_byte_handler!` macro, as this handler is for non-ASCII bytes.
byte_handler!(UER(_lexer) {
    unreachable!();
});
