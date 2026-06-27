use crate::db::{
    AppDatabase, GameRuntimeConstructionExportRecord, GameRuntimePartAliasRecord,
    GameRuntimePartAttachmentTypeRecord, GameRuntimePartInstanceRecord,
    GameRuntimePartMetadataValueRecord, GameRuntimePartOutputChannelValueRecord,
    GameRuntimePartSettingValueRecord,
};
use std::collections::HashMap;

pub struct GearBlocksSceneContextService<'a> {
    database: &'a AppDatabase,
}

impl<'a> GearBlocksSceneContextService<'a> {
    pub fn new(database: &'a AppDatabase) -> Self {
        Self { database }
    }

    pub fn render_current_scene_context(&self, game_id: i64) -> Result<Option<String>, String> {
        let Some(export) = self
            .database
            .latest_game_runtime_construction_export(game_id)
            .map_err(|error| error.to_string())?
        else {
            return Ok(None);
        };
        let instances = self
            .database
            .list_game_runtime_part_instances(game_id)
            .map_err(|error| error.to_string())?;
        if instances.is_empty() {
            return Ok(None);
        }

        let aliases = self
            .database
            .list_game_runtime_part_aliases(game_id)
            .map_err(|error| error.to_string())?;
        let details = GearBlocksSceneDetailIndex::load(self.database, game_id)?;
        let parts = build_scene_parts(instances, &aliases);

        let sections = vec![
            "# GearBlocks Runtime Scene Context".to_string(),
            "Source: normalized SQLite runtime scene rows. Treat part keys, instance keys, construction ids, runtime ids, and indexes as database references for the current scene. Raw full-scene export JSON is not retained after import."
                .to_string(),
            scene_source_section(&export, parts.len()),
            system_counts_section(&parts),
            inventory_section("Functional inventory by system", &parts, false),
            inventory_section("Structural inventory", &parts, true),
            part_aliases_section(&parts, &aliases),
            coordinate_reference_section(&parts),
            construction_groups_section(&parts),
            build_guide_api_context_section(),
            definition_details_section(&parts, &details),
            structural_bounds_section(&parts),
            functional_parts_section(&parts),
        ];

        Ok(Some(sections.join("\n\n")))
    }
}

struct GearBlocksScenePart {
    record: GameRuntimePartInstanceRecord,
    friendly_name: Option<String>,
    system: &'static str,
    purpose: &'static str,
    behaviours: Vec<String>,
    is_structural: bool,
    is_functional: bool,
}

struct GearBlocksSceneDetailIndex {
    metadata_by_part: HashMap<String, Vec<GameRuntimePartMetadataValueRecord>>,
    attachments_by_part: HashMap<String, Vec<GameRuntimePartAttachmentTypeRecord>>,
    settings_by_part: HashMap<String, Vec<GameRuntimePartSettingValueRecord>>,
    outputs_by_part: HashMap<String, Vec<GameRuntimePartOutputChannelValueRecord>>,
}

impl GearBlocksSceneDetailIndex {
    fn load(database: &AppDatabase, game_id: i64) -> Result<Self, String> {
        Ok(Self {
            metadata_by_part: group_by_part_key(
                database
                    .list_game_runtime_part_metadata_values(game_id)
                    .map_err(|error| error.to_string())?,
                |row| row.part_key.clone(),
            ),
            attachments_by_part: group_by_part_key(
                database
                    .list_game_runtime_part_attachment_types(game_id)
                    .map_err(|error| error.to_string())?,
                |row| row.part_key.clone(),
            ),
            settings_by_part: group_by_part_key(
                database
                    .list_game_runtime_part_setting_values(game_id)
                    .map_err(|error| error.to_string())?,
                |row| row.part_key.clone(),
            ),
            outputs_by_part: group_by_part_key(
                database
                    .list_game_runtime_part_output_channel_values(game_id)
                    .map_err(|error| error.to_string())?,
                |row| row.part_key.clone(),
            ),
        })
    }
}

fn group_by_part_key<T, F>(rows: Vec<T>, key_for: F) -> HashMap<String, Vec<T>>
where
    F: Fn(&T) -> String,
{
    let mut grouped = HashMap::new();
    for row in rows {
        grouped
            .entry(key_for(&row))
            .or_insert_with(Vec::new)
            .push(row);
    }
    grouped
}

