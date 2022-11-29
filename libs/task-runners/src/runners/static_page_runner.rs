use std::collections::HashMap;

use persistance::fs::utils::get_config_location;
use render::{uploaded_files_page::UploadedFilesPage, Render, styles_page::StylesPage, error_page::ErrorPage};
use tokio::fs::{read_dir, self};


pub struct StaticPageRunner {}

impl StaticPageRunner {
    pub async fn list_files(media_location: String) -> String {
        let mut entry_list = Vec::new();
        let mut entries = read_dir(media_location).await.unwrap();
        while let Ok(entry) = entries.next_entry().await {
            if entry.is_some() {
                let entry = entry.unwrap();
                entry_list.push(entry.file_name().into_string().unwrap());
            } else {
                break;
            }
        }
        let ctx = UploadedFilesPage {
            entries: entry_list,
        };
        ctx.render().await
    }
    pub async fn render_styles() -> String {
        let (path, _) = get_config_location();
        let style_location = path.join("userstyles.css");
        let body = fs::read_to_string(style_location).await.unwrap();
        let body = body.replace('\n', "\r\n");
        let ctx = StylesPage { body };
        ctx.render().await
    }
    pub async fn show_error(params: HashMap<String, String>) -> String {
        let msg = params
            .get("msg")
            .unwrap_or(&String::from("Error could not be determined."))
            .to_string();
        let ctx = ErrorPage { msg };
        ctx.render().await
    }
}
