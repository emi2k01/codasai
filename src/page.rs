use pulldown_cmark::Parser;

pub fn to_html(markdown: &str) -> String {
    let parser = markdown_parser(&markdown);
    let mut page_html_unsafe = String::new();
    pulldown_cmark::html::push_html(&mut page_html_unsafe, parser);
    ammonia::clean(&page_html_unsafe)
}

pub fn markdown_parser(markdown: &str) -> Parser {
    let options = pulldown_cmark::Options::all();
    Parser::new_ext(markdown, options)
}

pub fn extract_title(page: &str) -> String {
    use pulldown_cmark::{Event, Tag};

    let parser = markdown_parser(page);
    let mut in_heading = false;
    for event in parser {
        match event {
            Event::Start(Tag::Heading(_)) => in_heading = true,
            Event::End(Tag::Heading(_)) => in_heading = false,
            Event::Text(text) => {
                if in_heading {
                    return text.to_string();
                }
            },
            _ => {},
        }
    }

    return String::from("Untitled");
}
