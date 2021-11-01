use crate::{get_template_file, parse_includes, process_included_file, Render};

pub struct HelpPage {}

impl HelpPage {
    pub fn new() -> Self {
        HelpPage {}
    }
}

impl Render for HelpPage {
    fn render(&self) -> String {
        let ctx = get_template_file("help").unwrap();
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
