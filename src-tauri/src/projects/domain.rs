use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub enum ProjectStage {
    Idea,
    Planned,
    #[serde(rename = "In progress")]
    InProgress,
    #[serde(rename = "Ready for review")]
    ReadyForReview,
    #[serde(rename = "On hold")]
    OnHold,
    Completed,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectLink {
    pub label: String,
    pub target: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectDraft {
    pub title: String,
    pub description: String,
    pub stage: ProjectStage,
    pub next_step: String,
    pub waiting_on_rob: String,
    pub links: Vec<ProjectLink>,
}

impl ProjectDraft {
    pub fn validate(mut self) -> Result<Self, String> {
        self.title = self.title.trim().to_owned();
        self.description = self.description.trim().to_owned();
        self.next_step = self.next_step.trim().to_owned();
        self.waiting_on_rob = self.waiting_on_rob.trim().to_owned();
        if self.title.is_empty() || self.title.len() > 160 {
            return Err("Enter a project title of at most 160 bytes.".into());
        }
        if [&self.description, &self.next_step, &self.waiting_on_rob]
            .iter()
            .any(|value| value.len() > 2000)
            || self.links.len() > 12
        {
            return Err("Keep notes within 2,000 bytes each and use at most 12 links.".into());
        }
        for link in &mut self.links {
            link.label = link.label.trim().to_owned();
            link.target = link.target.trim().to_owned();
            if link.label.is_empty()
                || link.label.len() > 160
                || link.target.len() > 2048
                || link.target.chars().any(char::is_control)
                || !valid_target(&link.target)
            {
                return Err(
                    "Each link needs a label and an HTTPS URL or absolute workspace/document path."
                        .into(),
                );
            }
        }
        Ok(self)
    }
}

// Paths are inert references. They are never passed to a process or filesystem operation.
fn valid_target(target: &str) -> bool {
    if let Ok(url) = reqwest::Url::parse(target) {
        if url.scheme() == "https" {
            return url.host_str().is_some()
                && url.username().is_empty()
                && url.password().is_none();
        }
    }
    target.starts_with('/')
        || (target.len() >= 3
            && target.as_bytes()[0].is_ascii_alphabetic()
            && target.as_bytes()[1] == b':'
            && matches!(target.as_bytes()[2], b'/' | b'\\'))
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    #[serde(flatten)]
    pub draft: ProjectDraft,
    pub archived: bool,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectQuery {
    #[serde(default)]
    pub search: String,
    pub stage: Option<ProjectStage>,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub waiting_only: bool,
}

impl ProjectQuery {
    pub fn matches(&self, project: &Project) -> bool {
        project.archived == self.archived
            && self.stage.is_none_or(|stage| stage == project.draft.stage)
            && (!self.waiting_only || !project.draft.waiting_on_rob.trim().is_empty())
            && [
                &project.draft.title,
                &project.draft.description,
                &project.draft.next_step,
                &project.draft.waiting_on_rob,
            ]
            .into_iter()
            .chain(
                project
                    .draft
                    .links
                    .iter()
                    .flat_map(|link| [&link.label, &link.target]),
            )
            .any(|value| {
                value
                    .to_lowercase()
                    .contains(&self.search.trim().to_lowercase())
            })
    }
}
