use crate::{get_template_file, parse_includes, process_included_file, Render};

pub struct StylesPage {
    pub body: String,
}

impl StylesPage {
    pub fn new(body: String) -> Self {
        StylesPage { body }
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

impl Render for StylesPage {
    fn render(&self) -> String {
        let mut ctx = get_template_file("index").unwrap();
        ctx = ctx.replace("<%= body %>", &self.body);
        self.render_includes(ctx)
    }
}
