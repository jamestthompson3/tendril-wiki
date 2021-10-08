use std::{
    collections::{BTreeMap, HashMap},
    fs::{self, File, ReadDir},
    io::Read,
    sync::{Arc, Mutex},
};

use tasks::search::SearchResult;

use sailfish::TemplateOnce;

use crate::{ingestors::get_template_file, parsers::format_links};

#[derive(TemplateOnce)]
#[template(path = "user_style.stpl")]
pub struct StylesPage {
    pub body: String,
}

#[derive(TemplateOnce)]
#[template(path = "file_list.stpl")]
pub struct UploadedFilesPage {
    pub entries: ReadDir,
}

#[derive(TemplateOnce)]
#[template(path = "new_page.stpl")]
pub struct NewPage<'a> {
    pub title: Option<String>,
    pub linkto: Option<&'a String>,
    pub action_params: Option<&'a str>,
}

#[derive(TemplateOnce)]
#[template(path = "login_page.stpl")]
pub struct LoginPage {}

#[derive(TemplateOnce)]
#[template(path = "help_page.stpl")]
pub struct HelpPage {}

#[derive(TemplateOnce)]
#[template(path = "search_page.stpl")]
pub struct SearchPage {}

#[derive(TemplateOnce)]
#[template(path = "search_results.stpl")]
pub struct SearchResultsPage {
    pub pages: Vec<String>,
}

#[derive(TemplateOnce)]
#[template(path = "file_uploader.stpl")]
pub struct FileUploader {}

#[derive(TemplateOnce)]
#[template(path = "search_results_context.stpl")]
pub struct SearchResultsContextPage {
    pub pages: Vec<SearchResult>,
}

#[derive(TemplateOnce)]
#[template(path = "tag_idx.stpl")]
pub struct TagIndex {
    pub tags: BTreeMap<String, Vec<String>>,
}

#[derive(TemplateOnce)]
#[template(path = "index.stpl")]
pub struct IndexPage {
    pub user: String,
}

#[derive(TemplateOnce)]
#[template(path = "tags.stpl")]
pub struct TagPage {
    pub title: String,
    pub tags: Vec<String>,
}

#[derive(TemplateOnce)]
#[template(path = "backlinks.stpl")]
pub struct LinkPage {
    pub links: BTreeMap<String, Vec<String>>,
}

#[derive(Debug)]
pub struct TemplattedPage {
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
    pub raw_md: String,
    pub metadata: HashMap<String, String>,
}

pub struct ParsedTemplate {
    pub outlinks: Vec<String>,
    pub page: TemplattedPage,
}

pub type TagMapping = Arc<Mutex<BTreeMap<String, Vec<String>>>>;
pub type GlobalBacklinks = Arc<Mutex<BTreeMap<String, Vec<String>>>>;
pub type ParsedPages = Arc<Mutex<Vec<TemplattedPage>>>;

