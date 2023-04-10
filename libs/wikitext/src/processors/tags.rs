pub struct TagsArray<'a> {
    pub values: Vec<&'a str>,
}

impl<'a> TagsArray<'a> {
    pub fn new(tag_str: &'a str) -> Self {
        if tag_str.find('[').is_some() {
            let split_tags = tag_str
                .strip_prefix('[')
                .unwrap()
                .strip_suffix(']')
                .unwrap()
                .split(',')
                .filter(|s| !s.is_empty() && s != &" ") // maybe use filter_map here?
                .map(|s| s.trim())
                .collect();
            TagsArray { values: split_tags }
        } else {
            TagsArray {
                values: tag_str.split(' ').filter(|s| !s.is_empty()).collect(),
            }
        }
    }
    pub fn write(&self) -> String {
        let mut tag_string = self.values.join(",");
        tag_string.push(']');
        tag_string.insert(0, '[');
        tag_string
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

pub fn tag_string_from_vec(vec: Vec<String>) -> String {
    let mut tag_string = vec.join(",");
    tag_string.push(']');
    tag_string.insert(0, '[');
    tag_string
}

// impl<'a> From<String> for TagsArray<'a> {
//     fn from(tag_string: String) -> Self {
//         TagsArray::new(&tag_string)
//     }
// }

// impl From<Vec<String>> for TagsArray<'_> {
//     fn from(tag_vec: Vec<String>) -> Self {
//         TagsArray { values: tag_vec }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tags_with_wikilink() {
        let tag_string = "[reality building, Article]";
        assert_eq!(
            TagsArray::new(tag_string).values,
            vec!["reality building", "Article"]
        );
    }

    #[test]
    fn parse_tags_without_wikilinks() {
        let tag_string = "Tools Article project-management";
        assert_eq!(
            TagsArray::new(tag_string).values,
            vec!["Tools", "Article", "project-management"]
        );
    }

    #[test]
    fn writes_tags_without_quotes() {
        let tags_arr = TagsArray::new("[Tools Article, project-management]");

        assert_eq!(
            tags_arr.write(),
            String::from("[Tools Article,project-management]")
        );
    }
}
