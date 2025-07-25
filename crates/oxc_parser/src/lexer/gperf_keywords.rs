use super::Kind;

/// Perfect hash function for JavaScript/TypeScript keyword lookup
/// Generated from gperf with modifications for Rust

const MIN_WORD_LENGTH: usize = 2;
const MAX_WORD_LENGTH: usize = 11;
const MAX_HASH_VALUE: usize = 159;

// Association values table for the perfect hash function
static ASSO_VALUES: [u8; 256] = [
    160, 160, 160, 160, 160, 160, 160, 160, 160, 160,
    160, 160, 160, 160, 160, 160, 160, 160, 160, 160,
    160, 160, 160, 160, 160, 160, 160, 160, 160, 160,
    160, 160, 160, 160, 160, 160, 160, 160, 160, 160,
    160, 160, 160, 160, 160, 160, 160, 160, 160, 160,
    160, 160, 160, 160, 160, 160, 160, 160, 160, 160,
    160, 160, 160, 160, 160, 160, 160, 160, 160, 160,
    160, 160, 160, 160, 160, 160, 160, 160, 160, 160,
    160, 160, 160, 160, 160, 160, 160, 160, 160, 160,
    160, 160, 160, 160, 160, 160, 160,  10,  75,   0,
     20,   5,  35,  70, 115,  40, 160,   0,  40,  60,
     15,  30, 110, 160,   0,   5,   0,  30,  60,   0,
     60,  25, 160, 160, 160, 160, 160, 160, 160, 160,
    160, 160, 160, 160, 160, 160, 160, 160, 160, 160,
    160, 160, 160, 160, 160, 160, 160, 160, 160, 160,
    160, 160, 160, 160, 160, 160, 160, 160, 160, 160,
    160, 160, 160, 160, 160, 160, 160, 160, 160, 160,
    160, 160, 160, 160, 160, 160, 160, 160, 160, 160,
    160, 160, 160, 160, 160, 160, 160, 160, 160, 160,
    160, 160, 160, 160, 160, 160, 160, 160, 160, 160,
    160, 160, 160, 160, 160, 160, 160, 160, 160, 160,
    160, 160, 160, 160, 160, 160, 160, 160, 160, 160,
    160, 160, 160, 160, 160, 160, 160, 160, 160, 160,
    160, 160, 160, 160, 160, 160, 160, 160, 160, 160,
    160, 160, 160, 160, 160, 160, 160, 160, 160, 160,
    160, 160, 160, 160, 160, 160
];

/// Keyword lookup table entry
struct KeywordEntry {
    name: &'static str,
    kind: Kind,
}

