use crate::{
    get_template_file, render_includes, render_page_backlinks, render_page_metadata, Render,
};
use async_trait::async_trait;
use persistance::fs::{config::read_config, ReadPageError};
use wikitext::GlobalBacklinks;

pub struct IndexPage {
    pub user: String,
    pub host: String,
    pub links: GlobalBacklinks,
    today: String,
}

impl IndexPage {
    pub fn new(user: String, host: String, links: GlobalBacklinks) -> Self {
        use chrono::Local;
        let now = Local::now();
        let today = now.format("%Y-%m-%d").to_string();
        Self {
            user,
            host,
            today,
            links,
        }
    }
    fn check_updates(&self) -> String {
        let config = read_config();
        if config.general.check_for_updates {
            String::from(r#"<script src="static/update-check.js" type="module"></script>"#)
        } else {
            String::with_capacity(0)
        }
    }
    async fn render_today(&self) -> String {
        let mut content = get_template_file("content").await.unwrap();
        match persistance::fs::read(self.today.clone()).await {
            Ok(note) => {
                let templatted = note.to_template();
                let mut links = self
                    .links
                    .lock()
                    .await
                    .get(&self.today)
                    .unwrap_or(&Vec::with_capacity(0))
                    .to_owned();
                links.dedup();
                links.sort_unstable();
                let tag_string = templatted
                    .page
                    .tags
                    .iter()
                    .map(|t| format!("<li><a href=\"{}\">#{}</a></li>", t, t))
                    .collect::<Vec<String>>()
                    .join("\n");
                content = content
                    .replace("<%= title %>", &self.today)
                    .replace("<%= body %>", &templatted.page.body)
                    .replace("<%= tags %>", &tag_string)
                    .replace(
                        "<%= metadata %>",
                        &render_page_metadata(templatted.page.metadata),
                    )
                    .replace("<%= links %>", &render_page_backlinks(links));
                content
            }

            Err(ReadPageError::PageNotFoundError) => {
                content = content
                    .replace("<%= title %>", &self.today)
                    .replace("<%= body %>", "<div class=\"text-block\"></div>")
                    .replace("<%= tags %>", "")
                    .replace("<%= metadata %>", "")
                    .replace("<%= links %>", "");
                content
            }
            e => {
                eprintln!("{:?}", e);
                String::with_capacity(0)
            }
        }
    }
}

#[async_trait]
impl Render for IndexPage {
    async fn render(&self) -> String {
        let mut ctx = get_template_file("index").await.unwrap();
        let nav = get_template_file("nav").await.unwrap();
        ctx = ctx
            .replace("<%= updateCheck %>", &self.check_updates())
            .replace("<%= user %>", &self.user)
            .replace("<%= host %>", &self.host)
            .replace("<%= nav %>", &nav)
            .replace("<%= content %>", &self.render_today().await);
        render_includes(ctx, None)
            .await
            .replace("<%= title %>", &self.today)
    }
}
