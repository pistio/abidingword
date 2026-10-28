//! Teaching Preparation (v0.16): what a study leads to — a sermon, a group
//! study, a lesson — written by the person, reusing the study's findings and
//! naming Scripture. The TypeScript mirror is `src/types/teaching.ts`.

use serde::{Deserialize, Serialize};

use crate::domain::Finding;
use crate::scripture::reference::ScriptureRefDto;

/// The kinds of teaching a preparation can be for. A hint for wording and
/// the assistant, never a fixed structure.
pub const TEACHING_KINDS: &[&str] = &[
    "sermon",
    "bible_study",
    "youth",
    "children",
    "lesson",
    "talk",
    "other",
];

/// The kinds of step a lesson can be made of. Audio is accepted for the
/// future (a step with a recording), not yet offered in the interface.
pub const STEP_KINDS: &[&str] = &[
    "text",
    "scripture",
    "question",
    "activity",
    "image",
    "audio",
];

/// Blocks a template or a teaching may be made of. A kind the application
/// does not know yet is still stored; the editor shows it as writing.
pub const TEACHING_BLOCK_TYPES: &[&str] = &[
    "section",
    "writing",
    "scripture",
    "teaching_notes",
    "discussion_questions",
    "explanation",
    "illustration",
    "activity",
    "application",
    "objectives",
    "time",
    "presentation_notes",
    "handout",
    "outline_point",
];

/// A block whose body is also one of the columns presentations already read.
pub const BLOCK_ROLES: &[&str] = &[
    "introduction",
    "conclusion",
    "central_message",
    "notes",
    "explanation",
    "application",
    "illustration",
];

/// Leader-only unless the person explicitly marks the block for participants.
pub fn block_is_private_by_default(block_type: &str) -> bool {
    !matches!(
        block_type,
        "discussion_questions" | "handout" | "section" | "scripture" | "outline_point"
    )
}

/// What the person kept from the assistant.
pub const MATERIAL_KINDS: &[&str] = &["story", "illustration", "image", "visual"];

