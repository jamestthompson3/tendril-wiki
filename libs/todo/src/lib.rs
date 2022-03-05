use std::collections::HashMap;

use regex::Regex;
#[macro_use]
extern crate lazy_static;

// TODO: explore parsing with something like tendril and doing it based on a stream instead of
// regex

lazy_static! {
    static ref PRIO_RGX: Regex = Regex::new(r"\([[:upper:]]\)\s").unwrap();
    static ref PROJ_RGX: Regex = Regex::new(r"\+\S+").unwrap();
    static ref CTX_RGX: Regex = Regex::new(r"@\S+").unwrap();
    static ref META_RGX: Regex = Regex::new(r"\S+:\S+").unwrap();
    static ref CREATE_RGX: Regex = Regex::new(r"^(\([[:upper:]]\)\s)?(\d{4}-\d{2}-\d{2})").unwrap();
    static ref DONE_RGX: Regex =
        Regex::new(r"^x\s\d{4}-\d{2}-\d{2}\s(\([[:upper:]]\)|\d{4}-\d{2}-\d{2})?").unwrap();
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

impl From<&str> for Task {
    fn from(s: &str) -> Task {
        let completed = parse_completed(s);
        let project = parse_project(s);
        let context = parse_context(s);
        let priority = parse_priority(s);
        let metadata = parse_meta(s);
        let created = parse_created(s);
        Task {
            completed,
            created,
            project,
            priority,
            context,
            metadata,
            body: s.into(),
        }
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
    let created_found = CREATE_RGX.captures(s);
    if let Some(found) = created_found {
        let found = found.get(2).unwrap();
        let created = found.as_str();
        let created = created.trim();
        Some(String::from(created))
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
        .map(|m| {
            let found = m.as_str();
            found.strip_prefix('+').unwrap()
        })
        .map(|s| s.into())
        .collect()
}

fn parse_context(s: &str) -> Vec<String> {
    CTX_RGX
        .find_iter(s)
        .map(|m| {
            let found = m.as_str();
            found.strip_prefix('@').unwrap()
        })
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
    }

    #[test]
    fn parses_project() {
        let test_line = "+wiki update tags";
        let parsed = parse_project(test_line);
        assert_eq!(parsed, vec!["wiki"]);

        let multi_project_tags = "+wiki +update +tags";
        let parsed = parse_project(multi_project_tags);
        assert_eq!(parsed, vec!["wiki", "update", "tags"]);

        let middle_ctx = "wiki update tags +pkb";
        let parsed = parse_project(middle_ctx);
        assert_eq!(parsed, vec!["pkb"]);

        let no_proj = "wiki update tags pkb";
        let parsed = parse_project(no_proj);
        let empty_vec: Vec<String> = Vec::new();
        assert_eq!(parsed, empty_vec);

        let middle_ctx = "+wiki-update +tags_pkb";
        let parsed = parse_project(middle_ctx);
        assert_eq!(parsed, vec!["wiki-update", "tags_pkb"]);
    }
    #[test]
    fn parses_context() {
        let test_line = "integrate todos into @tendril-wiki";
        let parsed = parse_context(test_line);
        assert_eq!(parsed, vec!["tendril-wiki"]);

        let test_line = "integrate todos into @wiki";
        let parsed = parse_context(test_line);
        assert_eq!(parsed, vec!["wiki"]);

        let test_line = "@wiki_page todos need work";
        let parsed = parse_context(test_line);
        assert_eq!(parsed, vec!["wiki_page"]);

        let test_line = "integrate todos into wiki_page";
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
        let task = Task::from(test_line);
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
        let task = Task::from(test_line);
        let expected_task = Task {
            completed: (true, None),
            created: None,
            project: vec!["tendril-wiki".into()],
            context: Vec::new(),
            priority: None,
            metadata: HashMap::new(),
            body: test_line.into(),
        };
        assert_eq!(task, expected_task);

        let test_line = "(A) implement todo viewer +tendril-wiki @ui-ux";
        let task = Task::from(test_line);
        let expected_task = Task {
            completed: (false, None),
            created: None,
            project: vec!["tendril-wiki".into()],
            context: vec!["ui-ux".into()],
            priority: Some("A".into()),
            metadata: HashMap::new(),
            body: test_line.into(),
        };
        assert_eq!(task, expected_task);

        let test_line = "(A) implement todo viewer +tendril-wiki @ui-ux due:2022-04-01";
        let task = Task::from(test_line);
        let expected_task = Task {
            completed: (false, None),
            created: None,
            project: vec!["tendril-wiki".into()],
            context: vec!["ui-ux".into()],
            priority: Some("A".into()),
            metadata: HashMap::from([("due".into(), "2022-04-01".into())]),
            body: test_line.into(),
        };
        assert_eq!(task, expected_task);
    }
}
