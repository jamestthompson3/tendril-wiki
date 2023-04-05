use crate::parsers::{ParsedPages, TemplattedPage};

pub mod tags;

pub async fn update_templatted_pages(page: TemplattedPage, pages: ParsedPages) {
    let mut tempatted_pages = pages.lock().await;
    tempatted_pages.push(page);
}

const FORBIDDEN_TAGS: [&str; 5] = ["noscript", "script", "object", "embed", "link"];

pub fn sanitize_html(html: &str) -> String {
    let mut sanitized = String::from(html);
    for tag in FORBIDDEN_TAGS {
        if sanitized.contains(tag) {
            sanitized = sanitized
                .replace(&format!("<{}>", tag), &format!("&lt;{}&gt;", tag))
                .replace(&format!("</{}>", tag), &format!("&lt;/{}&gt;", tag))
                .replace(&format!("{}>", tag), &format!("{}&gt;", tag))
                .replace(&format!("<{}", tag), &format!("&lt;{}", tag))
                .replace(&format!("</{}", tag), &format!("&lt;/{}", tag));
        }
    }
    sanitized
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_html() {
        for tag in FORBIDDEN_TAGS {
            let test_string = format!("<{}>asdf</{}>", tag, tag);
            let result = sanitize_html(&test_string);
            assert_ne!(test_string, result);
            assert!(result.find('>').is_none());
            assert!(result.find('<').is_none());
        }
        // broken html
        for tag in FORBIDDEN_TAGS {
            let test_string = format!("<{}asdf</{}>", tag, tag);
            let result = sanitize_html(&test_string);
            assert_ne!(test_string, result);
            assert!(result.find('>').is_none());
            assert!(result.find('<').is_none());
        }
        for tag in FORBIDDEN_TAGS {
            let test_string = format!("{}>asdf</{}>", tag, tag);
            let result = sanitize_html(&test_string);
            assert_ne!(test_string, result);
            assert!(result.find('>').is_none());
            assert!(result.find('<').is_none());
        }
    }
}
