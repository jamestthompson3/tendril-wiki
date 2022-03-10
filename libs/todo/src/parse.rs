use std::collections::HashMap;

use regex::Regex;

// TODO: explore parsing with something like tendril and doing it based on a stream instead of
// regex

lazy_static! {
    pub(crate) static ref PRIO_RGX: Regex = Regex::new(r"\([[:upper:]]\)\s").unwrap();
    static ref PROJ_RGX: Regex = Regex::new(r"(^\+|\s\+)\S+").unwrap();
    static ref CTX_RGX: Regex = Regex::new(r"(^@|\s@)\S+").unwrap();
    pub(crate) static ref META_RGX: Regex = Regex::new(r"\S+:\S+").unwrap();
    // NOTE: This would be easier if the regex crate supported look behinds
    static ref CREATE_RGX: Regex = Regex::new(r"(?:^\([[:upper:]]\)\s)(\d{4}-\d{2}-\d{2})|(^\d{4}-\d{2}-\d{2})").unwrap();
    static ref DATE_RGX: Regex = Regex::new(r"^\d{4}-\d{2}-\d{2}").unwrap();
    static ref DONE_RGX: Regex =
        Regex::new(r"^x\s\d{4}-\d{2}-\d{2}\s(\([[:upper:]]\)|\d{4}-\d{2}-\d{2})?").unwrap();
}

pub(crate) fn parse_completed(s: &str) -> (bool, Option<String>) {
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

pub(crate) fn parse_created(s: &str) -> Option<String> {
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

pub(crate) fn capture_created(s: &str) -> Option<String> {
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

pub(crate) fn parse_priority(s: &str) -> Option<String> {
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

pub(crate) fn parse_project(s: &str) -> Vec<String> {
    PROJ_RGX
        .find_iter(s)
        .map(|m| m.as_str().trim())
        .map(|s| s.into())
        .collect()
}

pub(crate) fn parse_context(s: &str) -> Vec<String> {
    CTX_RGX
        .find_iter(s)
        .map(|m| m.as_str().trim())
        .map(|s| s.into())
        .collect()
}

pub(crate) fn parse_meta(s: &str) -> HashMap<String, String> {
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
    use crate::{CompletionState, Task, UpdateType};
    use std::str::FromStr;

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

    #[test]
    fn serialize_full_task() {
        let mut test_line = String::from("reticulate splines due:2022-03-14");
        let mut expected_task = Task {
            completed: (false, None),
            project: Vec::new(),
            context: Vec::new(),
            created: None,
            priority: None,
            metadata: HashMap::from([("due".into(), "2022-03-14".into())]),
            body: test_line.clone(),
        };
        expected_task.patch(UpdateType::Prio(String::from("(B)")));
        test_line.insert_str(0, "(B)");
        assert_eq!(expected_task.body, test_line);

        let test_line = "x implement todos +tendril-wiki";
        let mut expected_task = Task {
            completed: (true, None),
            created: None,
            project: vec!["+tendril-wiki".into()],
            context: Vec::new(),
            priority: None,
            metadata: HashMap::new(),
            body: test_line.into(),
        };
        expected_task.patch(UpdateType::Completion(CompletionState {
            done: false,
            date: None,
        }));
        assert_eq!(expected_task.body, test_line.strip_prefix("x ").unwrap());

        let test_line = "(A) implement todo viewer +tendril-wiki @ui-ux";
        let mut expected_task = Task {
            completed: (false, None),
            created: None,
            project: vec!["+tendril-wiki".into()],
            context: vec!["@ui-ux".into()],
            priority: Some("A".into()),
            metadata: HashMap::new(),
            body: test_line.into(),
        };
        expected_task.patch(UpdateType::Content(String::from(
            "redo patch functionality for +tendril-wiki",
        )));
        assert_eq!(
            expected_task.body,
            "(A) redo patch functionality for +tendril-wiki"
        );

        let test_line = "implement todo viewer +tendril-wiki @ui-ux due:2022-04-01";
        let mut expected_task = Task {
            completed: (false, None),
            created: None,
            project: vec!["+tendril-wiki".into()],
            context: vec!["@ui-ux".into()],
            priority: None,
            metadata: HashMap::from([("due".into(), "2022-04-01".into())]),
            body: test_line.into(),
        };
        expected_task.patch(UpdateType::Prio(String::new()));
        assert_eq!(expected_task.body, test_line);

        let test_line = "(A) implement todo viewer +tendril-wiki @ui-ux";
        let mut expected_task = Task {
            completed: (false, None),
            created: None,
            project: vec!["+tendril-wiki".into()],
            context: vec!["@ui-ux".into()],
            priority: Some("A".into()),
            metadata: HashMap::from([("due".into(), "2022-04-01".into())]),
            body: test_line.into(),
        };
        expected_task.patch(UpdateType::Meta(String::new()));
        assert_eq!(expected_task.body, test_line);

        let test_line = "(A) implement todo viewer +tendril-wiki @ui-ux due:2022-04-01";
        let mut expected_task = Task {
            completed: (false, None),
            created: None,
            project: vec!["+tendril-wiki".into()],
            context: vec!["@ui-ux".into()],
            priority: Some("A".into()),
            metadata: HashMap::new(),
            body: "(A) implement todo viewer +tendril-wiki @ui-ux".into(),
        };
        expected_task.patch(UpdateType::Meta(String::from("due:2022-04-01")));
        assert_eq!(expected_task.body, test_line);
    }
}
