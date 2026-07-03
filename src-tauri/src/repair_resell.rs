use crate::db::{
    AppDatabase, RepairResellDealEstimateDraft, RepairResellDealEstimateRecord,
    RepairResellListingDraft, RepairResellListingRecord, RepairResellScrapeRunRecord,
    RepairResellSourceRecord,
};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::time::Duration;

const MAX_REFRESH_BODY_BYTES: u64 = 1_500_000;
const MAX_REFRESH_LISTINGS: usize = 25;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepairResellManualImportInput {
    pub source_id: String,
    pub canonical_url: String,
    pub title: String,
    pub description_text: String,
    pub source_category_text: Option<String>,
    pub condition_text: Option<String>,
    pub location_text: Option<String>,
    pub current_price_cents: Option<i64>,
    pub closing_at: Option<String>,
    pub pickup_text: Option<String>,
    pub inspection_text: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepairResellWatchlistInput {
    pub listing_id: String,
    pub is_watchlisted: bool,
    pub watch_status: Option<String>,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepairResellDealEstimateInput {
    pub listing_id: String,
    pub travel_profile_id: Option<String>,
    pub estimate_label: String,
    pub acquisition_price_cents: Option<i64>,
    pub buyer_premium_cents: Option<i64>,
    pub tax_cents: Option<i64>,
    pub travel_km: Option<f64>,
    pub fuel_cost_cents: Option<i64>,
    pub parts_cost_cents: Option<i64>,
    pub other_cost_cents: Option<i64>,
    pub expected_resale_low_cents: Option<i64>,
    pub expected_resale_high_cents: Option<i64>,
    pub expected_resale_target_cents: Option<i64>,
    pub desired_profit_cents: Option<i64>,
    pub risk_buffer_cents: Option<i64>,
    pub confidence: Option<String>,
    pub notes: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepairResellRefreshResult {
    pub source: RepairResellSourceRecord,
    pub run: RepairResellScrapeRunRecord,
    pub listings: Vec<RepairResellListingRecord>,
}

pub fn manual_import_listing(
    database: &AppDatabase,
    input: RepairResellManualImportInput,
) -> Result<RepairResellListingRecord, String> {
    if input.title.trim().is_empty() {
        return Err("Listing title is required.".to_string());
    }
    if input.canonical_url.trim().is_empty() {
        return Err("Listing URL is required.".to_string());
    }
    let draft = listing_draft_from_input(input)?;
    database
        .import_repair_resell_listing(draft, None)
        .map_err(|error| error.to_string())
}

pub fn refresh_source(
    database: &AppDatabase,
    source_id: String,
) -> Result<RepairResellRefreshResult, String> {
    let source = database
        .get_repair_resell_source_by_id(&source_id)
        .map_err(|error| error.to_string())?;
    let run = database
        .create_repair_resell_scrape_run(&source.id)
        .map_err(|error| error.to_string())?;

    if !source.enabled || source.scrape_mode != "public_http" {
        let finished = database
            .finish_repair_resell_scrape_run(
                &run.id,
                "skipped",
                0,
                0,
                0,
                1,
                "Source is disabled or manual-import only.",
            )
            .map_err(|error| error.to_string())?;
        return Ok(RepairResellRefreshResult {
            source,
            run: finished,
            listings: Vec::new(),
        });
    }

    let refresh_result = fetch_public_source_candidates(&source)
        .and_then(|candidates| import_refresh_candidates(database, &source, &run.id, candidates));

    match refresh_result {
        Ok(listings) => {
            let finished = database
                .finish_repair_resell_scrape_run(
                    &run.id,
                    "succeeded",
                    listings.len() as i64,
                    0,
                    listings.len() as i64,
                    0,
                    "",
                )
                .map_err(|error| error.to_string())?;
            Ok(RepairResellRefreshResult {
                source,
                run: finished,
                listings,
            })
        }
        Err(error) => {
            let finished = database
                .finish_repair_resell_scrape_run(&run.id, "failed", 0, 0, 0, 0, &error)
                .map_err(|finish_error| finish_error.to_string())?;
            Ok(RepairResellRefreshResult {
                source,
                run: finished,
                listings: Vec::new(),
            })
        }
    }
}

pub fn set_watchlist(
    database: &AppDatabase,
    input: RepairResellWatchlistInput,
) -> Result<RepairResellListingRecord, String> {
    database
        .set_repair_resell_listing_watchlist(
            &input.listing_id,
            input.is_watchlisted,
            input.watch_status.as_deref().unwrap_or("watching"),
            input.notes.as_deref().unwrap_or_default(),
        )
        .map_err(|error| error.to_string())
}

pub fn save_deal_estimate(
    database: &AppDatabase,
    input: RepairResellDealEstimateInput,
) -> Result<RepairResellDealEstimateRecord, String> {
    if input.estimate_label.trim().is_empty() {
        return Err("Estimate label is required.".to_string());
    }
    database
        .save_repair_resell_deal_estimate(RepairResellDealEstimateDraft {
            listing_id: input.listing_id,
            travel_profile_id: input.travel_profile_id,
            estimate_label: input.estimate_label,
            acquisition_price_cents: input.acquisition_price_cents,
            buyer_premium_cents: input.buyer_premium_cents,
            tax_cents: input.tax_cents,
            travel_km: input.travel_km,
            fuel_cost_cents: input.fuel_cost_cents,
            parts_cost_cents: input.parts_cost_cents,
            other_cost_cents: input.other_cost_cents,
            expected_resale_low_cents: input.expected_resale_low_cents,
            expected_resale_high_cents: input.expected_resale_high_cents,
            expected_resale_target_cents: input.expected_resale_target_cents,
            desired_profit_cents: input.desired_profit_cents,
            risk_buffer_cents: input.risk_buffer_cents,
            confidence: input.confidence.unwrap_or_else(|| "low".to_string()),
            notes: input.notes.unwrap_or_default(),
        })
        .map_err(|error| error.to_string())
}

fn listing_draft_from_input(
    input: RepairResellManualImportInput,
) -> Result<RepairResellListingDraft, String> {
    let title = input.title.trim().to_string();
    let description_text = input.description_text.trim().to_string();
    Ok(RepairResellListingDraft {
        source_id: input.source_id,
        external_id: None,
        canonical_url: input.canonical_url.trim().to_string(),
        title,
        description_text,
        source_category_text: input.source_category_text.unwrap_or_default(),
        condition_text: input.condition_text.unwrap_or_default(),
        location_text: input.location_text.unwrap_or_default(),
        current_price_cents: input.current_price_cents,
        closing_at: input.closing_at.unwrap_or_default(),
        pickup_text: input.pickup_text.unwrap_or_default(),
        inspection_text: input.inspection_text.unwrap_or_default(),
        listing_status: "unknown".to_string(),
        content_hash: content_hash(&[
            input.canonical_url.as_str(),
            input.title.as_str(),
            input.description_text.as_str(),
        ]),
        structured_json: "{}".to_string(),
    })
}

fn import_refresh_candidates(
    database: &AppDatabase,
    source: &RepairResellSourceRecord,
    run_id: &str,
    candidates: Vec<RefreshCandidate>,
) -> Result<Vec<RepairResellListingRecord>, String> {
    let mut listings = Vec::new();
    for candidate in candidates {
        let draft = RepairResellListingDraft {
            source_id: source.id.clone(),
            external_id: Some(candidate.external_id.clone()),
            canonical_url: candidate.url.clone(),
            title: candidate.title.clone(),
            description_text: format!("Discovered from {} manual refresh.", source.name),
            source_category_text: source.kind_label.clone(),
            condition_text: String::new(),
            location_text: source.region_label.clone(),
            current_price_cents: candidate.price_cents,
            closing_at: String::new(),
            pickup_text: String::new(),
            inspection_text: String::new(),
            listing_status: "unknown".to_string(),
            content_hash: content_hash(&[candidate.url.as_str(), candidate.title.as_str()]),
            structured_json: "{}".to_string(),
        };
        listings.push(
            database
                .import_repair_resell_listing(draft, Some(run_id))
                .map_err(|error| error.to_string())?,
        );
    }
    Ok(listings)
}

fn fetch_public_source_candidates(
    source: &RepairResellSourceRecord,
) -> Result<Vec<RefreshCandidate>, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(12))
        .user_agent("OverlayForge/0.9 local manual refresh")
        .build()
        .map_err(|error| error.to_string())?;
    let mut response = client
        .get(&source.base_url)
        .send()
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err(format!("Source returned HTTP status {}.", response.status()));
    }
    let mut body = String::new();
    response
        .by_ref()
        .take(MAX_REFRESH_BODY_BYTES)
        .read_to_string(&mut body)
        .map_err(|error| error.to_string())?;
    Ok(extract_listing_candidates(&source.base_url, &body))
}

