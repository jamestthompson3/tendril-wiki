pub mod parse;

#[macro_use]
extern crate lazy_static;

use parse::{
    parse_completed, parse_context, parse_created, parse_meta, parse_priority, parse_project,
    META_RGX, PRIO_RGX,
};
use serde::{Deserialize, Serialize};
use wikitext::processors::sanitize_html;
use std::fmt::Write as _;
use std::{collections::HashMap, str::FromStr};
use thiserror::Error;

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

#[derive(Debug, PartialEq, Eq)]
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
        let priority = self
            .priority
            .to_owned()
            .unwrap_or_else(|| String::with_capacity(0));
        let created = self
            .created
            .to_owned()
            .unwrap_or_else(|| String::with_capacity(0));
        let created = if created.is_empty() {
            created
        } else {
            format!("<span title=\"created on {}\">{}<span>", created, created)
        };
        let metadata = self.format_metadata();
        let body = self.format_body_context(self.format_body());
        let str_idx = if idx.is_none() {
            String::with_capacity(0)
        } else {
            format!("data-idx=\"{}\"", idx.unwrap())
        };

        let checked = if self.completed.0 { "checked" } else { "" };
        let table_html = format!(
            r#"
<li role="row" {}>
    <div class="task-body">
          <input id="status" type="checkbox" {}>
           <span>
                <span class="priority">{}</span>
                {}
           </span>
          <span>
            <span class="edit-text-button">{}</span>
            <input type="text" class="edit-text-input hidden" value="{}" />
          </span>
    </div>
    <div>
        <span class="task-metadata edit-text-button">{}</span><input type="text" class="edit-text-input hidden" value="{}" />
        <div class="task-meta">
            {}
            <span class="status">{}</span>
            <span id="delete" aria-label="delete-task" title="delete task"></span>
        </div>
    </div>
</li>
"#,
            str_idx,
            checked,
            priority,
            construct_priority_input(&priority),
            body,
            self.format_body(),
            metadata,
            self.metadata
                .iter()
                .fold(String::new(), |mut formatted_str, (key, value)| {
                    let ctx_string = format!("{}:{}", key, value);
                    formatted_str.push_str(&ctx_string);
                    formatted_str
                }),
            created,
            self.format_status()
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
                                .trim()
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
                        if !prio.is_empty() {
                            let next_prio = format!("({}) ", prio);
                            self.body = PRIO_RGX.replace(&self.body, next_prio).into();
                            self.priority = Some(prio.clone());
                        }
                    }
                    None => {
                        let prio = if !prio.is_empty() {
                            format!("({}) ", prio)
                        } else {
                            String::with_capacity(0)
                        };
                        match &self.completed {
                            (true, Some(_)) => {
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
                let priority = if self.priority.as_ref().is_some() {
                    format!("({})", self.priority.as_ref().unwrap())
                } else {
                    String::with_capacity(0)
                };
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
                self.format_body_context(self.format_body())
            }
            UpdateType::Meta(metadata) => {
                for (key, value) in &self.metadata {
                    self.body = self.body.replace(&format!("{}:{}", key, value), "");
                }
                self.metadata = parse_meta(&metadata);
                for (key, value) in &self.metadata {
                    write!(self.body, " {}:{}", key, value).unwrap();
                }
                self.format_metadata()
            }
        }
    }

    fn format_body(&self) -> String {
        let mut formatted = sanitize_html(&self.body);
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

        META_RGX
            .find_iter(formatted.clone().as_str())
            .for_each(|m| {
                formatted = formatted.replace(m.as_str(), "");
            });
        formatted.trim().into()
    }
    fn format_body_context(&self, mut formatted: String) -> String {
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
                let ctx_string = format!("<strong>{}:</strong>&nbsp;{}&nbsp;", key, value);
                formatted_str.push_str(&ctx_string);
                formatted_str
            })
    }
}

const ALPHA_STRING: &str = "abcdefghijklmnopqrstuvwxyz";
fn construct_priority_input(priority: &str) -> String {
    let prio_opts = ALPHA_STRING
        .chars()
        .map(|c| format!("<option value=\"{}\">{}</option>", c, c.to_uppercase()))
        .collect::<Vec<String>>()
        .join("");
    format!(
        r#"
<select name="priority" class="hidden" id="priority-select">
    <option value="{}">{}</option>
    {}
</select>"#,
        priority, priority, prio_opts
    )
}