/// How far a kept item can be relied on. Always shown with it.
/// * `verified`: a real event or quotation supported by at least two
///   independent sources the search actually returned;
/// * `unverified`: offered as real, but not supported well enough;
/// * `hypothetical`: an invented illustration, never presented as fact;
/// * `found`: an image found with its source and licence;
/// * `generated`: an image made by an image generator, never a photograph;
/// * `suggestion`: an idea (e.g. a visual) from the assistant's own reasoning.
pub const MATERIAL_STANDINGS: &[&str] = &[
    "verified",
    "unverified",
    "hypothetical",
    "found",
    "generated",
    "suggestion",
];

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TeachingSummary {
    pub id: String,
    pub study_id: String,
    pub title: String,
    pub kind: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TeachingPreparation {
    pub id: String,
    pub study_id: String,
    pub title: String,
    pub kind: Option<String>,
    pub audience: Option<String>,
    pub purpose: Option<String>,
    pub central_message: Option<String>,
    pub introduction: String,
    pub conclusion: String,
    pub notes: String,
    pub points: Vec<TeachingPoint>,
    /// Study findings used here (live: always the finding as it is now).
    pub findings: Vec<TeachingFindingLink>,
    pub passages: Vec<TeachingPassage>,
    pub materials: Vec<TeachingMaterial>,
    /// The template this teaching was started from, when it uses blocks.
    /// Absent for a teaching written in the earlier editor.
    pub template_key: Option<String>,
    /// 2: block editor. Absent: the editor this teaching was written in.
    pub template_version: Option<i64>,
    pub blocks: Vec<TeachingBlock>,
    pub created_at: String,
    pub updated_at: String,
    pub revision: i64,
}

/// One block of a teaching. `participant_facing` is false for private leader
/// notes: handouts, worksheets and a proposed presentation leave them out
/// unless the person has marked the block for participants.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TeachingBlock {
    pub id: String,
    pub parent_id: Option<String>,
    pub position: i64,
    pub block_type: String,
    pub block_key: Option<String>,
    pub title: String,
    pub body: String,
    pub participant_facing: bool,
    pub role: Option<String>,
    pub guidance_key: Option<String>,
    /// The outline point this block keeps in step with, when it is one.
    pub point_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TeachingPoint {
    pub id: String,
    pub position: i64,
    /// For a step of a lesson: what kind of step it is (see [`STEP_KINDS`]).
    pub kind: Option<String>,
    pub title: String,
    pub explanation: String,
    pub application: String,
    pub illustration: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TeachingFindingLink {
    pub id: String,
    /// The point it is used under; `None` for the preparation as a whole.
    pub point_id: Option<String>,
    pub finding: Finding,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TeachingPassage {
    pub id: String,
    pub point_id: Option<String>,
    /// The block this passage belongs to; absent for the whole teaching or a point.
    pub block_id: Option<String>,
    pub reference: ScriptureRefDto,
}

/// A source the assistant was given by a search (never one it wrote).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct ExternalSource {
    pub title: String,
    pub url: String,
    pub site: String,
    pub excerpt: String,
}

/// An image's provenance: where it was found and under which licence, or
/// that it was generated (then `file` is where it was saved).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct ImageInfo {
    pub url: Option<String>,
    pub page_url: Option<String>,
    pub thumb_url: Option<String>,
    pub licence: Option<String>,
    pub licence_url: Option<String>,
    pub author: Option<String>,
    pub attribution: Option<String>,
    pub file: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TeachingMaterial {
    pub id: String,
    pub point_id: Option<String>,
    pub kind: String,
    pub standing: String,
    pub title: String,
    pub content: String,
    pub sources: Vec<ExternalSource>,
    pub image: Option<ImageInfo>,
    /// Which assistant produced it, e.g. "OpenAI · gpt-6.1-sol".
    pub assistant: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTeaching {
    pub study_id: String,
    pub title: String,
    #[serde(default)]
    pub kind: Option<String>,
    /// The steps or points it starts with (a format's starting structure),
    /// named in the person's language by the interface.
    #[serde(default)]
    pub points: Vec<StartingPoint>,
    /// Set when the teaching is made from a block template. Absent: the
    /// earlier editor, unchanged.
    #[serde(default)]
    pub template_key: Option<String>,
    #[serde(default)]
    pub template_version: Option<i64>,
    #[serde(default)]
    pub blocks: Vec<NewTeachingBlock>,
}

/// A block copied from a template into a new teaching. Structure, not a link
/// back to the template.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NewTeachingBlock {
    pub key: Option<String>,
    pub parent_key: Option<String>,
    pub block_type: String,
    pub title: String,
    pub body: String,
    pub participant_facing: Option<bool>,
    pub role: Option<String>,
    pub guidance_key: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CreateTeachingBlock {
    pub preparation_id: String,
    pub parent_id: Option<String>,
    /// Insert after this sibling; at the end of the group when absent.
    pub after_block_id: Option<String>,
    pub block_type: String,
    pub title: String,
    pub body: String,
    pub participant_facing: Option<bool>,
    pub role: Option<String>,
    pub guidance_key: Option<String>,
}

/// Each field: `None` leaves it as it is.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct UpdateTeachingBlock {
    pub id: String,
    pub title: Option<String>,
    pub body: Option<String>,
    pub block_type: Option<String>,
    pub participant_facing: Option<bool>,
    pub role: Option<String>,
    pub guidance_key: Option<String>,
    /// `Some("")` moves the block to the top level.
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ReorderTeachingBlocks {
    pub preparation_id: String,
    /// The section these blocks sit in; empty for the top level.
    pub parent_id: Option<String>,
    pub ids: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct StartingPoint {
    pub title: String,
    pub kind: Option<String>,
}

/// Each field: `None` leaves it as it is; `Some("")` clears an optional one.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct UpdateTeaching {
    pub id: String,
    pub title: Option<String>,
    pub kind: Option<String>,
    pub audience: Option<String>,
    pub purpose: Option<String>,
    pub central_message: Option<String>,
    pub introduction: Option<String>,
    pub conclusion: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CreateTeachingPoint {
    pub preparation_id: String,
    pub title: Option<String>,
    pub kind: Option<String>,
    /// Put the new point straight after this one; at the end when absent.
    pub after_point_id: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct UpdateTeachingPoint {
    pub id: String,
    /// `Some("")` clears it.
    pub kind: Option<String>,
    pub title: Option<String>,
    pub explanation: Option<String>,
    pub application: Option<String>,
    pub illustration: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LinkTeachingFinding {
    pub preparation_id: String,
    pub point_id: Option<String>,
    pub finding_id: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AddTeachingPassage {
    pub preparation_id: String,
    pub point_id: Option<String>,
    pub block_id: Option<String>,
    /// Canonical reference, e.g. "JHN.15.1-JHN.15.8".
    pub reference: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct KeepTeachingMaterial {
    pub preparation_id: String,
    pub point_id: Option<String>,
    pub kind: String,
    pub standing: String,
    pub title: String,
    pub content: String,
    pub sources: Vec<ExternalSource>,
    pub image: Option<ImageInfo>,
    pub assistant: Option<String>,
}

/// A Bible Teaching template of the person's own (v0.18.7): a format and the
/// points, questions or steps a teaching starts with. Structure only.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TeachingTemplate {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    /// One of [`TEACHING_KINDS`].
    pub kind: String,
    pub points: Vec<TeachingTemplatePoint>,
    /// 2 when the template is a block template. Absent for a points template.
    #[serde(default, skip_serializing_if = "is_legacy_template_version")]
    pub version: i64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocks: Vec<TeachingTemplateBlock>,
    pub created_at: String,
    pub updated_at: String,
    pub revision: i64,
}

/// One point, question or step a teaching starts with.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct TeachingTemplatePoint {
    /// The built-in starting step it was taken from, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// May be empty: a point to be named in the teaching.
    pub title: String,
    /// A step's kind (see [`STEP_KINDS`]), if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

fn is_legacy_template_version(version: &i64) -> bool {
    *version < 2
}

/// One block a new teaching starts with. Titles are the person's words.
/// Nothing they would write in a session is kept here.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct TeachingTemplateBlock {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_key: Option<String>,
    pub block_type: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participant_facing: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guidance_key: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TeachingTemplateInput {
    pub name: String,
    pub description: Option<String>,
    pub kind: String,
    #[serde(default)]
    pub points: Vec<TeachingTemplatePoint>,
    #[serde(default)]
    pub version: i64,
    #[serde(default)]
    pub blocks: Vec<TeachingTemplateBlock>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTeachingTemplate {
    pub id: String,
    #[serde(flatten)]
    pub template: TeachingTemplateInput,
}