fn build_scene_parts(
    instances: Vec<GameRuntimePartInstanceRecord>,
    aliases: &[GameRuntimePartAliasRecord],
) -> Vec<GearBlocksScenePart> {
    let alias_map = aliases
        .iter()
        .map(|alias| (alias.part_instance_key.clone(), alias.friendly_name.clone()))
        .collect::<HashMap<_, _>>();

    instances
        .into_iter()
        .map(|record| {
            let behaviours = behaviour_names(&record.behaviour_names_json);
            let behaviour_text = behaviours.join(" ").to_ascii_lowercase();
            let combined_text = format!(
                "{} {} {} {} {}",
                part_name(&record),
                record.category,
                behaviour_text,
                record.asset_name,
                record.full_display_name
            )
            .to_ascii_lowercase();
            let category = record.category.to_ascii_lowercase();
            let system = classify_part_system(&combined_text, &category, &behaviours);
            let purpose = part_purpose(system, &combined_text, &behaviour_text);
            let is_structural = system == "structural frame" || system == "mounts and connectors";
            let is_functional =
                !is_structural || !behaviours.is_empty() || record.link_node_count > 0;
            let friendly_name = alias_map.get(&record.part_instance_key).cloned();
            GearBlocksScenePart {
                record,
                friendly_name,
                system,
                purpose,
                behaviours,
                is_structural,
                is_functional,
            }
        })
        .collect()
}

fn scene_source_section(export: &GameRuntimeConstructionExportRecord, part_count: usize) -> String {
    format!(
        "## Scene Snapshot\nExport `{}` from `{}`. Intended output path: `{}`. Exported at: `{}`.\nConstruction ID: {}. Current instance rows: {}. Runtime-reported parts: {}. Mass: {:.2}. Frozen: {}. Invulnerable: {}. Player character: {}.",
        export.export_id,
        export.source_log_path,
        export.intended_path,
        export.exported_at,
        export.construction_id,
        part_count,
        export.part_count,
        export.mass,
        option_bool_label(export.is_frozen),
        option_bool_label(export.is_invulnerable),
        option_bool_label(export.is_player_character)
    )
}

fn system_counts_section(parts: &[GearBlocksScenePart]) -> String {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    let mut mass_by_system: HashMap<&str, f64> = HashMap::new();
    for part in parts {
        *counts.entry(part.system).or_default() += 1;
        *mass_by_system.entry(part.system).or_default() += part.record.mass;
    }

    let mut systems = counts.keys().copied().collect::<Vec<_>>();
    systems.sort_unstable();
    let lines = systems
        .into_iter()
        .map(|system| {
            format!(
                "- {}: {} part(s), {:.2} mass",
                system,
                counts.get(system).copied().unwrap_or_default(),
                mass_by_system.get(system).copied().unwrap_or_default()
            )
        })
        .collect::<Vec<_>>();

    format!("## System Counts\n{}", lines.join("\n"))
}

fn inventory_section(title: &str, parts: &[GearBlocksScenePart], structural_only: bool) -> String {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for part in parts {
        if structural_only != part.is_structural {
            continue;
        }
        let key = format!("{} | {}", part.system, part_name(&part.record));
        *counts.entry(key).or_default() += 1;
    }

    if counts.is_empty() {
        return format!("## {title}\nNo matching parts identified.");
    }

    let mut rows = counts.into_iter().collect::<Vec<_>>();
    rows.sort_by(|left, right| left.0.cmp(&right.0));
    let lines = rows
        .into_iter()
        .take(80)
        .map(|(name, count)| format!("- {} x{}", name, count))
        .collect::<Vec<_>>();

    format!("## {title}\n{}", lines.join("\n"))
}

