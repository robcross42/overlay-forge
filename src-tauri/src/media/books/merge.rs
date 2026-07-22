use super::domain::{BookCatalogResult, NormalizedBookImport};
use std::collections::{HashMap, HashSet};

pub struct BookMetadataMerger;

impl BookMetadataMerger {
    pub fn merge(base: &mut NormalizedBookImport, enrichment: NormalizedBookImport) {
        fill_string(&mut base.subtitle, enrichment.subtitle);
        fill_string(&mut base.description, enrichment.description);
        fill_string(
            &mut base.canonical_cover_url,
            enrichment.canonical_cover_url,
        );
        if base.first_publish_year.is_none() {
            base.first_publish_year = enrichment.first_publish_year;
        }
        if base.community_rating.is_none() {
            base.community_rating = enrichment.community_rating;
        }
        if base.community_rating_count.is_none() {
            base.community_rating_count = enrichment.community_rating_count;
        }
        append_unique(&mut base.authors, enrichment.authors);
        append_unique(&mut base.subjects, enrichment.subjects);

        for edition in enrichment.editions {
            let duplicate = base.editions.iter().any(|existing| {
                (!edition.isbn_13.is_empty() && existing.isbn_13 == edition.isbn_13)
                    || (!edition.isbn_10.is_empty() && existing.isbn_10 == edition.isbn_10)
            });
            if !duplicate {
                base.editions.push(edition);
            }
        }
        let mut source_keys = base
            .sources
            .iter()
            .map(|source| {
                format!(
                    "{}:{}:{}",
                    source.source_key, source.entity_type, source.external_id
                )
            })
            .collect::<HashSet<_>>();
        for source in enrichment.sources {
            let key = format!(
                "{}:{}:{}",
                source.source_key, source.entity_type, source.external_id
            );
            if source_keys.insert(key) {
                base.sources.push(source);
            }
        }
        let mut link_keys = base
            .links
            .iter()
            .map(|link| format!("{}:{}", link.link_type, link.url))
            .collect::<HashSet<_>>();
        for link in enrichment.links {
            if link_keys.insert(format!("{}:{}", link.link_type, link.url)) {
                base.links.push(link);
            }
        }
        base.series.extend(enrichment.series);
    }
}

pub fn merge_catalog_results(results: Vec<BookCatalogResult>) -> Vec<BookCatalogResult> {
    let mut merged = Vec::<BookCatalogResult>::new();
    let mut indexes = HashMap::<String, usize>::new();
    for candidate in results {
        let key = candidate.dedupe_key();
        if let Some(index) = indexes.get(&key).copied() {
            merge_candidate(&mut merged[index], candidate);
        } else {
            indexes.insert(key, merged.len());
            merged.push(candidate);
        }
    }
    merged
}

fn merge_candidate(base: &mut BookCatalogResult, candidate: BookCatalogResult) {
    if base.source_key != "google_books" && candidate.source_key == "google_books" {
        let mut preferred = candidate;
        preferred
            .source_identities
            .extend(base.source_identities.clone());
        append_unique(&mut preferred.provider_badges, base.provider_badges.clone());
        *base = preferred;
        return;
    }
    fill_string(&mut base.subtitle, candidate.subtitle);
    fill_string(&mut base.description, candidate.description);
    fill_string(&mut base.publisher, candidate.publisher);
    fill_string(&mut base.published_date, candidate.published_date);
    fill_string(&mut base.isbn_10, candidate.isbn_10);
    fill_string(&mut base.isbn_13, candidate.isbn_13);
    fill_string(&mut base.cover_url, candidate.cover_url);
    fill_string(&mut base.work_key, candidate.work_key);
    fill_string(&mut base.edition_key, candidate.edition_key);
    fill_string(&mut base.info_url, candidate.info_url);
    fill_string(&mut base.preview_url, candidate.preview_url);
    if base.page_count.is_none() {
        base.page_count = candidate.page_count;
    }
    append_unique(&mut base.authors, candidate.authors);
    append_unique(&mut base.provider_badges, candidate.provider_badges);
    for identity in candidate.source_identities {
        if !base.source_identities.iter().any(|existing| {
            existing.source_key == identity.source_key
                && existing.entity_type == identity.entity_type
                && existing.external_id == identity.external_id
        }) {
            base.source_identities.push(identity);
        }
    }
}

