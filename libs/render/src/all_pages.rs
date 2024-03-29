use async_trait::async_trait;

use crate::{get_template_file, render_includes, Render};

type PageEntries<'a> = Vec<(&'a String, usize)>;

pub struct PageList<'a> {
    entries: PageEntries<'a>,
}

impl<'a> PageList<'a> {
    pub fn new(entries: PageEntries<'a>) -> Self {
        Self { entries }
    }
}

#[async_trait]
impl<'a> Render for PageList<'a> {
    async fn render(&self) -> String {
        let page_string = self
            .entries
            .iter()
            .map(|(name, num_links)| {
                format!(
                    "<tr><td><a href=\"{}\">{}</a></td><td style=\"text-align: center;\">{}</td></tr>",
                    name,
                    name,
                    num_links
                )
            })
            .collect::<Vec<String>>()
            .join("\n");
        let mut ctx = get_template_file("page_list").await.unwrap();
        let nav = get_template_file("nav").await.unwrap();
        ctx = ctx
            .replace("<%= content %>", &page_string);
        render_includes(ctx, None).await.replace("<%= nav %>", &nav)
    }
}
