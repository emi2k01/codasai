use once_cell::unsync::Lazy;
use syntect::highlighting::ThemeSet;
use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::html;

thread_local! {
    static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(|| {
        SyntaxSet::load_defaults_newlines()
    });
    static THEME_SET: Lazy<ThemeSet> = Lazy::new(|| {
        ThemeSet::load_defaults()
    });
}

pub fn escape_and_highlight(code: &str, ext: &str) -> String {
    SYNTAX_SET.with(|ss| -> String {
        if let Some(syntax) = ss.find_syntax_by_extension(ext) {
            // SAFETY: syntect already escapes `code` so we don't escape it beforehand.
            // If another library is used in the future, make sure `code` is
            // escaped in this branch.
            let mut html_generator =
                ClassedHTMLGenerator::new_with_class_style(syntax, ss, ClassStyle::Spaced);

            for line in LinesWithEndings::from(code) {
                html_generator.parse_html_for_line_which_includes_newline(line);
            }

            html_generator.finalize()
        } else {
            html::escape(code)
        }
    })
}