fn part_aliases_section(
    parts: &[GearBlocksScenePart],
    aliases: &[GameRuntimePartAliasRecord],
) -> String {
    if aliases.is_empty() {
        return "## Friendly Part Names\nNo friendly part names have been imported yet."
            .to_string();
    }

    let part_map = parts
        .iter()
        .map(|part| (part.record.part_instance_key.clone(), part))
        .collect::<HashMap<_, _>>();
    let mut lines = Vec::new();
    for alias in aliases.iter().take(120) {
        if let Some(part) = part_map.get(&alias.part_instance_key) {
            lines.push(format!(
                "- `{}` = {} [{} / {}]; {}; {}; part_key={}; instance={}",
                alias.friendly_name,
                part_reference(part),
                part.system,
                part.record.category,
                part_name(&part.record),
                position_pair_text(&part.record),
                part.record.part_key,
                alias.part_instance_key
            ));
        } else {
            let label = if alias.full_display_name.trim().is_empty() {
                alias.display_name.as_str()
            } else {
                alias.full_display_name.as_str()
            };
            lines.push(format!(
                "- `{}` = {} [{} / {}]; last seen {}; instance={} (not present in latest runtime scene rows)",
                alias.friendly_name,
                label,
                alias.category,
                alias.asset_name,
                alias.last_seen_at,
                alias.part_instance_key
            ));
        }
    }

    if aliases.len() > lines.len() {
        lines.push(format!(
            "- {} additional friendly part name(s) omitted from prompt context for size.",
            aliases.len() - lines.len()
        ));
    }

    format!(
        "## Friendly Part Names\nUse these aliases when the user refers to exact physical parts. Do not apply an alias to all parts of the same catalog type unless the user asks for that.\n{}",
        lines.join("\n")
    )
}

fn coordinate_reference_section(parts: &[GearBlocksScenePart]) -> String {
    let mut sorted_parts = parts.iter().collect::<Vec<_>>();
    sorted_parts.sort_by(|left, right| {
        left.record
            .runtime_part_index
            .cmp(&right.record.runtime_part_index)
            .then(part_name(&left.record).cmp(&part_name(&right.record)))
            .then(left.record.category.cmp(&right.record.category))
    });

    let mut lines = Vec::new();
    for part in sorted_parts.iter().take(320) {
        lines.push(format!(
            "- {} [{} / {}] {}: {}{}; part_key={}; instance={}",
            part_reference(part),
            part.system,
            part.record.category,
            part_label(part),
            position_pair_text(&part.record),
            size_text(&part.record),
            part.record.part_key,
            part.record.part_instance_key
        ));
    }

    if parts.len() > lines.len() {
        lines.push(format!(
            "- {} additional part coordinate row(s) omitted from prompt context for size.",
            parts.len() - lines.len()
        ));
    }

    if lines.is_empty() {
        "## Runtime Coordinate Reference\nNo runtime parts were available for coordinate reference."
            .to_string()
    } else {
        format!(
            "## Runtime Coordinate Reference\nCoordinates are GearBlocks units; 1 unit equals 10 cm. Use coordinates for spatial reasoning, measurements, and part identification. Do not request or emit Overlay Forge marker blocks; in-game visual markers are disabled for now.\n{}",
            lines.join("\n")
        )
    }
}

fn construction_groups_section(parts: &[GearBlocksScenePart]) -> String {
    let mut groups: HashMap<String, Vec<&GearBlocksScenePart>> = HashMap::new();
    for part in parts
        .iter()
        .filter(|part| !part.record.source_construction_id.trim().is_empty())
    {
        groups
            .entry(part.record.source_construction_id.clone())
            .or_default()
            .push(part);
    }

    if groups.is_empty() {
        return "## Runtime Construction Groups\nNo parent construction groups were exposed in the latest runtime scene rows.".to_string();
    }

    let mut rows = groups.into_iter().collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        left.0
            .parse::<i64>()
            .ok()
            .cmp(&right.0.parse::<i64>().ok())
            .then(left.0.cmp(&right.0))
    });

    let mut lines = Vec::new();
    for (construction_id, mut group_parts) in rows.into_iter().take(120) {
        group_parts.sort_by(|left, right| {
            left.record
                .runtime_part_index
                .cmp(&right.record.runtime_part_index)
                .then(
                    left.record
                        .runtime_part_id
                        .cmp(&right.record.runtime_part_id),
                )
                .then(part_name(&left.record).cmp(&part_name(&right.record)))
        });

        let mut inventory: HashMap<String, Vec<String>> = HashMap::new();
        for part in &group_parts {
            inventory
                .entry(part_name(&part.record))
                .or_default()
                .push(format!(
                    "#{} idx {}",
                    part.record.runtime_part_id, part.record.runtime_part_index
                ));
        }
        let mut inventory_rows = inventory.into_iter().collect::<Vec<_>>();
        inventory_rows
            .sort_by(|left, right| right.1.len().cmp(&left.1.len()).then(left.0.cmp(&right.0)));

        let inventory_text = inventory_rows
            .into_iter()
            .take(10)
            .map(|(name, refs)| {
                let total_count = refs.len();
                let visible_refs = refs.into_iter().take(24).collect::<Vec<_>>();
                let suffix = if total_count > visible_refs.len() {
                    format!(", ... {} more", total_count - visible_refs.len())
                } else {
                    String::new()
                };
                format!(
                    "{} x{} [{}{}]",
                    name,
                    total_count,
                    visible_refs.join(", "),
                    suffix
                )
            })
            .collect::<Vec<_>>()
            .join("; ");

        lines.push(format!(
            "- construction {}: {} part(s); {}",
            construction_id,
            group_parts.len(),
            inventory_text
        ));
    }

    format!(
        "## Runtime Construction Groups\nUse this section to reason about parts that are attached into the same GearBlocks construction. Part `#id` values can repeat across different parent constructions, so disambiguate with construction id, index, part key, instance key, and coordinates before saying a part is missing.\n{}",
        lines.join("\n")
    )
}

