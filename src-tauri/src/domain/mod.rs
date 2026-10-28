//! Domain types shared by the database layer and the command layer.
//!
//! Everything here is serialised to the frontend with camelCase field names.
//! The TypeScript mirror lives in `src/types/domain.ts`; keep the two in step.

pub mod presentation;
pub mod studies;
pub mod teaching;
pub mod validate;

pub use presentation::*;
pub use studies::*;
pub use teaching::*;

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::AppError;

// ---------------------------------------------------------------------------
// Records
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
    pub revision: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub workspace_id: String,
    pub title: String,
    pub description: Option<String>,
    pub study_count: i64,
    pub created_at: String,
    pub updated_at: String,
    pub revision: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Study {
    pub id: String,
    pub project_id: String,
    pub template_id: Option<String>,
    pub title: String,
    pub study_type: StudyType,
    pub status: StudyStatus,
    /// The study's purpose, in the user's words.
    pub description: Option<String>,
    pub details: StudyDetails,
    pub created_at: String,
    pub updated_at: String,
    pub revision: i64,
}

/// A study together with its sections and their blocks, in display order.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StudyDetail {
    pub study: Study,
    pub sections: Vec<StudySection>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StudySection {
    pub id: String,
    pub study_id: String,
    pub key: Option<String>,
    pub title: String,
    pub guidance: Option<String>,
    /// Where the part belongs in the study's movement (v0.18.7); none for a
    /// part the person did not place.
    pub phase: Option<StudyPhase>,
    pub position: i64,
    /// How far the person has got with this part.
    pub status: SectionStatus,
    /// The structured fields this part was created with (a snapshot, so a
    /// later template change never alters the study); none for plain parts.
    pub fields: Option<SectionFields>,
    pub blocks: Vec<StudyBlock>,
    pub created_at: String,
    pub updated_at: String,
    pub revision: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StudyBlock {
    pub id: String,
    pub section_id: String,
    pub block_type: BlockType,
    pub position: i64,
    pub content: String,
    /// The field of the part's snapshot this block answers; none for free writing.
    pub field_key: Option<String>,
    /// An entry of a repeatable field: JSON object of its parts' values.
    pub data: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub revision: i64,
}

/// The kinds of content a study block can hold.
///
/// Stored as text, so adding a variant (quotation, word_study,
/// source_citation, ...) needs no schema change: add it here, in
/// `src/types/domain.ts`, and give it an editor in the frontend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockType {
    Paragraph,
    Note,
    Heading,
    Quotation,
    Question,
}

impl BlockType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Paragraph => "paragraph",
            Self::Note => "note",
            Self::Heading => "heading",
            Self::Quotation => "quotation",
            Self::Question => "question",
        }
    }
}

impl fmt::Display for BlockType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for BlockType {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "paragraph" => Ok(Self::Paragraph),
            "note" => Ok(Self::Note),
            "heading" => Ok(Self::Heading),
            "quotation" => Ok(Self::Quotation),
            "question" => Ok(Self::Question),
            other => Err(AppError::Unexpected(format!(
                "unknown block type in database: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StudyTemplate {
    pub id: String,
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub study_type: StudyType,
    pub is_builtin: bool,
    pub sections: Vec<TemplateSection>,
    pub created_at: String,
    pub updated_at: String,
    pub revision: i64,
}

/// The JSON shape of a template file in `resources/templates/`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TemplateDefinition {
    pub key: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub study_type: StudyType,
    pub sections: Vec<TemplateSection>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TemplateSection {
    /// Stable machine key for built-in sections; absent for sections the
    /// user adds.
    #[serde(default)]
    pub key: Option<String>,
    pub title: String,
    #[serde(default)]
    pub guidance: Option<String>,
    /// Not made with a new study: offered later, when the person adds a part.
    #[serde(default, skip_serializing_if = "is_false")]
    pub optional: bool,
    /// No longer offered at all. Kept so that studies made with it still show
    /// its title and guidance in the interface language.
    #[serde(default, skip_serializing_if = "is_false")]
    pub retired: bool,
    /// Where the part belongs in the study's movement (v0.18.7); the template is
    /// authoritative, and a part may have none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<StudyPhase>,
    /// Structured answer fields. A study copies them as a snapshot when it
    /// is created; customised copies of built-ins retain their fields.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<TemplateField>,
    /// Bumped when `fields` change in a way a snapshot should be able to tell.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub fields_version: u32,
    /// The built-in that supplies translated field labels and examples.
    /// Set when the user customises a structured built-in template.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fields_source: Option<String>,
}

/// How a field is answered: one text, or a list of entries made of parts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldKind {
    Writing,
    Text,
    Entries,
}

/// How one part of an entry is answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PartKind {
    /// A short single line.
    Line,
    /// Longer text.
    Text,
    /// One of the field's `options`.
    Choice,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryPart {
    pub key: String,
    pub kind: PartKind,
    /// For a choice: the stable keys of its options; the first is the default.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateField {
    pub key: String,
    pub kind: FieldKind,
    /// For entries: the parts of one entry, in order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parts: Vec<EntryPart>,
}

/// The snapshot a study part keeps of its template's fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionFields {
    /// Key of the built-in template the study was made from.
    pub template: String,
    pub version: u32,
    pub fields: Vec<TemplateField>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SectionStatus {
    #[default]
    NotStarted,
    InProgress,
    Complete,
}

impl SectionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotStarted => "not_started",
            Self::InProgress => "in_progress",
            Self::Complete => "complete",
        }
    }
}

