use crate::{get_template_file, render_includes, render_sidebar, Render};
use async_trait::async_trait;

pub struct NewPage<'a> {
    pub title: Option<String>,
    pub linkto: Option<&'a String>,
    pub action_params: Option<&'a str>,
}

impl<'a> NewPage<'a> {
    pub fn new(
        title: Option<String>,
        linkto: Option<&'a String>,
        action_params: Option<&'a str>,
    ) -> Self {
        Self {
            title,
            linkto,
            action_params,
        }
    }
    fn get_page_title(&self) -> &str {
        if let Some(page_title) = &self.title {
            page_title
        } else {
            "New Entry"
        }
    }
    fn get_note_title(&self) -> String {
        if let Some(note_title) = &self.title {
            String::from(note_title)
        } else {
            use chrono::Local;
            let date = Local::now();
            date.format("%Y%m%d%H%M%S").to_string()
        }
    }
    fn get_linkto(&self) -> String {
        if let Some(linkto) = &self.linkto {
            format!(r#"<a href="{}">{}</a>"#, linkto, linkto)
        } else {
            String::new()
        }
    }
}

#[async_trait]
impl<'a> Render for NewPage<'a> {
    async fn render(&self) -> String {
        let mut ctx = get_template_file("new_page").await.unwrap();
        let mut content = get_template_file("content").await.unwrap();
        let nav = get_template_file("nav").await.unwrap();
        let body = format!(r#"<div class="text-block" tabindex="0">{}</div>"#, self.get_linkto());
        content = content
            .replace("<%= body %>", &body)
            .replace("<%= metadata %>", "")
            .replace("<%= links %>", "");
        ctx = ctx
            .replace("<%= sidebar %>", &render_sidebar().await)
            .replace("<%= content %>", &content)
            .replace("<%= page_title %>", self.get_page_title())
            .replace("<%= action_params %>", self.action_params.unwrap_or(""))
            .replace("<%= title %>", &self.get_note_title())
            .replace("<%= tags %>", "");
        render_includes(ctx, None).await.replace("<%= nav %>", &nav)
    }
}