fn build_guide_api_context_section() -> String {
    [
        "## Build Guide API Context",
        "Use these exported GearBlocks API surfaces when explaining build guides and current scene state:",
        "- `IPart`: part identity, category, mass, visibility/collision/selectability, world position, local position, orientation, and current unit size.",
        "- `IPartPaint` and `IPartProperties`: paint target colour, paintability, material name, strength, density, and material swap capability.",
        "- `IPartAttachments`, `IAttachment`, `ILinkNode`, and `ILink`: owned/associated attachments, attached parts, attachment type names, locked state, joint/interior flags, link-node type names, link availability, and connection positions.",
        "- `ITweakables`, `IResizable`, and `IControllablePartBehaviour`: configurable settings such as tweakable labels/values, resize step, current unit size, control bindings, activation state, direction/inversion options, and RPM/limit settings when the game exposes them.",
        "- `IEngineCrank`, `IEngineDrivenCrank`, `IEngineCylinder`, and `IEngineHead`: combustion-engine relationships, including crank, driven crank, crank shaft, linked cylinders, cylinder head, crank angle, timing angle, and current rotation speed.",
        "If a requested value is absent, say it was not exposed in the latest export rather than guessing.",
    ]
    .join("\n")
}

fn definition_details_section(
    parts: &[GearBlocksScenePart],
    details: &GearBlocksSceneDetailIndex,
) -> String {
    let mut sorted_parts = parts.iter().collect::<Vec<_>>();
    sorted_parts.sort_by(|left, right| {
        left.record
            .runtime_part_index
            .cmp(&right.record.runtime_part_index)
            .then(part_name(&left.record).cmp(&part_name(&right.record)))
    });

    let mut lines = Vec::new();
    for part in sorted_parts {
        let mut values = Vec::new();
        if let Some(summary) = metadata_summary(
            details.metadata_by_part.get(&part.record.part_key),
            &["property", "value"],
            8,
        ) {
            values.push(summary);
        }
        if let Some(summary) =
            attachment_summary(details.attachments_by_part.get(&part.record.part_key), 5)
        {
            values.push(summary);
        }
        if let Some(summary) =
            setting_summary(details.settings_by_part.get(&part.record.part_key), 8)
        {
            values.push(summary);
        }
        if let Some(summary) = output_summary(details.outputs_by_part.get(&part.record.part_key), 6)
        {
            values.push(summary);
        }
        if values.is_empty() {
            continue;
        }

        lines.push(format!(
            "- {} [{} / {}] {}: {}",
            part_reference(part),
            part.system,
            part.record.category,
            part_label(part),
            values.join("; ")
        ));
        if lines.len() >= 180 {
            break;
        }
    }

    if lines.is_empty() {
        "## DB Definition Details\nNo normalized metadata, attachment, setting, or output/control values were available for the current scene. Run a fresh GearBlocks scene export/import to populate the newer definition tables.".to_string()
    } else {
        format!(
            "## DB Definition Details\nThese rows are assembled from normalized definition/value tables instead of raw export JSON. Use them for paint/materials, attachment types, configurable settings, output/control channels, and other exposed metadata.\n{}",
            lines.join("\n")
        )
    }
}