impl FromStr for SectionStatus {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "not_started" => Ok(Self::NotStarted),
            "in_progress" => Ok(Self::InProgress),
            "complete" => Ok(Self::Complete),
            other => Err(AppError::Unexpected(format!(
                "unknown section status in database: {other}"
            ))),
        }
    }
}

impl TemplateSection {
    /// A part a new study starts with.
    pub fn starts(&self) -> bool {
        !self.optional && !self.retired
    }
}

fn is_false(value: &bool) -> bool {
    !value
}

fn is_zero(value: &u32) -> bool {
    *value == 0
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub theme: Theme,
    /// The translation the Scripture pane shows.
    pub scripture_translation_id: Option<String>,
    /// The active passage, as a canonical reference (e.g. `JHN.15.1-JHN.15.11`).
    pub scripture_passage: Option<String>,
}

// ---------------------------------------------------------------------------
// Command inputs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProject {
    pub workspace_id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// Fields left out are unchanged. An empty description clears it.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProject {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateStudy {
    pub project_id: String,
    pub title: String,
    /// Defaults to passage.
    #[serde(default)]
    pub study_type: Option<StudyType>,
    /// A template by id; otherwise by key; otherwise the built-in template
    /// for the study type.
    #[serde(default)]
    pub template_id: Option<String>,
    #[serde(default)]
    pub template_key: Option<String>,
    /// Canonical reference of the primary passage, if any.
    #[serde(default)]
    pub primary_passage: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStudy {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub study_type: Option<StudyType>,
    #[serde(default)]
    pub status: Option<StudyStatus>,
    /// An empty string clears it.
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub details: Option<StudyDetails>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStudySection {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    /// An empty string clears it.
    #[serde(default)]
    pub guidance: Option<String>,
    /// A phase ("text", "understanding"…); an empty string clears it.
    #[serde(default)]
    pub phase: Option<String>,
    #[serde(default)]
    pub status: Option<SectionStatus>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateStudyBlock {
    pub section_id: String,
    pub block_type: BlockType,
    #[serde(default)]
    pub content: Option<String>,
    /// The field of the part's snapshot the block answers.
    #[serde(default)]
    pub field_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStudyBlock {
    pub id: String,
    #[serde(default)]
    pub content: Option<String>,
    /// An entry's values as a JSON object (entries of a repeatable field only).
    #[serde(default)]
    pub data: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSettings {
    #[serde(default)]
    pub theme: Option<Theme>,
    /// An empty string clears the setting.
    #[serde(default)]
    pub scripture_translation_id: Option<String>,
    /// A canonical reference; an empty string clears the setting.
    #[serde(default)]
    pub scripture_passage: Option<String>,
}

/// A user template as edited: everything but identity and timestamps.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateInput {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub study_type: StudyType,
    pub sections: Vec<TemplateSection>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTemplate {
    pub id: String,
    #[serde(flatten)]
    pub template: TemplateInput,
}
