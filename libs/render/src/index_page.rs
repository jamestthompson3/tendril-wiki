use crate::{get_template_file, parse_includes, process_included_file, Render};

pub struct IndexPage {
    pub user: String,
}

impl IndexPage {
    pub fn new(user: String) -> Self {
        IndexPage { user }
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

impl Render for IndexPage {
    fn render(&self) -> String {
        println!("rendering index");
        let mut ctx = get_template_file("index").unwrap();
        ctx = ctx.replace("<%= user %>", &self.user);
        self.render_includes(ctx)
    }
}
