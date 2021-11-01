use crate::{get_template_file, parse_includes, process_included_file, Render};

pub struct FileUploader {}

impl FileUploader {
    pub fn new() -> Self {
        FileUploader {}
    }
}

impl Render for FileUploader {
    fn render(&self) -> String {
        let ctx = get_template_file("file_upload").unwrap();
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
