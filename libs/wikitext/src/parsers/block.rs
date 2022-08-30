use tendril::{StrTendril, SubtendrilError, Tendril};

#[derive(Debug, PartialEq)]
pub(crate) enum BlockElement {
    Heading(Vec<BlockElement>),
    PageLink(StrTendril),
    Quote(Vec<BlockElement>),
    EmptySpace(StrTendril),
    Text(StrTendril),
    HyperLink(StrTendril),
}

type BlockResult = Result<(BlockElement, usize), SubtendrilError>;
type SliceWithIndex = Result<(StrTendril, usize), SubtendrilError>;

fn parse_heading(slice: &StrTendril) -> BlockResult {
    let mut iter = slice.char_indices().peekable();
    let mut elements = Vec::new();
    // Advance iterator to skip # character
    iter.next();
    while let Some(&(index, token)) = iter.peek() {
        match token {
            ' ' | '\t' => {
                iter.next();
            }
            _ => {
                elements = iterate_slice(&cut(slice, index)?);
                break;
            }
        }
    }
    Ok((BlockElement::Heading(elements), slice.len()))
}

fn parse_empty_space(_: &StrTendril) -> BlockResult {
    Ok((BlockElement::EmptySpace(StrTendril::from_char(' ')), 0))
}

fn parse_link(slice: &StrTendril) -> BlockResult {
    let mut link = String::new();
    let mut iter = slice.char_indices().peekable();
    let mut end_detected = false;
    // Start with the next char since we know that we've detected a link char already
    iter.next();
    while let Some(&(_, token)) = iter.peek() {
        match token {
            '[' => {
                iter.next();
            }
            ']' => {
                if !end_detected {
                    end_detected = true;
                    iter.next();
                } else {
                    break;
                }
            }
            _ => {
                link.push(token);
                iter.next();
            }
        }
    }

    let content = StrTendril::from_slice(&link);
    Ok((BlockElement::PageLink(content), link.len() + 3))
}
fn parse_quote(slice: &StrTendril) -> BlockResult {
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
                elements = iterate_slice(&cut(slice, index)?);
                break;
            }
        }
    }

    Ok((BlockElement::Quote(elements), slice.len()))
}
fn parse_text(slice: &StrTendril) -> BlockResult {
    let (content, first_empty_space) = until_empty_space(slice)?;
    if content.starts_with("http://") || content.starts_with("https://") {
        Ok((BlockElement::HyperLink(content), first_empty_space))
    } else {
        Ok((BlockElement::Text(content), first_empty_space))
    }
}

pub(crate) fn parse_block(block: &[u8]) -> Vec<BlockElement> {
    let input = StrTendril::try_from_byte_slice(block).unwrap();
    iterate_slice(&input)
}

fn iterate_slice(input: &StrTendril) -> Vec<BlockElement> {
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
            ' ' | '\t' => parse_empty_space,
            '>' => {
                if index == 0 {
                    parse_quote
                } else {
                    parse_text
                }
            }
            _ => parse_text,
        };

        let advance = match cut(input, index) {
            Ok(slice) => match parse_block(&slice) {
                Ok((block, steps)) => {
                    elements.push(block);
                    steps
                }
                Err(error) => {
                    println!("Failed to parse block: {:?}\n  {:?}", slice, error);
                    break;
                }
            },
            Err(error) => {
                println!("Failed to slice input: {:?}\n  {:?}", input, error);
                break;
            }
        };

        iter.nth(advance);
    }
    elements
}

fn cut<T>(tendril: &Tendril<T>, at: usize) -> Result<Tendril<T>, SubtendrilError>
where
    T: tendril::Format,
{
    tendril.try_subtendril(at as u32, tendril.len32() - at as u32)
}

