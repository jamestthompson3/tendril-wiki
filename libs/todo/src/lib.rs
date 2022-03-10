pub mod parse;

#[macro_use]
extern crate lazy_static;

use parse::{
    parse_completed, parse_context, parse_created, parse_meta, parse_priority, parse_project,
    META_RGX, PRIO_RGX,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};
use thiserror::Error;

const FORBIDDEN_TAGS: [&str; 10] = [
    "<noscript>",
    "</noscript>",
    "<script>",
    "</script>",
    "<object>",
    "</object>",
    "<embed>",
    "</embed>",
    "<link>",
    "</link>",
];

// use this to prevent a million if let(Some) = ...  code branches in the `patch` method
#[derive(Debug, Serialize, Deserialize)]
pub enum UpdateType {
    Completion(CompletionState),
    Content(String),
    Prio(String),
    Meta(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskUpdate {
    pub completed: Option<CompletionState>,
    pub content: Option<String>,
    pub priority: Option<String>,
    pub metadata: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionState {
    done: bool,
    date: Option<String>,
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
    pub fn to_html(&self, idx: Option<usize>) -> String {
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
        let str_idx = if idx.is_none() {
            String::with_capacity(0)
        } else {
            format!("data-idx=\"{}\"", idx.unwrap())
        };

        let table_html = format!(
            r#"<tr role="row" {}><td tabindex="-1" aria-role="checkbox" aria-checked="{}">{}</td><td tabindex="-1" class="priority">{}</td><td tabindex="-1">{}</td><td tabindex="-1">{}</td><td tabindex="-1">{}</td></tr>"#,
            str_idx, self.completed.0, status, priority, created, body, metadata
        );
        html.push_str(&table_html);
        html
    }
    // FIXME: This is a bit of a dumpster fire, but let's getting working and then make it better
    // ;)
    pub fn patch(&mut self, update: UpdateType) -> String {
        match update {
            UpdateType::Completion(completed) => {
                match &self.completed {
                    (true, None) => {
                        match (&completed.done, &completed.date) {
                            (true, Some(date)) => {
                                self.completed = (true, Some(date.clone()));
                                // index at 2 since the `x ` will be at index 0 and 1
                                self.body.insert_str(1, date);
                            }
                            (false, None) => {
                                self.completed = (false, None);
                                self.body = self.body.strip_prefix("x ").unwrap().into();
                            }
                            (true, None) => {}
                            (false, Some(_)) => {
                                unreachable!("Should not have a completion date without the task being complete.");
                            }
                        }
                    }
                    (true, Some(date)) => match (&completed.done, &completed.date) {
                        (true, Some(_)) => {}
                        (false, None) => {
                            let completion_date = date.clone();
                            self.completed = (false, None);
                            self.body = self
                                .body
                                .strip_prefix(&format!("x {}", completion_date))
                                .unwrap()
                                .into();
                        }
                        (true, None) => {
                            let completion_date = date.clone();
                            self.completed = (false, None);
                            self.body = self.body.replace(&completion_date, "");
                        }
                        (false, Some(_)) => {
                            unreachable!("Should not have a completion date without the task being complete.");
                        }
                    },
                    (false, None) => match (&completed.done, &completed.date) {
                        (true, Some(date)) => {
                            self.completed = (true, Some(date.to_owned()));
                            self.body.insert_str(0, &format!("x {} ", date));
                        }
                        (false, None) => {}
                        (true, None) => {
                            self.completed = (true, None);
                            self.body.push_str("x ");
                        }
                        (false, Some(_)) => {
                            unreachable!("Should not have a completion date without the task being complete.");
                        }
                    },
                    (false, Some(_)) => {
                        unreachable!(
                            "Should not have a completion date without the task being complete."
                        )
                    }
                }
                self.format_status()
            }
            UpdateType::Prio(prio) => {
                match &self.priority {
                    Some(_) => {
                        let next_prio = if !prio.is_empty() {
                            format!("({}) ", prio)
                        } else {
                            String::with_capacity(0)
                        };
                        self.body = PRIO_RGX.replace(&self.body, next_prio).into();
                        self.priority = Some(prio.clone());
                    }
                    None => {
                        //TODO: match on completion status so we don't clobber that.
                        match &self.completed {
                            (true, Some(_)) => {
                                // index is 12 to account for: x YYYY-DD-MM
                                self.body.insert_str(12, &prio);
                            }
                            (true, None) => {
                                // index is 2 to account for: `x `
                                self.body.insert_str(2, &prio);
                            }
                            (false, None) => {
                                self.body.insert_str(0, &prio);
                            }
                            (false, Some(_)) => {
                                unreachable!(
                            "Should not have a completion date without the task being complete."
                        );
                            }
                        }
                    }
                }
                prio
            }
            UpdateType::Content(text) => {
                self.project = parse_project(&text);
                let completed = if self.completed.0 { "x" } else { "" };
                let priority = format!(
                    "({})",
                    self.priority.as_ref().unwrap_or(&String::with_capacity(0))
                );
                self.body = format!(
                    "{} {} {} {}",
                    completed,
                    self.completed
                        .1
                        .as_ref()
                        .unwrap_or(&String::with_capacity(0)),
                    priority,
                    text
                )
                .trim()
                .into();
                self.format_body()
            }
            UpdateType::Meta(metadata) => {
                for (key, value) in &self.metadata {
                    self.body = self.body.replace(&format!("{}:{}", key, value), "");
                }
                self.metadata = parse_meta(&metadata);
                for (key, value) in &self.metadata {
                    self.body.push_str(&format!(" {}:{}", key, value));
                }
                self.format_metadata()
            }
        }
    }

    fn sanitize_body(&self) -> String {
        let mut sanitized = self.body.clone();
        for tag in FORBIDDEN_TAGS {
            sanitized = sanitized.replace(tag, &cleaned(tag))
        }
        fn cleaned(tag: &str) -> String {
            tag.replace('>', "&gt;").replace('<', "&lt;")
        }
        sanitized
    }

    fn format_body(&self) -> String {
        let mut formatted = self.sanitize_body();
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
        META_RGX
            .find_iter(formatted.clone().as_str())
            .for_each(|m| {
                formatted = formatted.replace(m.as_str(), "");
            });
        formatted
    }

    fn format_status(&self) -> String {
        let (done, date) = &self.completed;
        if let true = done {
            if date.is_some() {
                format!("✅&nbsp;&nbsp;{}", date.as_ref().unwrap())
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
            r#"<a href="{}" class="{}">{}</a>"#,
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