fn fill_string(target: &mut String, candidate: String) {
    if target.trim().is_empty() && !candidate.trim().is_empty() {
        *target = candidate;
    }
}

fn append_unique(target: &mut Vec<String>, candidates: Vec<String>) {
    for candidate in candidates {
        if !candidate.trim().is_empty()
            && !target
                .iter()
                .any(|existing| existing.eq_ignore_ascii_case(&candidate))
        {
            target.push(candidate);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::books::domain::{BookFormat, NormalizedBookEdition};

    fn import(description: &str, _source: &str) -> NormalizedBookImport {
        NormalizedBookImport {
            title: "Title".to_string(),
            subtitle: String::new(),
            description: description.to_string(),
            authors: vec![],
            first_publish_year: None,
            subjects: vec![],
            canonical_cover_url: String::new(),
            community_rating: None,
            community_rating_count: None,
            editions: vec![NormalizedBookEdition {
                title: "Title".to_string(),
                subtitle: String::new(),
                format: BookFormat::Unknown,
                isbn_10: String::new(),
                isbn_13: String::new(),
                publisher: String::new(),
                published_date: String::new(),
                language: String::new(),
                page_count: None,
                audio_duration_minutes: None,
                cover_url: String::new(),
                is_ebook: false,
                access_viewability: String::new(),
            }],
            sources: vec![],
            links: vec![],
            series: vec![],
        }
    }

    #[test]
    fn empty_enrichment_never_overwrites_populated_metadata() {
        let mut base = import("Useful description", "google_books");
        BookMetadataMerger::merge(&mut base, import("", "open_library"));
        assert_eq!(base.description, "Useful description");
    }

    fn candidate(source: &str, id: &str, isbn: &str, title: &str) -> BookCatalogResult {
        BookCatalogResult {
            source_key: source.to_string(),
            external_id: id.to_string(),
            title: title.to_string(),
            subtitle: String::new(),
            authors: vec!["Author".to_string()],
            published_date: "2020".to_string(),
            publisher: String::new(),
            isbn_10: String::new(),
            isbn_13: isbn.to_string(),
            page_count: None,
            language: String::new(),
            format: BookFormat::Unknown,
            cover_url: String::new(),
            description: String::new(),
            work_key: String::new(),
            edition_key: String::new(),
            info_url: String::new(),
            preview_url: String::new(),
            already_in_library: false,
            existing_entry_id: None,
            match_basis: String::new(),
            provider_badges: vec![source.to_string()],
            source_identities: Vec::new(),
        }
    }

    #[test]
    fn exact_isbn_results_merge_but_title_only_candidates_do_not() {
        let merged = merge_catalog_results(vec![
            candidate("open_library", "ol", "9780306406157", "Same"),
            candidate("google_books", "google", "9780306406157", "Same"),
            candidate("open_library", "title-only", "", "Same"),
        ]);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].source_key, "google_books");
    }

    #[test]
    fn provider_identity_work_key_and_isbn10_are_high_confidence_matches() {
        let same_google = merge_catalog_results(vec![
            candidate("google_books", "volume-1", "", "First title"),
            candidate("google_books", "volume-1", "", "Updated title"),
        ]);
        assert_eq!(same_google.len(), 1);

        let mut first_work = candidate("open_library", "edition-1", "", "Work");
        first_work.work_key = "OL1W".to_string();
        let mut second_work = candidate("open_library", "edition-2", "", "Work");
        second_work.work_key = "OL1W".to_string();
        assert_eq!(
            merge_catalog_results(vec![first_work, second_work]).len(),
            1
        );

        let mut first_isbn10 = candidate("google_books", "g", "", "ISBN-10");
        first_isbn10.isbn_10 = "0306406152".to_string();
        let mut second_isbn10 = candidate("open_library", "ol", "", "ISBN-10");
        second_isbn10.isbn_10 = "0306406152".to_string();
        assert_eq!(
            merge_catalog_results(vec![first_isbn10, second_isbn10]).len(),
            1
        );
    }
}