#[derive(Clone)]
struct RefreshCandidate {
    external_id: String,
    url: String,
    title: String,
    price_cents: Option<i64>,
}

fn extract_listing_candidates(base_url: &str, html: &str) -> Vec<RefreshCandidate> {
    let mut candidates = Vec::new();
    let mut remaining = html;
    while let Some(anchor_start) = remaining.find("<a") {
        remaining = &remaining[anchor_start + 2..];
        let Some(tag_end) = remaining.find('>') else {
            break;
        };
        let tag = &remaining[..tag_end];
        remaining = &remaining[tag_end + 1..];
        let Some(anchor_end) = remaining.find("</a>") else {
            continue;
        };
        let text = strip_html(&remaining[..anchor_end]);
        remaining = &remaining[anchor_end + 4..];
        let Some(href) = extract_href(tag) else {
            continue;
        };
        let title = normalize_space(&text);
        if title.len() < 4 || !looks_like_listing(&title, &href) {
            continue;
        }
        let url = absolutize_url(base_url, &href);
        let external_id = content_hash(&[url.as_str()]);
        if candidates.iter().any(|candidate: &RefreshCandidate| candidate.url == url) {
            continue;
        }
        candidates.push(RefreshCandidate {
            external_id,
            url,
            title,
            price_cents: extract_price_cents(&text),
        });
        if candidates.len() >= MAX_REFRESH_LISTINGS {
            break;
        }
    }
    candidates
}

