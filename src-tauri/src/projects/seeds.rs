//! Curated from the named workspaces' README/status docs on 27 November 2026.
//! This is an initial snapshot, never a live status scanner. Aliases share one record.
use super::domain::{ProjectDraft, ProjectLink, ProjectStage};

pub fn catalog() -> Vec<(&'static str, ProjectDraft)> {
    use ProjectStage::*;
    let rows = [
        ("core", "Overlay Forge (core)", "Resident personal host with shared identity and embedded Media.", InProgress,
            "Add the private Projects catalog and review it alongside Media.", "", vec!["overlay-forge"]),
        ("media", "Media", "Tracks movies and series by episode. Google sign-in is integrated; UI polish and browser validation are in progress.", InProgress,
            "Finish display controls and browser UI validation against the host's pinned Media version.", "", vec!["overlay-forge-media"]),
        ("youtube", "YouTube", "Public video search, embedded playback and read-only account subscriptions.", InProgress,
            "Continue subscription and playback workflow refinement.", "", vec!["overlay-forge-youtube"]),
        ("hockey", "Hockey Card Companion", "Card inventory, collection imports and marketplace research; read-only COMC sync is integrated.", InProgress,
            "Review the current collection/research branch before extending listing alerts or catalogue work.", "", vec!["hockey-card-companion"]),
        ("retirement", "Retirement Planning", "Local retirement planning and paper research. Includes the separate RRSP research dashboard.", InProgress,
            "Review the planning and research dashboards; keep real transactions outside this catalog.", "", vec!["overlay-forge-retirement", "retirement-planner"]),
        ("side-hustles", "Side Hustles", "Local workspace for evaluating and validating independent business opportunities.", InProgress,
            "Continue opportunity comparison and validation.", "", vec!["overlay-forge-side-hustles"]),
        ("house-ready", "Real Estate / House Ready", "House preparation tracker. Cleanup is the active phase; larger projects remain deferred.", InProgress,
            "Continue the existing cleanup plan in HouseReady.", "", vec!["house-ready"]),
        ("world-editor", "7 Days to Die world viewer/editor", "Read-only Unity world viewer with terrain and Petrolia GIS previews; editing remains future work.", ReadyForReview,
            "Review Kibofa Valley's world alignment.", "Open Kibofa Valley, pan and zoom, and check a known road intersection, shoreline, and POI alignment.", vec!["7dtd-world-editor"]),
        ("lake-map", "Fictional lake map", "Planned fictional lake-map concept; no confirmed implementation or dedicated workspace.", Planned,
            "Define the map's purpose, geography and initial scope.", "", vec![]),
        ("ruckus", "Ruckus", "Petrolia layout validated at full scale. Local main is unborn with staged and untracked prototype work.", InProgress,
            "Complete road coverage and street naming/signage. Review the current local candidate before requesting another playtest.", "", vec!["ruckus"]),
        ("construction", "Unity construction sim", "Contractor Simulator: concrete-pad tutorial and structural basement-contract prototype.", InProgress,
            "Continue basement/foundation gameplay and review acceptance gaps.", "", vec!["contractor-simulator"]),
        ("gaming", "Gaming", "Standalone game workspace with screenshots, build guides and game overlays.", InProgress,
            "Continue product-owned guide and overlay workflows.", "", vec!["overlay-forge-gaming"]),
        ("unity-game", "Overlay Forge Unity Game", "Separate voxel-inspired game prototype with movement, camera switching and physics interactions.", InProgress,
            "Review the small playable foundation before expanding world systems.", "", vec!["overlay-forge-unity-game"]),
        ("custom-city", "7 Days to Die custom city / modpack", "Personal modpack and custom-city project with a user-accepted Sarnia / Port Huron world baseline.", InProgress,
            "Author highway pavement, grading, markings and roadside detail.", "", vec!["7dtd-custom-city"]),
        ("journal", "Journal", "Independent local journal app with its own backend and SQLite storage.", InProgress,
            "Review the existing local workflow before planning further work.", "", vec!["overlay-forge-journal"]),
        ("poe", "Path of Exile companion", "Icicle Mines Deadeye campaign checklist and overlay through the first T16 map.", InProgress,
            "Continue mapping and guide refinement after the campaign workflow.", "", vec!["overlay-forge-poe"]),
        ("minecraft", "Minecraft companion", "Java core and Fabric client adapter for a standalone Minecraft companion.", InProgress,
            "Review the current core and client-adapter milestone before expanding coaching features.", "", vec!["overlay-forge-minecraft"]),
        ("valheim", "Valheim companion", "Windows overlay with a local catalog of BepInEx capabilities.", InProgress,
            "Continue capability catalog and overlay refinement.", "", vec!["overlay-forge-valheim"]),
        ("enshrouded", "Enshrouded Asset Explorer", "Local asset-explorer proof of concept with resource indexing and texture previews.", InProgress,
            "Review the existing asset explorer before extending supported resources.", "", vec!["overlay-forge-enshrouded"]),
        ("vampire-survivors", "Vampire Survivors companion", "Planned optional level-up advisor; existing documentation records progression and unlocks.", Planned,
            "Define the first advisor milestone from the existing progression notes.", "", vec!["overlay-forge-vampire-survivors"]),
        ("web-observatory", "Web Script Observatory", "Local browser JavaScript observation and correlation tools at an initial observation milestone.", InProgress,
            "Review the observation milestone and its documented limitations.", "", vec!["overlay-forge-web-observatory"]),
        ("poker-trainer", "Tablecraft Poker Trainer", "Local play-money pre-flop trainer with equity estimates and decision scoring.", InProgress,
            "Review training scenarios and decision feedback.", "", vec!["poker-trainer"]),
        ("zomboid", "Zomboid Companion", "Standalone Windows companion for Project Zomboid with its own local data.", InProgress,
            "Review the current companion workflow and research backlog.", "", vec!["zomboid-companion"]),
        ("7-days-direct", "7 Days to Direct", "Planning placeholder for an optional world-agnostic director and scenario toolkit. No mod implementation yet.", Planned,
            "Define the first bounded director/scenario capability.", "", vec!["7dtd-director-toolkit"]),
    ];
    rows.into_iter()
        .map(|(key, title, description, stage, next, waiting, repos)| {
            (
                key,
                ProjectDraft {
                    title: title.into(),
                    description: description.into(),
                    stage,
                    next_step: next.into(),
                    waiting_on_rob: waiting.into(),
                    links: repos
                        .into_iter()
                        .map(|repo| ProjectLink {
                            label: format!("Workspace · {repo}"),
                            target: format!("C:\\Dev\\Repos\\{repo}"),
                        })
                        .collect(),
                },
            )
        })
        .collect()
}
