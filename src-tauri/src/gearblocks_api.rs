#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const CONSTRUCTION_NAMESPACE_DOCS_URL: &str =
    "https://www.gearblocksgame.com/apidoc/namespace_smash_hammer_1_1_gear_blocks_1_1_construction.html";

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksInterfaceDefinition {
    pub name: &'static str,
    pub docs_url: &'static str,
    pub members: &'static [&'static str],
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksVector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksQuaternion {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksBounds {
    pub center: Option<GearBlocksVector3>,
    pub size: Option<GearBlocksVector3>,
    pub min: Option<GearBlocksVector3>,
    pub max: Option<GearBlocksVector3>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksRuntimeRef {
    pub id: Option<i64>,
    pub index: Option<i64>,
    pub name: String,
    pub asset_guid: String,
    pub asset_name: String,
    pub type_name: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksOperationDescriptor {
    pub name: String,
    pub signature: String,
    pub parameter_names: Vec<String>,
    pub is_mutating: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksConstructionRuntimeSnapshot {
    pub construction: Option<GearBlocksConstructionSnapshot>,
    pub construction_operations: GearBlocksConstructionOperationsSupport,
    pub attachment_operations: GearBlocksAttachmentOperationsSupport,
    pub part_behaviour_operations: GearBlocksPartBehaviourOperationsSupport,
    pub populate_constructions: GearBlocksPopulateConstructionsSupport,
    pub parts: Vec<GearBlocksPartSnapshot>,
    pub attachments: Vec<GearBlocksAttachmentSnapshot>,
    pub links: Vec<GearBlocksLinkSnapshot>,
    pub link_nodes: Vec<GearBlocksLinkNodeSnapshot>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksAttachmentSnapshot {
    pub can_cycle_types: Option<bool>,
    pub connected_orientation: Option<GearBlocksQuaternion>,
    pub connected_part: Option<GearBlocksRuntimeRef>,
    pub connected_part_attachment_orientation: Option<GearBlocksQuaternion>,
    pub connected_part_attachment_position: Option<GearBlocksVector3>,
    pub connected_position: Option<GearBlocksVector3>,
    pub is_interior: Option<bool>,
    pub is_joint_attaching_composite_to_itself: Option<bool>,
    pub is_joint_attachment: Option<bool>,
    pub is_locked: Option<bool>,
    pub is_type_allowed: BTreeMap<String, bool>,
    pub owner_orientation: Option<GearBlocksQuaternion>,
    pub owner_part: Option<GearBlocksRuntimeRef>,
    pub owner_part_attachment_orientation: Option<GearBlocksQuaternion>,
    pub owner_part_attachment_position: Option<GearBlocksVector3>,
    pub owner_position: Option<GearBlocksVector3>,
    #[serde(rename = "type")]
    pub attachment_type: String,
    pub type_name: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksAttachmentOperationsSupport {
    pub create_attachment: Vec<GearBlocksOperationDescriptor>,
    pub delete_all_attachments: Vec<GearBlocksOperationDescriptor>,
    pub delete_attachment: Option<GearBlocksOperationDescriptor>,
    pub replace_attachment: Option<GearBlocksOperationDescriptor>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksCheckpointSnapshot {
    #[serde(flatten)]
    pub behaviour: GearBlocksPartBehaviourSnapshot,
    pub construction_handler_available: bool,
    pub has_inside_part: Option<bool>,
    pub has_inside_construction: Option<bool>,
    pub on_construction_entered_available: bool,
    pub on_construction_exited_available: bool,
    pub on_player_entered_available: bool,
    pub on_player_exited_available: bool,
    pub player_handler_available: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksConstructionSnapshot {
    pub id: Option<i64>,
    pub active_stage_idx: Option<i64>,
    pub builder_player_id: Option<i64>,
    pub character_player_id: Option<i64>,
    pub is_atomic: Option<bool>,
    pub is_buildable_by: BTreeMap<String, bool>,
    pub is_frozen: Option<bool>,
    pub is_invulnerable: Option<bool>,
    pub is_player_character: Option<bool>,
    pub is_selectable_by: BTreeMap<String, bool>,
    pub mass: Option<f64>,
    pub max_stage_idx: Option<i64>,
    pub num_composites: Option<i64>,
    pub num_parts: Option<i64>,
    pub part_catalogue: BTreeMap<String, i64>,
    pub parts: Vec<GearBlocksRuntimeRef>,
    pub preview_image: String,
    pub world_bounds: Option<GearBlocksBounds>,
    pub world_centre_of_mass: Option<GearBlocksVector3>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksConstructionOperationsSupport {
    pub assign_builder_player_id: Option<GearBlocksOperationDescriptor>,
    pub freeze_construction_and_move: Option<GearBlocksOperationDescriptor>,
    pub freeze_construction_at_ground: Option<GearBlocksOperationDescriptor>,
    pub freeze_construction_at_player: Option<GearBlocksOperationDescriptor>,
    pub set_construction_collidable: Option<GearBlocksOperationDescriptor>,
    pub set_construction_destroyable: Option<GearBlocksOperationDescriptor>,
    pub set_construction_frozen: Option<GearBlocksOperationDescriptor>,
    pub set_construction_selectable: Option<GearBlocksOperationDescriptor>,
    pub set_construction_targetable: Option<GearBlocksOperationDescriptor>,
    pub set_construction_visible: Option<GearBlocksOperationDescriptor>,
    pub set_part_collidable: Option<GearBlocksOperationDescriptor>,
    pub set_part_destroyable: Option<GearBlocksOperationDescriptor>,
    pub set_part_selectable: Option<GearBlocksOperationDescriptor>,
    pub set_parts_stage: Option<GearBlocksOperationDescriptor>,
    pub set_part_visible: Option<GearBlocksOperationDescriptor>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksPartBehaviourSnapshot {
    pub debug_info: String,
    pub idx: Option<i64>,
    pub is_activatable: Option<bool>,
    pub is_activated: Option<bool>,
    pub is_controllable: Option<bool>,
    pub is_tweakable: Option<bool>,
    pub name: String,
    pub part: Option<GearBlocksRuntimeRef>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksControllablePartBehaviourSnapshot {
    #[serde(flatten)]
    pub behaviour: GearBlocksPartBehaviourSnapshot,
    pub control_info: String,
    pub is_control_bound: Option<bool>,
    pub is_control_overridden: Option<bool>,
    pub only_control_when_player_locked: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksEnergyStoreSnapshot {
    #[serde(flatten)]
    pub behaviour: GearBlocksPartBehaviourSnapshot,
    pub capacity_remaining: Option<f64>,
    pub capacity_used: Option<f64>,
    pub charge_available: bool,
    pub discharge_available: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksEngineCrankSnapshot {
    #[serde(flatten)]
    pub behaviour: GearBlocksPartBehaviourSnapshot,
    pub driven_crank: Option<GearBlocksRuntimeRef>,
    pub linked_cylinders: Vec<GearBlocksRuntimeRef>,
    pub num_linked_cylinders: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksEngineCylinderSnapshot {
    #[serde(flatten)]
    pub behaviour: GearBlocksPartBehaviourSnapshot,
    pub crank: Option<GearBlocksRuntimeRef>,
    pub crank_angle: Option<f64>,
    pub head: Option<GearBlocksRuntimeRef>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksEngineDrivenCrankSnapshot {
    #[serde(flatten)]
    pub crank: GearBlocksEngineCrankSnapshot,
    pub crank_shaft: Option<GearBlocksRuntimeRef>,
    pub current_rotation_speed: Option<f64>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksEngineHeadSnapshot {
    #[serde(flatten)]
    pub behaviour: GearBlocksPartBehaviourSnapshot,
    pub crank: Option<GearBlocksRuntimeRef>,
    pub crank_angle: Option<f64>,
    pub cylinder: Option<GearBlocksRuntimeRef>,
    pub timing_angle: Option<f64>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksLinkSnapshot {
    pub link_from_node: Option<GearBlocksRuntimeRef>,
    pub link_to_node: Option<GearBlocksRuntimeRef>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksLinkNodeSnapshot {
    pub associated_links: Vec<GearBlocksRuntimeRef>,
    pub can_be_linked_to: BTreeMap<String, bool>,
    pub has_links: Option<bool>,
    pub idx: Option<i64>,
    pub is_linked_to: BTreeMap<String, bool>,
    pub is_same_type: BTreeMap<String, bool>,
    pub is_type_hidden: Option<bool>,
    pub link_from_available: Option<bool>,
    pub link_to_available: Option<bool>,
    pub local_position: Option<GearBlocksVector3>,
    pub owned_links: Vec<GearBlocksRuntimeRef>,
    pub part: Option<GearBlocksRuntimeRef>,
    pub position: Option<GearBlocksVector3>,
    pub type_name: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksPartSnapshot {
    pub asset_guid: String,
    pub asset_name: String,
    pub attachments: Option<GearBlocksPartAttachmentsSnapshot>,
    pub behaviours: Vec<GearBlocksBehaviourSnapshot>,
    pub bounds: Option<GearBlocksBounds>,
    pub category: String,
    pub display_name: String,
    pub full_display_name: String,
    pub id: Option<i64>,
    pub idx: Option<i64>,
    pub is_collidable: Option<bool>,
    pub is_deprecated: Option<bool>,
    pub is_destroyable: Option<bool>,
    pub is_enabled: Option<bool>,
    pub is_material_swappable: Option<bool>,
    pub is_paintable: Option<bool>,
    pub is_resizable: Option<bool>,
    pub is_selectable: Option<bool>,
    pub is_visible: Option<bool>,
    pub link_nodes: Vec<GearBlocksLinkNodeSnapshot>,
    pub mass: Option<f64>,
    pub paint: Option<GearBlocksPartPaintSnapshot>,
    pub parent_construction: Option<GearBlocksRuntimeRef>,
    pub properties: Option<GearBlocksPartPropertiesSnapshot>,
    pub resizable: Option<GearBlocksResizableSnapshot>,
    pub stage_idx: Option<i64>,
    pub strength: Option<f64>,
    pub tweakables: Option<GearBlocksTweakablesSnapshot>,
    pub unit_volume: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "interface", content = "snapshot")]
pub enum GearBlocksBehaviourSnapshot {
    PartBehaviour(GearBlocksPartBehaviourSnapshot),
    ControllablePartBehaviour(GearBlocksControllablePartBehaviourSnapshot),
    Checkpoint(GearBlocksCheckpointSnapshot),
    EnergyStore(GearBlocksEnergyStoreSnapshot),
    EngineCrank(GearBlocksEngineCrankSnapshot),
    EngineCylinder(GearBlocksEngineCylinderSnapshot),
    EngineDrivenCrank(GearBlocksEngineDrivenCrankSnapshot),
    EngineHead(GearBlocksEngineHeadSnapshot),
    Unknown(serde_json::Value),
}

impl Default for GearBlocksBehaviourSnapshot {
    fn default() -> Self {
        Self::Unknown(serde_json::Value::Null)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksPartAttachmentsSnapshot {
    pub associated: Vec<GearBlocksAttachmentSnapshot>,
    pub attached_parts: Vec<GearBlocksRuntimeRef>,
    pub attachment_by_part: BTreeMap<String, GearBlocksAttachmentSnapshot>,
    pub is_attached: BTreeMap<String, bool>,
    pub owned: Vec<GearBlocksAttachmentSnapshot>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksPartBehaviourOperationsSupport {
    pub toggle_activated: Option<GearBlocksOperationDescriptor>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksPartPaintSnapshot {
    pub target_colour: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksPartPropertiesSnapshot {
    pub density: Option<f64>,
    pub is_paintable: Option<bool>,
    pub is_swappable: Option<bool>,
    pub mass: Option<f64>,
    pub material_name: String,
    pub strength: Option<f64>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksPopulateConstructionsSupport {
    pub destroy_construction: Option<GearBlocksOperationDescriptor>,
    pub destroy_constructions: Option<GearBlocksOperationDescriptor>,
    pub destroy_part: Option<GearBlocksOperationDescriptor>,
    pub duplicate_construction: Vec<GearBlocksOperationDescriptor>,
    pub duplicate_part: Vec<GearBlocksOperationDescriptor>,
    pub get_player_construction_id: Option<GearBlocksOperationDescriptor>,
    pub spawn_construction: Vec<GearBlocksOperationDescriptor>,
    pub spawn_part: Vec<GearBlocksOperationDescriptor>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksResizableSnapshot {
    pub current_unit_size: Option<GearBlocksVector3>,
    pub resize_unit_step: Option<GearBlocksVector3>,
    pub set_size_available: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksTweakablesSnapshot {
    pub num_tweakables: Option<i64>,
    pub tweakables: Vec<serde_json::Value>,
    pub tweakable_by_label: BTreeMap<String, serde_json::Value>,
    pub sync_tweakables_available: bool,
}

pub const CONSTRUCTION_INTERFACE_DEFINITIONS: &[GearBlocksInterfaceDefinition] = &[
    interface_definition("IAttachment", "interface_smash_hammer_1_1_gear_blocks_1_1_construction_1_1_i_attachment.html", &["CanCycleTypes", "ConnectedOrientation", "ConnectedPart", "ConnectedPartAttachmentOrientation", "ConnectedPartAttachmentPosition", "ConnectedPosition", "IsInterior", "IsJointAttachingCompositeToItself", "IsJointAttachment", "IsLocked", "IsTypeAllowed(AttachmentTypeFlags type)", "OwnerOrientation", "OwnerPart", "OwnerPartAttachmentOrientation", "OwnerPartAttachmentPosition", "OwnerPosition", "Type", "TypeName"]),
    interface_definition("IAttachmentOperations", "interface_smash_hammer_1_1_gear_blocks_1_1_construction_1_1_i_attachment_operations.html", &["CreateAttachment(AttachmentTypeFlags type, IPart ownerPart, IPart connectedPart, Vector3Proxy searchPosition, Vector3Proxy searchNormal, bool snapPosition=false, bool invokeAsPlayer=false)", "CreateAttachment(AttachmentTypeFlags type, IPart ownerPart, IPart connectedPart, Vector3Proxy ownerSearchPosition, Vector3Proxy ownerSearchNormal, Vector3Proxy connectedSearchPosition, Vector3Proxy connectedSearchNormal, bool snapPosition=false, bool invokeAsPlayer=false)", "DeleteAllAttachments(IPart part, bool invokeAsPlayer=false)", "DeleteAllAttachments(IConstruction construction, bool invokeAsPlayer=false)", "DeleteAttachment(IAttachment attachment, bool invokeAsPlayer=false)", "ReplaceAttachment(IAttachment attachment, AttachmentTypeFlags newType, bool invokeAsPlayer=false)"]),
    interface_definition("ICheckpoint", "interface_smash_hammer_1_1_gear_blocks_1_1_construction_1_1_i_checkpoint.html", &["ConstructionHandler(IConstruction construction)", "DebugInfo", "HasInside(IPart part)", "HasInside(IConstruction construction)", "Idx", "IsActivatable", "IsActivated", "IsControllable", "IsTweakable", "Name", "OnConstructionEntered", "OnConstructionExited", "OnPlayerEntered", "OnPlayerExited", "Part", "PlayerHandler(IPlayer player)"]),
    interface_definition("IConstruction", "interface_smash_hammer_1_1_gear_blocks_1_1_construction_1_1_i_construction.html", &["ActiveStageIdx", "AddPartsToCatalogue(IDictionary< AssetGUID, ushort > partCatalogue)", "BuilderPlayerID", "CalcWorldBounds(bool activeStageOnly=false)", "CalcWorldCentreOfMass()", "CharacterPlayerID", "GetPart(ushort idx)", "ID", "IsAtomic", "IsBuildableBy(byte playerID)", "IsFrozen", "IsInvulnerable", "IsPlayerCharacter", "IsSelectableBy(byte playerID)", "Mass", "MaxStageIdx", "NumComposites", "NumParts", "Parts", "PreviewImage", "SetActiveStage(ushort stageIdx)"]),
    interface_definition("IConstructionOperations", "interface_smash_hammer_1_1_gear_blocks_1_1_construction_1_1_i_construction_operations.html", &["AssignBuilderPlayerID(ushort constructionID, byte builderPlayerID)", "FreezeConstructionAndMove(ushort constructionID, Vector3Proxy translation, QuaternionProxy rotation, bool invokeAsPlayer=false)", "FreezeConstructionAtGround(ushort constructionID, Vector3Proxy position, bool invokeAsPlayer=false)", "FreezeConstructionAtPlayer(ushort constructionID, bool invokeAsPlayer=false, bool selectAfterFreeze=false)", "SetConstructionCollidable(ushort constructionID, bool isCollidable)", "SetConstructionDestroyable(ushort constructionID, bool isDestroyable)", "SetConstructionFrozen(ushort constructionID, bool freeze, bool invokeAsPlayer=false)", "SetConstructionSelectable(ushort constructionID, bool isSelectable)", "SetConstructionTargetable(ushort constructionID, bool isTargetable)", "SetConstructionVisible(ushort constructionID, bool isVisible)", "SetPartCollidable(ushort partID, bool isCollidable)", "SetPartDestroyable(ushort partID, bool isDestroyable)", "SetPartSelectable(ushort partID, bool isSelectable)", "SetPartsStage(ushort[] partIDs, SetStageMode setStageMode, ushort stageIdx=0)", "SetPartVisible(ushort partID, bool isVisible)"]),
    interface_definition("IControllablePartBehaviour", "interface_smash_hammer_1_1_gear_blocks_1_1_construction_1_1_i_controllable_part_behaviour.html", &["ControlInfo", "DebugInfo", "Idx", "IsActivatable", "IsActivated", "IsControlBound", "IsControllable", "IsControlOverridden", "IsTweakable", "Name", "OnlyControlWhenPlayerLocked", "Part"]),
    interface_definition("IEnergyStore", "interface_smash_hammer_1_1_gear_blocks_1_1_construction_1_1_i_energy_store.html", &["CapacityRemaining", "CapacityUsed", "Charge(float proportionOfCapacityUsed)", "DebugInfo", "Discharge(float proportionOfCapacityRemaining)", "Idx", "IsActivatable", "IsActivated", "IsControllable", "IsTweakable", "Name", "Part"]),
    interface_definition("IEngineCrank", "interface_smash_hammer_1_1_gear_blocks_1_1_construction_1_1_i_engine_crank.html", &["DebugInfo", "DrivenCrank", "Idx", "IsActivatable", "IsActivated", "IsControllable", "IsTweakable", "LinkedCylinders", "Name", "NumLinkedCylinders", "Part"]),
    interface_definition("IEngineCylinder", "interface_smash_hammer_1_1_gear_blocks_1_1_construction_1_1_i_engine_cylinder.html", &["Crank", "DebugInfo", "GetCrankAngle()", "Head", "Idx", "IsActivatable", "IsActivated", "IsControllable", "IsTweakable", "Name", "Part"]),
    interface_definition("IEngineDrivenCrank", "interface_smash_hammer_1_1_gear_blocks_1_1_construction_1_1_i_engine_driven_crank.html", &["CrankShaft", "CurrentRotationSpeed", "DebugInfo", "DrivenCrank", "Idx", "IsActivatable", "IsActivated", "IsControllable", "IsTweakable", "LinkedCylinders", "Name", "NumLinkedCylinders", "Part"]),
    interface_definition("IEngineHead", "interface_smash_hammer_1_1_gear_blocks_1_1_construction_1_1_i_engine_head.html", &["Crank", "Cylinder", "DebugInfo", "GetCrankAngle()", "Idx", "IsActivatable", "IsActivated", "IsControllable", "IsTweakable", "Name", "Part", "TimingAngle"]),
    interface_definition("ILink", "interface_smash_hammer_1_1_gear_blocks_1_1_construction_1_1_i_link.html", &["LinkFromNode", "LinkToNode"]),
    interface_definition("ILinkNode", "interface_smash_hammer_1_1_gear_blocks_1_1_construction_1_1_i_link_node.html", &["AssociatedLinks", "CanBeLinkedTo(ILinkNode otherLinkNode)", "HasLinks", "Idx", "IsLinkedTo(ILinkNode otherLinkNode)", "IsSameType(ILinkNode otherLinkNode)", "IsTypeHidden", "LinkFromAvailable", "LinkToAvailable", "LocalPosition", "OwnedLinks", "Part", "Position", "TypeName"]),
    interface_definition("IPart", "interface_smash_hammer_1_1_gear_blocks_1_1_construction_1_1_i_part.html", &["AssetGUID", "AssetName", "Attachments", "Behaviours", "Bounds", "Category", "DisplayName", "FullDisplayName", "GetBehaviour(string name)", "ID", "Idx", "IsCollidable", "IsDeprecated", "IsDestroyable", "IsEnabled", "IsMaterialSwappable", "IsPaintable", "IsResizable", "IsSelectable", "IsVisible", "LinkNodes", "Mass", "Paint", "ParentConstruction", "Properties", "StageIdx", "Strength", "UnitVolume"]),
    interface_definition("IPartAttachments", "interface_smash_hammer_1_1_gear_blocks_1_1_construction_1_1_i_part_attachments.html", &["Associated", "GetAttachedParts()", "GetAttachment(IPart otherPart)", "IsAttached(IPart otherPart)", "Owned"]),
    interface_definition("IPartBehaviour", "interface_smash_hammer_1_1_gear_blocks_1_1_construction_1_1_i_part_behaviour.html", &["DebugInfo", "Idx", "IsActivatable", "IsActivated", "IsControllable", "IsTweakable", "Name", "Part"]),
    interface_definition("IPartBehaviourOperations", "interface_smash_hammer_1_1_gear_blocks_1_1_construction_1_1_i_part_behaviour_operations.html", &["ToggleActivated(IPart part)"]),
    interface_definition("IPartPaint", "interface_smash_hammer_1_1_gear_blocks_1_1_construction_1_1_i_part_paint.html", &["TargetColour"]),
    interface_definition("IPartProperties", "interface_smash_hammer_1_1_gear_blocks_1_1_construction_1_1_i_part_properties.html", &["Density", "IsPaintable", "IsSwappable", "Mass", "MaterialName", "Strength"]),
    interface_definition("IPopulateConstructions", "interface_smash_hammer_1_1_gear_blocks_1_1_construction_1_1_i_populate_constructions.html", &["DestroyConstruction(ushort constructionID, bool invokeAsPlayer=false)", "DestroyConstructions(ushort[] constructionIDs, bool invokeAsPlayer=false)", "DestroyPart(ushort partID, bool invokeAsPlayer=false)", "DuplicateConstruction(ushort constructionID, bool invokeAsPlayer=false)", "DuplicateConstruction(ushort constructionID, Vector3Proxy groundOrigin, bool invokeAsPlayer=false)", "DuplicatePart(ushort partID, bool invokeAsPlayer=false)", "DuplicatePart(ushort partID, Vector3Proxy spawnPosition, QuaternionProxy spawnOrientation, bool invokeAsPlayer=false)", "GetPlayerConstructionID(byte playerID)", "SpawnConstruction(string savedFolder, byte saveTypeID, bool invokeAsPlayer=false)", "SpawnConstruction(string savedFolder, byte saveTypeID, Vector3Proxy groundOrigin, bool invokeAsPlayer=false)", "SpawnPart(AssetGUID partAssetGUID, bool invokeAsPlayer=false)", "SpawnPart(AssetGUID partAssetGUID, Vector3Proxy spawnPosition, QuaternionProxy spawnOrientation, bool invokeAsPlayer=false)"]),
    interface_definition("IResizable", "interface_smash_hammer_1_1_gear_blocks_1_1_construction_1_1_i_resizable.html", &["CurrentUnitSize", "ResizeUnitStep", "SetSize(Vector3Proxy unitSize, bool clamp=true)"]),
    interface_definition("ITweakables", "interface_smash_hammer_1_1_gear_blocks_1_1_construction_1_1_i_tweakables.html", &["GetTweakable(string label)", "NumTweakables", "SyncTweakables()", "Tweakables"]),
];

const fn interface_definition(
    name: &'static str,
    page: &'static str,
    members: &'static [&'static str],
) -> GearBlocksInterfaceDefinition {
    GearBlocksInterfaceDefinition {
        name,
        docs_url: page,
        members,
    }
}
