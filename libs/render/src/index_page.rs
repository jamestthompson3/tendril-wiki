use crate::{
    get_template_file, render_includes, render_page_backlinks, render_page_metadata,
    render_sidebar, GlobalBacklinks, Render,
};
use async_trait::async_trait;
use persistance::fs::ReadPageError;
use wikitext::processors::to_template;

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
    async fn render_today(&self) -> String {
        let mut content = get_template_file("content").await.unwrap();
        match persistance::fs::read(self.today.clone()).await {
            Ok(note) => {
                let templatted = to_template(&note);
                let link_vals = self.links.lock().await;
                let mut links = match link_vals.get(&templatted.page.title) {
                    Some(links) => links.to_owned(),
                    None => Vec::with_capacity(0),
                };
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
                    .replace("<%= links %>", &render_page_backlinks(&links));
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
            .replace("<%= user %>", &self.user)
            .replace("<%= title %>", &self.today)
            .replace("<%= sidebar %>", &render_sidebar().await)
            .replace("<%= host %>", &self.host)
            .replace("<%= content %>", &self.render_today().await);
        render_includes(ctx, None).await.replace("<%= nav %>", &nav)
    }
}
