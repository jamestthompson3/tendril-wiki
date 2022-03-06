use std::{collections::HashMap, str::FromStr};
use thiserror::Error;

use regex::Regex;
#[macro_use]
extern crate lazy_static;

// TODO: explore parsing with something like tendril and doing it based on a stream instead of
// regex

lazy_static! {
    static ref PRIO_RGX: Regex = Regex::new(r"\([[:upper:]]\)\s").unwrap();
    static ref PROJ_RGX: Regex = Regex::new(r"(^\+|\s\+)\S+").unwrap();
    static ref CTX_RGX: Regex = Regex::new(r"(^@|\s@)\S+").unwrap();
    static ref META_RGX: Regex = Regex::new(r"\S+:\S+").unwrap();
    // NOTE: This would be easier if the regex crate supported look behinds
    static ref CREATE_RGX: Regex = Regex::new(r"(?:^\([[:upper:]]\)\s)(\d{4}-\d{2}-\d{2})|(^\d{4}-\d{2}-\d{2})").unwrap();
    static ref DATE_RGX: Regex = Regex::new(r"^\d{4}-\d{2}-\d{2}").unwrap();
    static ref DONE_RGX: Regex =
        Regex::new(r"^x\s\d{4}-\d{2}-\d{2}\s(\([[:upper:]]\)|\d{4}-\d{2}-\d{2})?").unwrap();
}

#[derive(Error, Debug)]
pub enum TaskParseErr {
    #[error("Could not parse &str")]
    StrParseFail,
}

#[derive(Debug, PartialEq)]
pub struct Task {
    pub priority: Option<String>,
    pub project: Vec<String>,
    pub context: Vec<String>,
    pub body: String,
    pub completed: (bool, Option<String>),
    pub created: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl FromStr for Task {
    type Err = TaskParseErr;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let completed = parse_completed(s);
        let project = parse_project(s);
        let context = parse_context(s);
        let priority = parse_priority(s);
        let metadata = parse_meta(s);
        let created = parse_created(s);
        Ok(Task {
            completed,
            created,
            project,
            priority,
            context,
            metadata,
            body: s.into(),
        })
    }
}

impl Task {
    pub fn to_html(&self) -> String {
        let mut html = String::new();
        let status = self.format_status();
        let priority = self
            .priority
            .to_owned()
            .unwrap_or_else(|| String::with_capacity(0));
        let created = self
            .created
            .to_owned()
            .unwrap_or_else(|| String::with_capacity(0));
        let metadata = self.format_metadata();
        let body = self.format_body();
        let table_html = format!(
            r#"<tr><td tabindex="-1">{}</td><td tabindex="-1">{}</td><td tabindex="-1">{}</td><td tabindex="-1">{}</td><td tabindex="-1">{}</td></tr>"#,
            status, priority, created, body, metadata
        );
        html.push_str(&table_html);
        html
    }

    fn format_body(&self) -> String {
        let mut formatted = self.body.clone();
        if let Some(prio) = &self.priority {
            formatted = formatted.replace(&format!("({})", prio), "");
        }
        let is_complete = &self.completed.0;
        if *is_complete {
            formatted = formatted.strip_prefix("x ").unwrap().into();
            let completion_date = &self.completed.1.as_ref();
            if completion_date.is_some() {
                let completion_date = completion_date.unwrap();
                formatted = formatted.replace(completion_date, "");
            }
        }
        if let Some(created) = &self.created {
            formatted = formatted.replace(created, "");
        }
        for p in &self.project {
            let project_fmt = self.format_contextual_data(p, '+');
            formatted = formatted.replace(p, &project_fmt);
        }

        for c in &self.context {
            let ctx_fmt = self.format_contextual_data(c, '@');
            formatted = formatted.replace(c, &ctx_fmt);
        }
        formatted
    }

    fn format_status(&self) -> String {
        let (done, date) = &self.completed;
        if let true = done {
            if date.is_some() {
                format!("✅ {}", date.as_ref().unwrap())
            } else {
                "✅".into()
            }
        } else {
            String::with_capacity(0)
        }
    }
    fn format_contextual_data(&self, context: &str, prefix: char) -> String {
        let css_class = match prefix {
            '+' => "project",
            '@' => "context",
            _ => panic!("Invalid prefix: {}", prefix),
        };
        format!(
            r#"<a href="{}" target="_blank" class="{}">{}</a>"#,
            context.strip_prefix(prefix).unwrap(),
            css_class,
            context
        )
    }
    fn format_metadata(&self) -> String {
        self.metadata
            .iter()
            .fold(String::new(), |mut formatted_str, (key, value)| {
                let ctx_string = format!("<strong>{}:</strong> {}", key, value);
                formatted_str.push_str(&ctx_string);
                formatted_str
            })
    }
}

