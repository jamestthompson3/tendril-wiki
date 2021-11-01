use crate::{get_template_file, parse_includes, process_included_file, Render};

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
            format!("[[{}]]", linkto)
        } else {
            String::new()
        }
    }
    fn render_includes(&self, ctx: String) -> String {
        let lines = ctx.lines().map(|line| {
            let line = line.trim();
            if line.starts_with("<%=") {
                process_included_file(parse_includes(line), None)
            } else {
                line.to_string()
            }
        });
        lines.collect::<Vec<String>>().join(" ")
    }
}

// TODO: Include pages
impl<'a> Render for NewPage<'a> {
    fn render(&self) -> String {
        let mut ctx = get_template_file("new_page").unwrap();
        ctx = ctx
            .replace("<%= page_title %>", self.get_page_title())
            .replace("<%= note_title %>", &self.get_note_title())
            .replace("<%= action_params %>", self.action_params.unwrap_or(""))
            .replace("<%= linkto %>", &self.get_linkto());
        self.render_includes(ctx)
    }
}
