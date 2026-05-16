use serde::Deserialize;

const GITHUB_REPOSITORY_API_BASE: &str = "https://api.github.com/repos";
const USER_AGENT: &str = "overlay-forge-milestone-4";

#[derive(Deserialize)]
struct GitHubRepositoryResponse {
    full_name: String,
    html_url: String,
    default_branch: String,
    visibility: Option<String>,
    private: bool,
}

#[derive(Deserialize)]
struct GitHubErrorBody {
    message: Option<String>,
}

pub struct GitHubRepositoryMetadata {
    pub repository_full_name: String,
    pub repository_url: String,
    pub default_branch: String,
    pub visibility: String,
}

pub async fn fetch_repository_metadata(
    repository_full_name: &str,
) -> Result<GitHubRepositoryMetadata, String> {
    let clean_name = normalize_repository_full_name(repository_full_name)?;
    let token = std::env::var("GITHUB_TOKEN")
        .map(|value| value.trim().to_string())
        .unwrap_or_default();

    if token.is_empty() {
        return Err(
            "GitHub token is not configured. Set GITHUB_TOKEN and restart Overlay Forge."
                .to_string(),
        );
    }

    let response = reqwest::Client::new()
        .get(format!("{GITHUB_REPOSITORY_API_BASE}/{clean_name}"))
        .bearer_auth(token)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .send()
        .await
        .map_err(|error| format!("GitHub request failed: {error}"))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| format!("GitHub response could not be read: {error}"))?;

    if !status.is_success() {
        if let Ok(error_body) = serde_json::from_str::<GitHubErrorBody>(&body) {
            if let Some(message) = error_body.message {
                return Err(format!("GitHub request failed: {message}"));
            }
        }

        return Err(format!("GitHub request failed with status {status}"));
    }

    let repository = serde_json::from_str::<GitHubRepositoryResponse>(&body)
        .map_err(|error| format!("GitHub response JSON was invalid: {error}"))?;

    Ok(GitHubRepositoryMetadata {
        repository_full_name: repository.full_name,
        repository_url: repository.html_url,
        default_branch: repository.default_branch,
        visibility: repository.visibility.unwrap_or_else(|| {
            if repository.private {
                "private"
            } else {
                "public"
            }
            .to_string()
        }),
    })
}

pub fn normalize_repository_full_name(repository_full_name: &str) -> Result<String, String> {
    let clean_name = repository_full_name.trim();
    let parts = clean_name.split('/').collect::<Vec<_>>();

    if parts.len() != 2
        || parts.iter().any(|part| part.trim().is_empty())
        || parts.iter().any(|part| !is_safe_repository_part(part))
        || clean_name.chars().any(char::is_whitespace)
    {
        return Err("GitHub repository must use the owner/repository-name format.".to_string());
    }

    Ok(clean_name.to_string())
}

fn is_safe_repository_part(value: &str) -> bool {
    value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-'))
}
