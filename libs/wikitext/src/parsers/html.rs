use super::block::{parse_block, BlockElement};

pub struct Html {
    pub outlinks: Vec<String>,
    pub body: String,
}

pub(crate) struct Block {
    pub indentation_level: u32,
    pub text: String,
}

impl Block {
    pub(crate) fn new() -> Self {
        Self {
            text: String::new(),
            indentation_level: 0,
        }
    }
    pub fn close(&self) -> String {
        format!(
            r#"<div data-indent="{}" class="text-block">{}</div>"#,
            self.indentation_level, self.text
        )
    }
    pub fn update_indentation(&mut self, indentation_level: u32) {
        self.indentation_level = indentation_level
    }
}

pub fn to_html(text: &str) -> Html {
    // let now = Instant::now();
    let mut outlinks = Vec::new();
    let mut page_blocks: Vec<Vec<BlockElement>> = Vec::new();
    for line in text.lines() {
        let blocks = parse_block(line);
        page_blocks.push(blocks);
    }
    let output = page_blocks
        .iter()
        .filter_map(|block| {
            if block.is_empty() {
                return None;
            }
            let mut final_block = Block::new();
            for entity in block {
                match entity {
                    BlockElement::PageLink(outlink) => {
                        let aliases = outlink.split('|').collect::<Vec<&str>>();
                        if aliases.len() > 1 {
                            outlinks.push(aliases[1].to_string());
                        } else {
                            outlinks.push(aliases[0].to_string());
                        }
                    }
                    BlockElement::IndentationLevel(level) => {
                        final_block.update_indentation(*level);
                    }
                    _ => {}
                }
                entity.collapse_to(&mut final_block.text);
            }

            Some(final_block.close())
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
    fn parses_wikitext_to_html_with_wikilinks() {
        let test_string = "[[Some Page]]";
        let test_html = Html {
            outlinks: vec!["Some Page".into()],
            body: r#"<div data-indent="0" class="text-block"><a href="/Some%20Page">Some Page</a></div>"#.into(),
        };
        let parsed = to_html(test_string);
        assert_eq!(parsed.outlinks, test_html.outlinks);
        assert_eq!(parsed.body, test_html.body);

        let test_string = "# Title\n[[Some Page]]. Another thing\n * Hi\n * List\n * Output";
        let test_html = Html {
            outlinks: vec!["Some Page".into()],
            body: r#"<div data-indent="0" class="text-block"><h2>Title</h2></div><div data-indent="0" class="text-block"><a href="/Some%20Page">Some Page</a>. Another thing</div><div data-indent="0" class="text-block"> * Hi</div><div data-indent="0" class="text-block"> * List</div><div data-indent="0" class="text-block"> * Output</div>"#.into()
        };
        let parsed = to_html(test_string);
        assert_eq!(parsed.outlinks, test_html.outlinks);
        assert_eq!(parsed.body, test_html.body);
    }
}