fn structural_bounds_section(parts: &[GearBlocksScenePart]) -> String {
    let mut min_values = [f64::MAX; 3];
    let mut max_values = [f64::MIN; 3];
    let mut found = false;

    for part in parts.iter().filter(|part| part.is_structural) {
        let Some((x, y, z)) = local_position(&part.record) else {
            continue;
        };
        for (index, value) in [x, y, z].into_iter().enumerate() {
            min_values[index] = min_values[index].min(value);
            max_values[index] = max_values[index].max(value);
        }
        found = true;
    }

    if !found {
        return "## Structural Envelope\nNo structural bounds could be inferred.".to_string();
    }

    format!(
        "## Structural Envelope\nStructural member local-position bounds: x {:.2}..{:.2}, y {:.2}..{:.2}, z {:.2}..{:.2}. This is a coarse chassis envelope, not a visual mesh.",
        min_values[0], max_values[0], min_values[1], max_values[1], min_values[2], max_values[2]
    )
}

fn functional_parts_section(parts: &[GearBlocksScenePart]) -> String {
    let mut functional_parts = parts
        .iter()
        .filter(|part| part.is_functional)
        .collect::<Vec<_>>();
    functional_parts.sort_by(|left, right| {
        left.system.cmp(right.system).then(
            left.record
                .runtime_part_index
                .cmp(&right.record.runtime_part_index),
        )
    });

    let mut lines = Vec::new();
    for part in functional_parts.iter().take(140) {
        let behaviours = if part.behaviours.is_empty() {
            "none".to_string()
        } else {
            part.behaviours.join(", ")
        };
        lines.push(format!(
            "- {} [{} / {}] {}: {}; behaviours={}; links={}{}; part_key={}; instance={}",
            part_reference(part),
            part.system,
            part.record.category,
            part_label(part),
            part.purpose,
            behaviours,
            part.record.link_node_count,
            position_and_size_suffix(&part.record),
            part.record.part_key,
            part.record.part_instance_key
        ));
    }

    if functional_parts.len() > lines.len() {
        lines.push(format!(
            "- {} additional functional part(s) omitted from prompt context for size.",
            functional_parts.len() - lines.len()
        ));
    }

    if lines.is_empty() {
        "## Functional Parts\nNo functional parts identified.".to_string()
    } else {
        format!("## Functional Parts\n{}", lines.join("\n"))
    }
}

fn metadata_summary(
    rows: Option<&Vec<GameRuntimePartMetadataValueRecord>>,
    source_order: &[&str],
    limit: usize,
) -> Option<String> {
    let rows = rows?;
    let mut ordered = Vec::new();
    for source in source_order {
        ordered.extend(rows.iter().filter(|row| row.source_area == *source));
    }
    ordered.extend(
        rows.iter()
            .filter(|row| !source_order.iter().any(|source| row.source_area == *source)),
    );
    let values = ordered
        .into_iter()
        .take(limit)
        .map(|row| {
            format!(
                "{}:{}={} ({}) src={}/{} seen={}",
                row.source_area,
                row.field_path,
                compact_json_text(&row.value_json),
                row.value_type,
                row.source_export_id,
                row.source_construction_id,
                row.last_seen_at
            )
        })
        .collect::<Vec<_>>();
    (!values.is_empty()).then(|| format!("metadata [{}]", values.join(", ")))
}

fn attachment_summary(
    rows: Option<&Vec<GameRuntimePartAttachmentTypeRecord>>,
    limit: usize,
) -> Option<String> {
    let values = rows?
        .iter()
        .take(limit)
        .map(|row| {
            format!(
                "{} type={} valueType={} value={} src={}/{} seen={}",
                row.attachment_path,
                row.type_name,
                row.value_type,
                compact_json_text(&row.attachment_json),
                row.source_export_id,
                row.source_construction_id,
                row.last_seen_at
            )
        })
        .collect::<Vec<_>>();
    (!values.is_empty()).then(|| format!("attachments [{}]", values.join(", ")))
}

