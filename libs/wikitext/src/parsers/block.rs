use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Could not cut at given index")]
    CutError,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum BlockElement<'a> {
    Heading(&'a str),
    PageLink(&'a str),
    Quote(Vec<BlockElement<'a>>),
    EmptySpace(&'a str),
    Text(&'a str),
    HyperLink(&'a str),
    IndentationLevel(u32),
}

type BlockResult<'a> = Result<(BlockElement<'a>, usize), ParseError>;
type SliceWithIndex<'a> = Result<(&'a str, usize), ParseError>;

fn parse_heading(slice: &str) -> BlockResult {
    let mut iter = slice.char_indices().peekable();
    // Advance iterator to skip # character
    iter.next();
    let mut end = 1usize;
    while let Some(&(_, token)) = iter.peek() {
        match token {
            ' ' | '\t' => {
                iter.next();
                end += 1;
            }
            _ => {
                break;
            }
        }
    }
    Ok((
        BlockElement::Heading(window(slice, end, slice.len())),
        slice.len(),
    ))
}

fn parse_empty_space(slice: &str) -> BlockResult {
    Ok((BlockElement::EmptySpace(window(slice, 0, 1)), 0))
}

fn parse_indentation(slice: &str) -> BlockResult {
    let mut iter = slice.char_indices().peekable();
    let mut indentation_level = 0;
    while let Some(&(index, token)) = iter.peek() {
        indentation_level = index;
        if token == '\t' {
            iter.next();
        } else {
            break;
        }
    }
    let step = if indentation_level > 0 {
        indentation_level - 1
    } else {
        0
    };
    Ok((
        BlockElement::IndentationLevel(indentation_level as u32),
        step,
    ))
}

fn parse_link(slice: &str) -> BlockResult {
    if slice.starts_with("[[") {
        let mut iter = slice.char_indices().peekable();
        // Skip first two indicies
        iter.next();
        iter.next();
        let mut idx = 0;
        while let Some(&(index, token)) = iter.peek() {
            idx = index;
            match token {
                ']' => {
                    break;
                }
                _ => {
                    iter.next();
                }
            }
        }
        return Ok((BlockElement::PageLink(window(slice, 2, idx)), idx + 1));
    }
    Ok((BlockElement::Text(window(slice, 0, 1)), 0))
}
fn parse_quote(slice: &str) -> BlockResult {
    let mut elements = Vec::new();
    let mut iter = slice.char_indices().peekable();
    // Advance iterator to skip > character
    iter.next();
    while let Some(&(index, token)) = iter.peek() {
        match token {
            ' ' | '\t' => {
                iter.next();
            }
            _ => {
                elements = iterate_slice(slice.get(index..slice.len()).unwrap());
                break;
            }
        }
    }

    Ok((BlockElement::Quote(elements), slice.len()))
}

fn parse_text(slice: &str) -> BlockResult {
    let (content, first_empty_space) = until_empty_space(slice)?;
    if content.starts_with("http://") || content.starts_with("https://") {
        if content.ends_with(')') || content.ends_with(']') {
            return Ok((
                BlockElement::HyperLink(window(slice, 0, content.len() - 1)),
                first_empty_space - 1,
            ));
        } else {
            return Ok((BlockElement::HyperLink(content), first_empty_space));
        }
    }
    if content.starts_with('(') {
        Ok((BlockElement::Text(window(slice, 0, 1)), 0))
    } else {
        Ok((BlockElement::Text(content), first_empty_space))
    }
}

pub(crate) fn parse_block(block: &str) -> Vec<BlockElement> {
    iterate_slice(block)
}

fn iterate_slice(input: &str) -> Vec<BlockElement> {
    let mut elements = Vec::new();
    let mut iter = input.char_indices().peekable();
    while let Some(&(index, token)) = iter.peek() {
        let parse_block = match token {
            '#' => {
                // Only make it a heading if it's at the beginning of the line
                if index == 0 {
                    parse_heading
                } else {
                    parse_text
                }
            }
            '[' => parse_link,
            ' ' => parse_empty_space,
            '\t' => {
                if index == 0 {
                    parse_indentation
                } else {
                    parse_empty_space
                }
            }
            '>' => {
                if index == 0 {
                    parse_quote
                } else {
                    parse_text
                }
            }
            _ => parse_text,
        };

        let advance = match parse_block(window(input, index, input.len())) {
            Ok((block, steps)) => {
                elements.push(block.clone());
                steps
            }
            Err(error) => {
                panic!("Failed to parse block: {:?}", error);
            }
        };

        iter.nth(advance);
    }
    elements
}

// fn cut(slice: &str, at: usize) -> Result<&str, ParseError> {
//     if at == 0 {
//         return Ok(slice);
//     }
//     Ok(window(slice, at, slice.len()))
// }

fn window(slice: &str, start: usize, end: usize) -> &str {
    slice.get(start..end).unwrap()
}

fn until_empty_space(slice: &str) -> SliceWithIndex {
    let mut iter = slice.char_indices().peekable();
    let mut end = 0usize;
    let mut unicode_offset = 0usize;
    let mut unicode_count = 0usize;
    while let Some(&(index, token)) = iter.peek() {
        match token {
            ' ' | '\t' | '\r' | '\n' => break,
            _ => {
                if !token.is_ascii() {
                    unicode_offset += token.len_utf8();
                    unicode_count += 1;
                }
                end = index;
                iter.next()
            }
        };
    }
    let window_offset = if end == 0 && unicode_offset > 0 {
        end + unicode_offset
    } else {
        end + 1
    };
    let windowed = window(slice, 0, window_offset);
    let steps = if end > 0 && end > unicode_offset && unicode_offset > 0 {
        // I don't know exactly why this works, but it does...
        // There's something with the number of unicode chars that makes a difference...
        (windowed.len() - unicode_offset) + unicode_count - 1
    } else {
        end
    };
    Ok((windowed, steps))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_block_headers() {
        let test_string = "# hello";
        let block = parse_block(test_string);
        assert_eq!(block.len(), 1);
    }

    #[test]
    fn parses_block_headers_no_space() {
        let test_string = "#hello";
        let block = parse_block(test_string);
        let matching_block = BlockElement::Heading("hello");
        assert_eq!(block.len(), 1);
        assert_eq!(block[0], matching_block);
    }

    #[test]
    fn parses_multi_word_headers() {
        let test_string = "#hello world";
        let block = parse_block(test_string);
        let matching_block = BlockElement::Heading("hello world");
        assert_eq!(block.len(), 1);
        assert_eq!(block[0], matching_block);
    }

    #[test]
    fn parses_no_block_headers_when_not_starting_with_sigil() {
        let test_string = "testing #again";
        let block = parse_block(test_string);
        assert_eq!(block.len(), 3);
        let matching_block = BlockElement::Text("testing");
        assert_eq!(block[0], matching_block);
    }
    #[test]
    fn parses_block_links_alone() {
        let test_string = "[[testing again]]";
        let block = parse_block(test_string);
        assert_eq!(block.len(), 1);
        let matching_block = BlockElement::PageLink("testing again");
        assert_eq!(block[0], matching_block);
    }

    #[test]
    fn parses_block_links_in_parens() {
        let test_string = "([[testing again]])";
        let block = parse_block(test_string);
        assert_eq!(block.len(), 3);
        let mut matching_block = BlockElement::PageLink("testing again");
        assert_eq!(block[1], matching_block);
        matching_block = BlockElement::Text("(");
        assert_eq!(block[0], matching_block);
        matching_block = BlockElement::Text(")");
        assert_eq!(block[2], matching_block);
    }

    #[test]
    fn parses_raw_links_in_brackets() {
        let test_string = "[https://example.com]";
        let block = parse_block(test_string);
        assert_eq!(block.len(), 3);
        let mut matching_block = BlockElement::HyperLink("https://example.com");
        assert_eq!(block[1], matching_block);
        matching_block = BlockElement::Text("[");
        assert_eq!(block[0], matching_block);
        matching_block = BlockElement::Text("]");
        assert_eq!(block[2], matching_block);
    }
    #[test]
    fn parses_raw_links_in_parens() {
        let test_string = "(https://example.com)";
        let block = parse_block(test_string);
        assert_eq!(block.len(), 3);
        let mut matching_block = BlockElement::HyperLink("https://example.com");
        assert_eq!(block[1], matching_block);
        matching_block = BlockElement::Text("(");
        assert_eq!(block[0], matching_block);
        matching_block = BlockElement::Text(")");
        assert_eq!(block[2], matching_block);
    }

    #[test]
    fn parses_block_links_in_sentences() {
        let test_string = "parsing [[another link]]";
        let block = parse_block(test_string);
        assert_eq!(block.len(), 3);
        let matching_block = BlockElement::PageLink("another link");
        assert_eq!(block[2], matching_block);
    }

    #[test]
    fn parses_block_links_in_sentences_with_aliases() {
        let test_string = "parsing [[another link|some page]]";
        let block = parse_block(test_string);
        assert_eq!(block.len(), 3);
        let matching_block = BlockElement::PageLink("another link|some page");
        assert_eq!(block[2], matching_block);
    }

    #[test]
    fn parses_more_complex_sentences_with_links() {
        let test_string = "[[another page]] asdf do the things.";
        let block = parse_block(test_string);
        assert_eq!(block.len(), 9);
        let mut matching_block = BlockElement::PageLink("another page");
        assert_eq!(block[0], matching_block);

        matching_block = BlockElement::Text("asdf");
        assert_eq!(block[2], matching_block);
    }

    #[test]
    fn parses_block_text() {
        let test_string = "testing again";
        let block = parse_block(test_string);
        assert_eq!(block.len(), 3);

        let mut matching_block = BlockElement::Text("testing");
        assert_eq!(block[0], matching_block);

        matching_block = BlockElement::EmptySpace(" ");
        assert_eq!(block[1], matching_block);

        matching_block = BlockElement::Text("again");
        assert_eq!(block[2], matching_block);
    }

    #[test]
    fn parses_obnoxiously_long_block_text() {
        let test_string = "The question then becomes, what constitutes as a reality discovery, and what is the impetuous for our discovery? I think it’s the reality that surrounds us that are brought to the foreground by observation, discussion, and thought that lead us down the path of reality discovery.";
        let block = parse_block(test_string);
        assert_eq!(block.len(), 91);

        let mut matching_block = BlockElement::Text("The");
        assert_eq!(block[0], matching_block);

        matching_block = BlockElement::EmptySpace(" ");
        assert_eq!(block[1], matching_block);

        matching_block = BlockElement::Text("question");
        assert_eq!(block[2], matching_block);

        matching_block = BlockElement::Text("the");
        assert_eq!(block[42], matching_block);
    }

    #[test]
    fn parses_block_with_hyperlink() {
        let test_string = "testing http://example.com";
        let block = parse_block(test_string);
        assert_eq!(block.len(), 3);
        let matching_block = BlockElement::HyperLink("http://example.com");
        assert_eq!(block[2], matching_block)
    }

    #[test]
    fn parses_unicode_text() {
        let mut test_string = "✅ = currently in app";
        let mut block = parse_block(test_string);
        assert_eq!(block.len(), 9);
        let mut matching_block = BlockElement::EmptySpace(" ");
        assert_eq!(block[1], matching_block);
        test_string = "lähtevät imuroin";
        block = parse_block(test_string);
        assert_eq!(block.len(), 3);
        matching_block = BlockElement::Text("imuroin");
        assert_eq!(block[2], matching_block);
        // A really tricky one! unicode apostrophes D:
        test_string = "it’s the worst";
        block = parse_block(test_string);
        assert_eq!(block.len(), 5);
        matching_block = BlockElement::Text("the");
        assert_eq!(block[2], matching_block);
    }

    #[test]
    fn parses_quotes() {
        let mut test_string = "> testing examples";
        let mut block = parse_block(test_string);
        assert_eq!(block.len(), 1);
        let mut matching_block = BlockElement::Quote(vec![
            BlockElement::Text("testing"),
            BlockElement::EmptySpace(" "),
            BlockElement::Text("examples"),
        ]);
        assert_eq!(block[0], matching_block);

        test_string = "> be invented-according to";

        block = parse_block(test_string);
        assert_eq!(block.len(), 1);
        matching_block = BlockElement::Quote(vec![
            BlockElement::Text("be"),
            BlockElement::EmptySpace(" "),
            BlockElement::Text("invented-according"),
            BlockElement::EmptySpace(" "),
            BlockElement::Text("to"),
        ]);

        assert_eq!(block[0], matching_block);
    }

    #[test]
    fn parses_indentation_levels() {
        let mut test_string = "\ttesting examples";
        let mut block = parse_block(test_string);
        assert_eq!(block.len(), 4);
        let mut matching_block = BlockElement::IndentationLevel(1);
        assert_eq!(block[0], matching_block);

        test_string = "\t\ttesting examples";
        block = parse_block(test_string);
        assert_eq!(block.len(), 4);
        matching_block = BlockElement::IndentationLevel(2);
        assert_eq!(block[0], matching_block);

        matching_block = BlockElement::Text("testing");
        assert_eq!(block[1], matching_block);
    }
}