fn parse_completed(s: &str) -> (bool, Option<String>) {
    let completed_found = DONE_RGX.find(s);
    if let Some(found) = completed_found {
        let done_parsed = found.as_str().split(' ').collect::<Vec<&str>>();
        (true, Some(String::from(done_parsed[1])))
    } else if s.starts_with("x ") {
        (true, None)
    } else {
        (false, None)
    }
}

fn parse_created(s: &str) -> Option<String> {
    if s.starts_with("x ") {
        let mut sliced = s.replace("x ", "");
        if DATE_RGX.is_match_at(&sliced, 0) {
            let (_, completed_removed) = sliced.split_at_mut(11);
            capture_created(completed_removed)
        } else {
            capture_created(&sliced)
        }
    } else {
        capture_created(s)
    }
}

fn capture_created(s: &str) -> Option<String> {
    let created_found = CREATE_RGX.captures(s);
    if let Some(found) = created_found {
        // we don't know which capture group caught the date, so let's check both one and two
        if let Some(cap_group) = found.get(1) {
            if DATE_RGX.is_match(cap_group.as_str()) {
                let created = cap_group.as_str();
                let created = created.trim();
                return Some(String::from(created));
            }
        }
        if let Some(cap_group) = found.get(2) {
            if DATE_RGX.is_match(cap_group.as_str()) {
                let created = cap_group.as_str();
                let created = created.trim();
                Some(String::from(created))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn parse_priority(s: &str) -> Option<String> {
    let prio_found = PRIO_RGX.find(s);
    if let Some(found) = prio_found {
        let alpha_prio = found.as_str();
        let alpha_prio = alpha_prio
            .trim()
            .strip_prefix('(')
            .unwrap()
            .strip_suffix(')')
            .unwrap();
        Some(String::from(alpha_prio))
    } else {
        None
    }
}

fn parse_project(s: &str) -> Vec<String> {
    PROJ_RGX
        .find_iter(s)
        .map(|m| m.as_str().trim())
        .map(|s| s.into())
        .collect()
}

fn parse_context(s: &str) -> Vec<String> {
    CTX_RGX
        .find_iter(s)
        .map(|m| m.as_str().trim())
        .map(|s| s.into())
        .collect()
}

fn parse_meta(s: &str) -> HashMap<String, String> {
    META_RGX
        .find_iter(s)
        .map(|m| {
            let found = m.as_str();
            found.split(':').collect()
        })
        .fold(
            HashMap::new(),
            |mut mapped: HashMap<String, String>, vals: Vec<&str>| {
                mapped.insert(vals[0].into(), vals[1].into());
                mapped
            },
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_priorities() {
        let test_line = "x (A) Write unit tests for new project.";
        let parsed = parse_priority(test_line);
        assert_eq!(Some(String::from("A")), parsed);

        let no_prio = "Write unit tests for new project.";
        let parsed = parse_priority(no_prio);
        assert_eq!(None, parsed);

        let no_prio_opt = "x Write unit tests for new project.";
        let parsed = parse_priority(no_prio_opt);
        assert_eq!(None, parsed);

        let prio_no_spc = "(A)->Write unit tests for new project.";
        let parsed = parse_priority(prio_no_spc);
        assert_eq!(None, parsed);
    }

    #[test]
    fn parses_completed() {
        let test_line = "x 2011-03-03 Call Mom";
        let parsed = parse_completed(test_line);
        assert_eq!((true, Some(String::from("2011-03-03"))), parsed);

        let test_line = "x (A) Call Mom";
        let parsed = parse_completed(test_line);
        assert_eq!((true, None), parsed);

        let test_line = "x 2011-03-03 2011-03-01 Call Mom";
        let parsed = parse_completed(test_line);
        assert_eq!((true, Some(String::from("2011-03-03"))), parsed);
    }

    #[test]
    fn parses_created() {
        let test_line = "(A) 2011-03-03 Call Mom";
        let parsed = parse_created(test_line);
        assert_eq!(Some(String::from("2011-03-03")), parsed);

        let test_line = "(A) Call Mom";
        let parsed = parse_created(test_line);
        assert_eq!(None, parsed);

        let test_line = "2011-03-03 Call Mom";
        let parsed = parse_created(test_line);
        assert_eq!(Some(String::from("2011-03-03")), parsed);

        let test_line = "x (A) 2011-03-03 Call Mom";
        let parsed = parse_created(test_line);
        assert_eq!(Some(String::from("2011-03-03")), parsed);

        let test_line = "x 2011-03-04 (A) 2011-03-03 Call Mom";
        let parsed = parse_created(test_line);
        assert_eq!(Some(String::from("2011-03-03")), parsed);
    }

    #[test]
    fn parses_project() {
        let test_line = "+wiki update tags";
        let parsed = parse_project(test_line);
        assert_eq!(parsed, vec!["+wiki"]);

        let multi_project_tags = "+wiki +update +tags";
        let parsed = parse_project(multi_project_tags);
        assert_eq!(parsed, vec!["+wiki", "+update", "+tags"]);

        let middle_ctx = "wiki update tags +pkb";
        let parsed = parse_project(middle_ctx);
        assert_eq!(parsed, vec!["+pkb"]);

        let no_proj = "wiki update tags pkb";
        let parsed = parse_project(no_proj);
        let empty_vec: Vec<String> = Vec::new();
        assert_eq!(parsed, empty_vec);

        let middle_ctx = "+wiki-update +tags_pkb";
        let parsed = parse_project(middle_ctx);
        assert_eq!(parsed, vec!["+wiki-update", "+tags_pkb"]);

        let middle_ctx = "Learn how to add 2+2";
        let parsed = parse_project(middle_ctx);
        assert_eq!(parsed, empty_vec);
    }
    #[test]
    fn parses_context() {
        let test_line = "integrate todos into @tendril-wiki";
        let parsed = parse_context(test_line);
        assert_eq!(parsed, vec!["@tendril-wiki"]);

        let test_line = "integrate todos into @wiki";
        let parsed = parse_context(test_line);
        assert_eq!(parsed, vec!["@wiki"]);

        // FIXME
        let test_line = "@wiki_page todos need work";
        let parsed = parse_context(test_line);
        assert_eq!(parsed, vec!["@wiki_page"]);

        let test_line = "integrate todos into wiki_page";
        let parsed = parse_context(test_line);
        let empty_vec: Vec<String> = Vec::new();
        assert_eq!(parsed, empty_vec);

        let test_line = "Email SoAndSo at soandso@example.com";
        let parsed = parse_context(test_line);
        let empty_vec: Vec<String> = Vec::new();
        assert_eq!(parsed, empty_vec);
    }
    #[test]
    fn parsed_medata() {
        let test_line = "reticulate splines due:2022-03-14";
        let parsed = parse_meta(test_line);
        let test_map = HashMap::from([("due".into(), "2022-03-14".into())]);
        assert_eq!(parsed, test_map);

        let test_line = "Discuss new features with person:firstname-lastname";
        let parsed = parse_meta(test_line);
        let test_map = HashMap::from([("person".into(), "firstname-lastname".into())]);
        assert_eq!(parsed, test_map);
    }
    #[test]
    fn parses_full_task() {
        let test_line = "reticulate splines due:2022-03-14";
        let task = Task::from_str(test_line).unwrap();
        let expected_task = Task {
            completed: (false, None),
            project: Vec::new(),
            context: Vec::new(),
            created: None,
            priority: None,
            metadata: HashMap::from([("due".into(), "2022-03-14".into())]),
            body: test_line.into(),
        };
        assert_eq!(task, expected_task);

        let test_line = "x implement todos +tendril-wiki";
        let task = Task::from_str(test_line).unwrap();
        let expected_task = Task {
            completed: (true, None),
            created: None,
            project: vec!["+tendril-wiki".into()],
            context: Vec::new(),
            priority: None,
            metadata: HashMap::new(),
            body: test_line.into(),
        };
        assert_eq!(task, expected_task);

        let test_line = "(A) implement todo viewer +tendril-wiki @ui-ux";
        let task = Task::from_str(test_line).unwrap();
        let expected_task = Task {
            completed: (false, None),
            created: None,
            project: vec!["+tendril-wiki".into()],
            context: vec!["@ui-ux".into()],
            priority: Some("A".into()),
            metadata: HashMap::new(),
            body: test_line.into(),
        };
        assert_eq!(task, expected_task);

        let test_line = "(A) implement todo viewer +tendril-wiki @ui-ux due:2022-04-01";
        let task = Task::from_str(test_line).unwrap();
        let expected_task = Task {
            completed: (false, None),
            created: None,
            project: vec!["+tendril-wiki".into()],
            context: vec!["@ui-ux".into()],
            priority: Some("A".into()),
            metadata: HashMap::from([("due".into(), "2022-04-01".into())]),
            body: test_line.into(),
        };
        assert_eq!(task, expected_task);
    }
}