fn setting_summary(
    rows: Option<&Vec<GameRuntimePartSettingValueRecord>>,
    limit: usize,
) -> Option<String> {
    let values = rows?
        .iter()
        .take(limit)
        .map(|row| {
            let label = if row.label.trim().is_empty() {
                row.setting_key.as_str()
            } else {
                row.label.as_str()
            };
            format!(
                "{}:{}={} ({}) src={}/{} seen={}",
                row.setting_area,
                label,
                compact_json_text(&row.value_json),
                row.value_type,
                row.source_export_id,
                row.source_construction_id,
                row.last_seen_at
            )
        })
        .collect::<Vec<_>>();
    (!values.is_empty()).then(|| format!("settings [{}]", values.join(", ")))
}

fn output_summary(
    rows: Option<&Vec<GameRuntimePartOutputChannelValueRecord>>,
    limit: usize,
) -> Option<String> {
    let values = rows?
        .iter()
        .take(limit)
        .map(|row| {
            let label = if row.label.trim().is_empty() {
                row.channel_key.as_str()
            } else {
                row.label.as_str()
            };
            format!(
                "{}:{}={} ({}) src={}/{} seen={}",
                row.channel_area,
                label,
                compact_json_text(&row.value_json),
                row.value_type,
                row.source_export_id,
                row.source_construction_id,
                row.last_seen_at
            )
        })
        .collect::<Vec<_>>();
    (!values.is_empty()).then(|| format!("outputs/controls [{}]", values.join(", ")))
}

fn part_name(part: &GameRuntimePartInstanceRecord) -> String {
    for value in [
        &part.full_display_name,
        &part.display_name,
        &part.asset_name,
    ] {
        if !value.trim().is_empty() {
            return value.trim().to_string();
        }
    }
    "Unnamed part".to_string()
}

fn part_label(part: &GearBlocksScenePart) -> String {
    match part.friendly_name.as_deref() {
        Some(alias) if !alias.trim().is_empty() => {
            format!("{} (alias: {})", part_name(&part.record), alias)
        }
        _ => part_name(&part.record),
    }
}

fn part_reference(part: &GearBlocksScenePart) -> String {
    if part.record.source_construction_id.trim().is_empty() {
        format!(
            "#{} idx {}",
            part.record.runtime_part_id, part.record.runtime_part_index
        )
    } else {
        format!(
            "construction {} / #{} idx {}",
            part.record.source_construction_id,
            part.record.runtime_part_id,
            part.record.runtime_part_index
        )
    }
}

fn position_pair_text(part: &GameRuntimePartInstanceRecord) -> String {
    let world_position = world_position(part)
        .map(|(x, y, z)| format!("world=({x:.2},{y:.2},{z:.2})"))
        .unwrap_or_else(|| "world=unavailable".to_string());
    let local_position = local_position(part)
        .map(|(x, y, z)| format!("local=({x:.2},{y:.2},{z:.2})"))
        .unwrap_or_else(|| "local=unavailable".to_string());
    format!("{world_position} {local_position}")
}

fn position_and_size_suffix(part: &GameRuntimePartInstanceRecord) -> String {
    let position = local_position(part)
        .map(|(x, y, z)| format!(" local=({x:.2},{y:.2},{z:.2})"))
        .unwrap_or_default();
    let world_position = world_position(part)
        .map(|(x, y, z)| format!(" world=({x:.2},{y:.2},{z:.2})"))
        .unwrap_or_default();
    format!("{position}{world_position}{}", size_text(part))
}

fn size_text(part: &GameRuntimePartInstanceRecord) -> String {
    json_vector3_from_text(&part.current_unit_size_json)
        .map(|(x, y, z)| format!(" size=({x:.2},{y:.2},{z:.2})"))
        .unwrap_or_default()
}

fn world_position(part: &GameRuntimePartInstanceRecord) -> Option<(f64, f64, f64)> {
    match (part.world_x, part.world_y, part.world_z) {
        (Some(x), Some(y), Some(z)) => Some((x, y, z)),
        _ => json_vector3_from_text(&part.world_position_json),
    }
}

