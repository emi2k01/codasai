use once_cell::unsync::Lazy;
use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::html;

static SYNTAX_SET_DUMP_BIN: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/syntaxset.packdump"));

thread_local! {
    static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(|| {
        syntect::dumps::from_binary(SYNTAX_SET_DUMP_BIN)
    });
}

pub const CLASS_STYLE: ClassStyle = ClassStyle::SpacedPrefixed {
    prefix: "csai-code-",
};

/// Escapes and highlights `code` using `ext` as the file extension to select the language's syntax.
///
/// If the syntax highlighting doesn't support the file extension, the returned string is only
/// escaped.
pub fn escape_and_highlight(code: &str, ext: &str) -> String {
    SYNTAX_SET.with(|ss| -> String {
        if let Some(syntax) = ss.find_syntax_by_extension(ext) {
            // SAFETY: syntect already escapes `code` so we don't escape it beforehand.
            // If another library is used in the future, make sure `code` is
            // escaped in this branch.
            let mut html_generator =
                ClassedHTMLGenerator::new_with_class_style(syntax, ss, CLASS_STYLE);

            for line in LinesWithEndings::from(code) {
                html_generator.parse_html_for_line_which_includes_newline(line);
            }

            html_generator.finalize()
        } else {
            html::escape(code)
        }
    })
}
