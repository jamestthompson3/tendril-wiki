use tasks::CompileState;

use crate::{get_template_file, render_includes, Render};

pub struct FileUploader {}

impl FileUploader {
    pub fn new() -> Self {
        FileUploader {}
    }
}

impl Render for FileUploader {
    fn render(&self, state: &CompileState) -> String {
        let ctx = get_template_file("file_upload").unwrap();
        render_includes(ctx, state, None)
    }
}