pub fn render_template(
    page: &TemplattedPage,
    links: Option<&Vec<String>>,
    render_static: bool,
) -> String {
    let mut backlinks = match links {
        Some(links) => links.to_owned(),
        None => Vec::new(),
    };
    backlinks.dedup();
    let tag_string = page
        .tags
        .iter()
        .map(|t| format!("<li><a href=\"/tags/{}\">#{}</a></li>", t, t))
        .collect::<Vec<String>>()
        .join("\n");
    let mut ctx = File::open("templates/main.html").unwrap();
    let mut ctx_string = String::new();
    ctx.read_to_string(&mut ctx_string).unwrap();
    ctx_string = ctx_string
        .replace("<%= title %>", &page.title)
        .replace("<%= body %>", &page.body)
        .replace("<%= tags %>", &tag_string)
        .replace("<%= links %>", &render_page_backlinks(&backlinks))
        .replace("<%= metadata %>", &render_page_metadata(&page.metadata));
    let parsed = ctx_string.split('\n');
    parsed
        .map(|line| {
            if line.trim().starts_with("<%= include") {
                let included_file = parse_includes(line.trim());
                match included_file.as_ref() {
                    "nav" | "edit" => {
                        if render_static {
                            return String::with_capacity(0);
                        }
                        process_included_file(included_file, page)
                    }

                    _ => get_template_file(&included_file).unwrap(),
                }
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn render_page_backlinks(links: &[String]) -> String {
    if !links.is_empty() {
        let backlinks_string = links
            .iter()
            .map(|l| format!("<a href=\"{}\">{}</a>", format_links(l), l))
            .collect::<Vec<String>>()
            .join("\n");
        format!(
            r#"
<section class="backlinks-container">
  <hr />
  <h3>Mentioned in:</h3>
  <div class="backlinks">{}</div>
</section>
"#,
            backlinks_string
        )
    } else {
        String::with_capacity(0)
    }
}

fn render_page_metadata(metadata: &HashMap<String, String>) -> String {
    let mut metadata_html = String::new();
    for (key, value) in metadata.iter() {
        metadata_html.push_str(&format!("<dt><strong>{}:</strong></dt>", key));
        // TODO: Add "created" date here as well
        // TODO: Modify dates to be compliant with DT parsing
        // if key == "modified" {
        //     val = value.parse::<DateTime<FixedOffset>>().unwrap().format("%Y-%m-%d %H:%M").to_string();
        //   }
        if value.starts_with("http") {
            match key.as_str() {
                "cover" => {
                    let val = format!(
                        "<img src=\"{}\" style=\"max-height: 200px; max-width: 200px;\">",
                        value
                    );
                    metadata_html.push_str(&format!("\n<dd>{}</dd>", val));
                }
                _ => {
                    let val = format!("<a href=\"{}\">{}</a>", value, value);
                    metadata_html.push_str(&format!("\n<dd>{}</dd>", val));
                }
            }
        } else {
            metadata_html.push_str(&format!("\n<dd>{}</dd>", &value));
        }
    }
    metadata_html
}

fn process_included_file(file: String, page: &TemplattedPage) -> String {
    match file.as_ref() {
        "nav" => {
            let templatefile = get_template_file("nav").unwrap();
            templatefile.replace("{{TITLE}}", &page.title)
        }
        "edit" => {
            let templatefile = get_template_file("edit").unwrap();
            let metadata_string = page
                .metadata
                .iter()
                .map(|(k, v)| format!("{}:{}", k, v))
                .collect::<Vec<String>>()
                .join("\n");
            templatefile
                .replace("<%= title %>", &page.title)
                .replace("<%= tags %>", &page.tags.join(","))
                .replace("<%= raw_md %>", &page.raw_md)
                .replace("<%= metadata %>", &metadata_string)
        }
        _ => String::with_capacity(0),
    }
}

fn parse_includes(include_str: &str) -> String {
    let included_file = include_str
        .strip_prefix("<%= include \"")
        .unwrap()
        .strip_suffix("\" %>")
        .unwrap();
    included_file.to_string()
}

pub fn write_index_page(user: String) {
    let ctx = IndexPage { user };
    fs::write("public/index.html", ctx.render_once().unwrap()).unwrap();
}

pub fn write_entries(pages: &ParsedPages, backlinks: &GlobalBacklinks) {
    let page_vals = pages.lock().unwrap();
    let link_vals = backlinks.lock().unwrap();
    for page in page_vals.iter() {
        let links = link_vals.get(&page.title);
        let output = render_template(&page, links, true);
        // TODO use path here instead of title? Since `/` in title can cause issues in fs::write
        fs::create_dir(format!("public/{}", page.title.replace('/', "-"))).unwrap();
        fs::write(
            format!("public/{}/index.html", page.title.replace('/', "-")),
            output,
        )
        .unwrap();
    }
}

pub fn write_tag_pages(map: TagMapping, pages: &ParsedPages) {
    let tag_map = map.lock().unwrap();
    for key in tag_map.keys() {
        let title = key.to_string();
        let tags = tag_map.get(key).unwrap().to_owned();
        let pages = pages.lock().unwrap();
        let page = pages.iter().find(|pg| pg.title == title);
        if let Some(template) = page {
            let output = render_template(template, Some(&tags), true);
            fs::create_dir(format!("public/tags/{}", title)).unwrap();
            fs::write(format!("public/tags/{}/index.html", title), output).unwrap();
        } else {
            let ctx = TagPage {
                title: title.clone(),
                tags,
            };
            fs::create_dir(format!("public/tags/{}", title)).unwrap();
            fs::write(
                format!("public/tags/{}/index.html", title),
                ctx.render_once().unwrap(),
            )
            .unwrap();
        }
    }
}

pub fn write_tag_index(map: TagMapping) {
    let tag_map = map.lock().unwrap();
    let ctx = TagIndex {
        tags: tag_map.clone(),
    };
    fs::write("public/tags/index.html", ctx.render_once().unwrap()).unwrap();
}

pub fn write_backlinks(map: GlobalBacklinks) {
    let link_map = map.lock().unwrap();
    let ctx = LinkPage {
        links: link_map.clone(),
    };
    fs::write(
        "public/links/index.html".to_string(),
        ctx.render_once().unwrap(),
    )
    .unwrap();
}