fn local_position(part: &GameRuntimePartInstanceRecord) -> Option<(f64, f64, f64)> {
    match (part.local_x, part.local_y, part.local_z) {
        (Some(x), Some(y), Some(z)) => Some((x, y, z)),
        _ => json_vector3_from_text(&part.local_position_json),
    }
}

fn json_vector3_from_text(value: &str) -> Option<(f64, f64, f64)> {
    let value = serde_json::from_str::<serde_json::Value>(value).ok()?;
    Some((
        value.get("x")?.as_f64()?,
        value.get("y")?.as_f64()?,
        value.get("z")?.as_f64()?,
    ))
}

fn behaviour_names(value: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(value).unwrap_or_default()
}

fn compact_json_text(value: &str) -> String {
    let parsed = serde_json::from_str::<serde_json::Value>(value).ok();
    let mut text = match parsed {
        Some(serde_json::Value::String(text)) => text.trim().to_string(),
        Some(value) => value.to_string(),
        None => value.trim().to_string(),
    };
    if text.len() > 120 {
        text = format!("{}...", text.chars().take(120).collect::<String>());
    }
    text
}

fn option_bool_label(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "unknown",
    }
}

fn classify_part_system(text: &str, category: &str, behaviours: &[String]) -> &'static str {
    let behaviour_text = behaviours.join(" ").to_ascii_lowercase();
    if text.contains("engine")
        || text.contains("crank")
        || text.contains("cylinder")
        || text.contains("combustion")
        || behaviour_text.contains("engine")
    {
        "engine and powertrain"
    } else if text.contains("gear")
        || text.contains("differential")
        || text.contains("axle")
        || text.contains("shaft")
        || text.contains("clutch")
        || text.contains("cv joint")
    {
        "drivetrain"
    } else if text.contains("wheel") || text.contains("tire") || text.contains("tyre") {
        "wheels and tires"
    } else if text.contains("steer") || text.contains("rack") {
        "steering"
    } else if text.contains("spring")
        || text.contains("damper")
        || text.contains("suspension")
        || text.contains("control arm")
    {
        "suspension"
    } else if text.contains("brake") {
        "brakes"
    } else if text.contains("button")
        || text.contains("switch")
        || text.contains("sensor")
        || text.contains("logic")
        || text.contains("controller")
    {
        "controls and data"
    } else if text.contains("beam")
        || text.contains("plate")
        || text.contains("bracket")
        || text.contains("frame")
        || text.contains("panel")
        || text.contains("bar")
        || text.contains("strut")
    {
        "structural frame"
    } else if category.contains("connector") || text.contains("connector") || text.contains("mount")
    {
        "mounts and connectors"
    } else if category.contains("body")
        || text.contains("body")
        || text.contains("fender")
        || text.contains("panel")
    {
        "bodywork"
    } else {
        "unknown or miscellaneous"
    }
}

fn part_purpose(system: &str, text: &str, behaviour_text: &str) -> &'static str {
    if behaviour_text.contains("spring") || text.contains("coil-over") || text.contains("damper") {
        "spring/damper element controlling suspension travel"
    } else if text.contains("control arm") || behaviour_text.contains("ball") {
        "articulated suspension locating member"
    } else if text.contains("differential") {
        "differential gear element distributing drive torque"
    } else if text.contains("clutch") {
        "engageable drivetrain coupling or clutch gear"
    } else if text.contains("brake") {
        "braking element"
    } else if text.contains("gear") {
        "gear train element transmitting or changing rotation"
    } else if text.contains("cv joint") {
        "constant-velocity joint or axle segment"
    } else if text.contains("axle") || text.contains("shaft") {
        "rotating shaft or axle segment"
    } else if text.contains("crank") || behaviour_text.contains("engine") {
        "engine crank or combustion powertrain element"
    } else if text.contains("wheel") || text.contains("tire") || text.contains("tyre") {
        "ground contact rolling element"
    } else if text.contains("steering") || text.contains("rack") {
        "steering input or linkage element"
    } else if system == "structural frame" {
        "rigid welded structural member"
    } else if system == "mounts and connectors" {
        "mounting connector or rigid attachment element"
    } else if system == "controls and data" {
        "control, signal, or data-bearing element"
    } else {
        "part role inferred from category and behaviours"
    }
}