fn until_empty_space(slice: &StrTendril) -> SliceWithIndex {
    let mut iter = slice.char_indices().peekable();
    let mut end = 0usize;
    let mut unicode_offset = 0usize;
    while let Some(&(index, token)) = iter.peek() {
        match token {
            ' ' | '\t' | '\r' | '\n' => break,
            _ => {
                // catchall for wonky unicode stuff...
                if !token.is_ascii() {
                    unicode_offset += token.len_utf8();
                }
                iter.next()
            }
        };
        end = index;
    }
    end += 1;
    let steps = if unicode_offset > 0 {
        end - unicode_offset
    } else {
        end - 1
    };
    Ok((slice.try_subtendril(0, end as u32)?, steps))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_block_headers() {
        let test_string = "# hello";
        let block = parse_block(test_string.as_bytes());
        assert_eq!(block.len(), 1);
    }

    #[test]
    fn parses_block_headers_no_space() {
        let test_string = "#hello";
        let block = parse_block(test_string.as_bytes());
        let matching_block =
            BlockElement::Heading(vec![BlockElement::Text(StrTendril::from_slice("hello"))]);
        assert_eq!(block.len(), 1);
        assert_eq!(block[0], matching_block);
    }

    #[test]
    fn parses_multi_word_headers() {
        let test_string = "#hello world";
        let block = parse_block(test_string.as_bytes());
        let matching_block = BlockElement::Heading(vec![
            BlockElement::Text(StrTendril::from_slice("hello")),
            BlockElement::EmptySpace(StrTendril::from_char(' ')),
            BlockElement::Text(StrTendril::from_slice("world")),
        ]);
        assert_eq!(block.len(), 1);
        assert_eq!(block[0], matching_block);
    }

    #[test]
    fn parses_no_block_headers_when_not_starting_with_sigil() {
        let test_string = "testing #again";
        let block = parse_block(test_string.as_bytes());
        assert_eq!(block.len(), 3);
        let matching_block = BlockElement::Text(StrTendril::from_slice("testing"));
        assert_eq!(block[0], matching_block);
    }
    #[test]
    fn parses_block_links_alone() {
        let test_string = "[[testing again]]";
        let block = parse_block(test_string.as_bytes());
        assert_eq!(block.len(), 1);
        let matching_block = BlockElement::PageLink(StrTendril::from_slice("testing again"));
        assert_eq!(block[0], matching_block);
    }

    #[test]
    #[ignore] // TODO: maybe we want to make this illegal syntax?
    fn parses_block_links_in_parens() {
        let test_string = "([[testing again]])";
        let block = parse_block(test_string.as_bytes());
        assert_eq!(block.len(), 3);
        let matching_block = BlockElement::PageLink(StrTendril::from_slice("testing again"));
        assert_eq!(block[1], matching_block);
    }

    #[test]
    fn parses_block_links_in_sentences() {
        let test_string = "parsing [[another link]]";
        let block = parse_block(test_string.as_bytes());
        assert_eq!(block.len(), 3);
        let matching_block = BlockElement::PageLink(StrTendril::from_slice("another link"));
        assert_eq!(block[2], matching_block);
    }

    #[test]
    fn parses_block_links_in_sentences_with_aliases() {
        let test_string = "parsing [[another link|some page]]";
        let block = parse_block(test_string.as_bytes());
        assert_eq!(block.len(), 3);
        let matching_block =
            BlockElement::PageLink(StrTendril::from_slice("another link|some page"));
        assert_eq!(block[2], matching_block);
    }

    #[test]
    fn parses_more_complex_sentences_with_links() {
        let test_string = "[[another page]] asdf do the things.";
        let block = parse_block(test_string.as_bytes());
        assert_eq!(block.len(), 9);
        let mut matching_block = BlockElement::PageLink(StrTendril::from_slice("another page"));
        assert_eq!(block[0], matching_block);

        matching_block = BlockElement::Text(StrTendril::from_slice("asdf"));
        assert_eq!(block[2], matching_block);
    }

    #[test]
    fn parses_block_text() {
        let test_string = "testing again";
        let block = parse_block(test_string.as_bytes());
        assert_eq!(block.len(), 3);

        let mut matching_block = BlockElement::Text(StrTendril::from_slice("testing"));
        assert_eq!(block[0], matching_block);

        matching_block = BlockElement::EmptySpace(StrTendril::from_char(' '));
        assert_eq!(block[1], matching_block);

        matching_block = BlockElement::Text(StrTendril::from_slice("again"));
        assert_eq!(block[2], matching_block);
    }

    #[test]
    fn parses_obnoxiously_long_block_text() {
        let test_string = "The question then becomes, what constitutes as a reality discovery, and what is the impetuous for our discovery? I think it’s the reality that surrounds us that are brought to the foreground by observation, discussion, and thought that lead us down the path of reality discovery.";
        let block = parse_block(test_string.as_bytes());
        assert_eq!(block.len(), 91);

        let mut matching_block = BlockElement::Text(StrTendril::from_slice("The"));
        assert_eq!(block[0], matching_block);

        matching_block = BlockElement::EmptySpace(StrTendril::from_char(' '));
        assert_eq!(block[1], matching_block);

        matching_block = BlockElement::Text(StrTendril::from_slice("question"));
        assert_eq!(block[2], matching_block);

        matching_block = BlockElement::Text(StrTendril::from_slice("the"));
        assert_eq!(block[42], matching_block);
    }

    #[test]
    fn parses_block_text_with_hyperlink() {
        let test_string = "testing http://example.com";
        let block = parse_block(test_string.as_bytes());
        assert_eq!(block.len(), 3);
        let matching_block = BlockElement::HyperLink(StrTendril::from_slice("http://example.com"));
        assert_eq!(block[2], matching_block)
    }

    #[test]
    fn parses_quotes() {
        let mut test_string = "> testing examples";
        let mut block = parse_block(test_string.as_bytes());
        assert_eq!(block.len(), 1);
        let mut matching_block = BlockElement::Quote(vec![
            BlockElement::Text(StrTendril::from_slice("testing")),
            BlockElement::EmptySpace(StrTendril::from_char(' ')),
            BlockElement::Text(StrTendril::from_slice("examples")),
        ]);
        assert_eq!(block[0], matching_block);

        test_string = "> be invented-according to";

        block = parse_block(test_string.as_bytes());
        assert_eq!(block.len(), 1);
        matching_block = BlockElement::Quote(vec![
            BlockElement::Text(StrTendril::from_slice("be")),
            BlockElement::EmptySpace(StrTendril::from_char(' ')),
            BlockElement::Text(StrTendril::from_slice("invented-according")),
            BlockElement::EmptySpace(StrTendril::from_char(' ')),
            BlockElement::Text(StrTendril::from_slice("to")),
        ]);

        assert_eq!(block[0], matching_block);
    }
}