fn extract_href(tag: &str) -> Option<String> {
    for marker in ["href=\"", "href='"] {
        if let Some(start) = tag.find(marker) {
            let value_start = start + marker.len();
            let quote = marker.chars().last()?;
            let end = tag[value_start..].find(quote)?;
            return Some(tag[value_start..value_start + end].trim().to_string());
        }
    }
    None
}

fn looks_like_listing(title: &str, href: &str) -> bool {
    let haystack = format!("{} {}", title, href).to_lowercase();
    [
        "auction",
        "lot",
        "listing",
        "item",
        "bid",
        "bike",
        "mower",
        "snow",
        "tool",
        "generator",
        "compressor",
        "pressure",
        "electronics",
        "tractor",
    ]
    .iter()
    .any(|term| haystack.contains(term))
}

fn absolutize_url(base_url: &str, href: &str) -> String {
    if href.starts_with("http://") || href.starts_with("https://") {
        return href.to_string();
    }
    if href.starts_with("//") {
        return format!("https:{href}");
    }
    let origin = base_url
        .split_once("://")
        .and_then(|(scheme, rest)| rest.split('/').next().map(|host| format!("{scheme}://{host}")))
        .unwrap_or_else(|| base_url.trim_end_matches('/').to_string());
    if href.starts_with('/') {
        format!("{origin}{href}")
    } else {
        format!("{}/{}", base_url.trim_end_matches('/'), href)
    }
}

fn strip_html(value: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    for character in value.chars() {
        match character {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => output.push(character),
            _ => {}
        }
    }
    output
        .replace("&amp;", "&")
        .replace("&nbsp;", " ")
        .replace("&quot;", "\"")
}

fn normalize_space(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn extract_price_cents(value: &str) -> Option<i64> {
    let dollar = value.find('$')?;
    let mut number = String::new();
    for character in value[dollar + 1..].chars() {
        if character.is_ascii_digit() || character == '.' || character == ',' {
            number.push(character);
        } else if !number.is_empty() {
            break;
        }
    }
    let amount = number.replace(',', "").parse::<f64>().ok()?;
    Some((amount * 100.0).round() as i64)
}

fn content_hash(parts: &[&str]) -> String {
    let mut hasher = DefaultHasher::new();
    for part in parts {
        part.hash(&mut hasher);
    }
    format!("{:x}", hasher.finish())
}