/// Pre-computed keyword lookup table organized by hash value
static KEYWORD_TABLE: [Option<KeywordEntry>; 160] = [
    None, None, None, None, None, None, None, None, None,
    Some(KeywordEntry { name: "true", kind: Kind::True }),
    None,
    Some(KeywordEntry { name: "static", kind: Kind::Static }),
    None,
    Some(KeywordEntry { name: "set", kind: Kind::Set }),
    None,
    Some(KeywordEntry { name: "await", kind: Kind::Await }),
    Some(KeywordEntry { name: "target", kind: Kind::Target }),
    Some(KeywordEntry { name: "require", kind: Kind::Require }),
    Some(KeywordEntry { name: "accessor", kind: Kind::Accessor }),
    Some(KeywordEntry { name: "case", kind: Kind::Case }),
    Some(KeywordEntry { name: "async", kind: Kind::Async }),
    Some(KeywordEntry { name: "assert", kind: Kind::Assert }),
    Some(KeywordEntry { name: "as", kind: Kind::As }),
    Some(KeywordEntry { name: "new", kind: Kind::New }),
    None,
    Some(KeywordEntry { name: "never", kind: Kind::Never }),
    Some(KeywordEntry { name: "return", kind: Kind::Return }),
    Some(KeywordEntry { name: "asserts", kind: Kind::Asserts }),
    Some(KeywordEntry { name: "try", kind: Kind::Try }),
    Some(KeywordEntry { name: "satisfies", kind: Kind::Satisfies }),
    Some(KeywordEntry { name: "defer", kind: Kind::Defer }),
    None,
    Some(KeywordEntry { name: "default", kind: Kind::Default }),
    Some(KeywordEntry { name: "debugger", kind: Kind::Debugger }),
    Some(KeywordEntry { name: "type", kind: Kind::Type }),
    Some(KeywordEntry { name: "const", kind: Kind::Const }),
    Some(KeywordEntry { name: "delete", kind: Kind::Delete }),
    Some(KeywordEntry { name: "declare", kind: Kind::Declare }),
    Some(KeywordEntry { name: "readonly", kind: Kind::Readonly }),
    Some(KeywordEntry { name: "namespace", kind: Kind::Namespace }),
    Some(KeywordEntry { name: "super", kind: Kind::Super }),
    Some(KeywordEntry { name: "constructor", kind: Kind::Constructor }),
    None,
    Some(KeywordEntry { name: "continue", kind: Kind::Continue }),
    None,
    Some(KeywordEntry { name: "keyof", kind: Kind::KeyOf }),
    Some(KeywordEntry { name: "source", kind: Kind::Source }),
    None,
    Some(KeywordEntry { name: "let", kind: Kind::Let }),
    None,
    Some(KeywordEntry { name: "class", kind: Kind::Class }),
    Some(KeywordEntry { name: "number", kind: Kind::Number }),
    Some(KeywordEntry { name: "is", kind: Kind::Is }),
    Some(KeywordEntry { name: "any", kind: Kind::Any }),
    Some(KeywordEntry { name: "else", kind: Kind::Else }),
    Some(KeywordEntry { name: "false", kind: Kind::False }),
    Some(KeywordEntry { name: "unique", kind: Kind::Unique }),
    None, None, None,
    Some(KeywordEntry { name: "infer", kind: Kind::Infer }),
    None, None,
    Some(KeywordEntry { name: "out", kind: Kind::Out }),
    Some(KeywordEntry { name: "intrinsic", kind: Kind::Intrinsic }),
    None,
    Some(KeywordEntry { name: "typeof", kind: Kind::Typeof }),
    Some(KeywordEntry { name: "unknown", kind: Kind::Unknown }),
    Some(KeywordEntry { name: "for", kind: Kind::For }),
    Some(KeywordEntry { name: "interface", kind: Kind::Interface }),
    None,
    Some(KeywordEntry { name: "export", kind: Kind::Export }),
    Some(KeywordEntry { name: "in", kind: Kind::In }),
    Some(KeywordEntry { name: "var", kind: Kind::Var }),
    Some(KeywordEntry { name: "undefined", kind: Kind::Undefined }),
    None,
    Some(KeywordEntry { name: "symbol", kind: Kind::Symbol }),
    Some(KeywordEntry { name: "extends", kind: Kind::Extends }),
    Some(KeywordEntry { name: "get", kind: Kind::Get }),
    Some(KeywordEntry { name: "meta", kind: Kind::Meta }),
    Some(KeywordEntry { name: "break", kind: Kind::Break }),
    Some(KeywordEntry { name: "string", kind: Kind::String }),
    Some(KeywordEntry { name: "do", kind: Kind::Do }),
    None,
    Some(KeywordEntry { name: "enum", kind: Kind::Enum }),
    None, None, None,
    Some(KeywordEntry { name: "function", kind: Kind::Function }),
    Some(KeywordEntry { name: "null", kind: Kind::Null }),
    Some(KeywordEntry { name: "yield", kind: Kind::Yield }),
    None, None,
    Some(KeywordEntry { name: "abstract", kind: Kind::Abstract }),
    None, None, None, None, None,
    Some(KeywordEntry { name: "from", kind: Kind::From }),
    Some(KeywordEntry { name: "instanceof", kind: Kind::Instanceof }),
    Some(KeywordEntry { name: "module", kind: Kind::Module }),
    Some(KeywordEntry { name: "of", kind: Kind::Of }),
    Some(KeywordEntry { name: "override", kind: Kind::Override }),
    None, None,
    Some(KeywordEntry { name: "import", kind: Kind::Import }),
    Some(KeywordEntry { name: "finally", kind: Kind::Finally }),
    None, None,
    Some(KeywordEntry { name: "using", kind: Kind::Using }),
    Some(KeywordEntry { name: "object", kind: Kind::Object }),
    Some(KeywordEntry { name: "if", kind: Kind::If }),
    None,
    Some(KeywordEntry { name: "void", kind: Kind::Void }),
    Some(KeywordEntry { name: "implements", kind: Kind::Implements }),
    None, None, None, None,
    Some(KeywordEntry { name: "throw", kind: Kind::Throw }),
    Some(KeywordEntry { name: "bigint", kind: Kind::BigInt }),
    Some(KeywordEntry { name: "private", kind: Kind::Private }),
    None,
    Some(KeywordEntry { name: "this", kind: Kind::This }),
    Some(KeywordEntry { name: "while", kind: Kind::While }),
    Some(KeywordEntry { name: "switch", kind: Kind::Switch }),
    Some(KeywordEntry { name: "boolean", kind: Kind::Boolean }),
    None, None,
    Some(KeywordEntry { name: "catch", kind: Kind::Catch }),
    None,
    Some(KeywordEntry { name: "package", kind: Kind::Package }),
    None, None, None, None, None, None,
    Some(KeywordEntry { name: "protected", kind: Kind::Protected }),
    None, None, None, None, None, None,
    Some(KeywordEntry { name: "public", kind: Kind::Public }),
    None, None, None, None, None, None, None, None, None,
    Some(KeywordEntry { name: "global", kind: Kind::Global }),
    None, None,
    Some(KeywordEntry { name: "with", kind: Kind::With }),
];

/// Hash function for keyword lookup
#[inline]
fn hash_keyword(s: &str) -> usize {
    let bytes = s.as_bytes();
    let len = bytes.len();
    if len == 0 {
        return 0;
    }

    let first = ASSO_VALUES[bytes[0] as usize] as usize;
    let second = if len > 1 { ASSO_VALUES[bytes[1] as usize] as usize } else { 0 };
    let last = ASSO_VALUES[bytes[len - 1] as usize] as usize;

    len + first + second + last
}

/// Look up a keyword using the perfect hash function
pub fn lookup_keyword(s: &str) -> Kind {
    let len = s.len();

    if len < MIN_WORD_LENGTH || len > MAX_WORD_LENGTH {
        return Kind::Ident;
    }

    let key = hash_keyword(s);

    if key > MAX_HASH_VALUE {
        return Kind::Ident;
    }

    if let Some(entry) = &KEYWORD_TABLE[key] {
        if entry.name == s {
            return entry.kind;
        }
    }

    Kind::Ident
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_lookup() {
        assert_eq!(lookup_keyword("await"), Kind::Await);
        assert_eq!(lookup_keyword("async"), Kind::Async);
        assert_eq!(lookup_keyword("function"), Kind::Function);
        assert_eq!(lookup_keyword("class"), Kind::Class);
        assert_eq!(lookup_keyword("interface"), Kind::Interface);
        assert_eq!(lookup_keyword("not_a_keyword"), Kind::Ident);
        assert_eq!(lookup_keyword(""), Kind::Ident);
        assert_eq!(lookup_keyword("verylongidentifiername"), Kind::Ident);
    }
}
