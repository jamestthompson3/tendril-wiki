use super::block::{parse_block, BlockElement};
use std::fmt::Write as _;

pub struct Html {
    pub outlinks: Vec<String>,
    pub body: String,
}

pub fn to_html(md: &str) -> Html {
    // let now = Instant::now();
    let mut outlinks = Vec::new();
    let mut page_blocks: Vec<Vec<BlockElement>> = Vec::new();
    for line in md.lines() {
        let blocks = parse_block(line.as_bytes());
        page_blocks.push(blocks);
    }
    let output = page_blocks
        .iter()
        .map(|block| {
            let mut rendered_string = String::from(r#"<div class="text-block">"#);
            for entity in block {
                if let BlockElement::PageLink(outlink) = entity {
                    let aliases = outlink.split('|').collect::<Vec<&str>>();
                    if aliases.len() > 1 {
                        outlinks.push(aliases[1].to_string());
                    } else {
                        outlinks.push(aliases[0].to_string());
                    }
                }
                entity.collapse_to(&mut rendered_string);
            }
            write!(rendered_string, "</div>").unwrap();

            rendered_string
        })
        .collect::<Vec<String>>()
        .join("");
    // println!("Parsed to HTML in {:?}", now.elapsed());

    Html {
        body: output,
        outlinks,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_md_to_html_with_wikilinks() {
        let test_string = "[[Some Page]]";
        let test_html = Html {
            outlinks: vec!["Some Page".into()],
            body: "<div class=\"text-block\"><a href=\"/Some%20Page\">Some Page</a></div>".into(),
        };
        let parsed = to_html(test_string);
        assert_eq!(parsed.outlinks, test_html.outlinks);
        assert_eq!(parsed.body, test_html.body);

        let test_string = "# Title\n[[Some Page]]. Another thing\n * Hi\n * List\n * Output";
        let test_html = Html {
            outlinks: vec!["Some Page".into()],
            body: "<div class=\"text-block\"><h2>Title</h2></div><div class=\"text-block\"><a href=\"/Some%20Page\">Some Page</a>. Another thing</div><div class=\"text-block\"> * Hi</div><div class=\"text-block\"> * List</div><div class=\"text-block\"> * Output</div>".into()
        };
        let parsed = to_html(test_string);
        assert_eq!(parsed.outlinks, test_html.outlinks);
        assert_eq!(parsed.body, test_html.body);
    }
}
