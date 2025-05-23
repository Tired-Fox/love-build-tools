//! Rust representation of the lua language server configuration file.
//!
//! Check the official documentation for possible settings and values.
//! - https://luals.github.io/wiki/settings/#settings
//!
//! This library handles parsing the defined fields in the configuration
//! file along with making adding your own fields a breeze.

use std::{
    borrow::Cow, collections::{BTreeMap, HashSet}, marker::PhantomData, ops::{Deref, DerefMut}, path::Path, str::FromStr
};

use serde::{
    Deserialize, Serialize,
    de::{DeserializeOwned, Visitor},
};

mod error;
pub use error::Error;

pub mod diagnostics;
use diagnostics::{Diagnostic, DiagnosticGroup};

#[inline(always)]
const fn enabled(ctx: &bool) -> bool { *ctx }
#[inline(always)]
const fn disabled(ctx: &bool) -> bool { !*ctx }
#[inline(always)]
const fn zero(ctx: &usize) -> bool { *ctx == 0 }
#[inline(always)]
const fn default_true() -> bool { true }

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct AddonManager<O = ()> {
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub enable: bool,

    #[serde(flatten)]
    pub custom: O,
}

impl<O: Default> Default for AddonManager<O> {
    fn default() -> Self {
        Self {
            enable: true,
            custom: Default::default(),
        }
    }
}

impl<O> Deref for AddonManager<O> {
    type Target = O;
    fn deref(&self) -> &Self::Target {
        &self.custom
    }
}

impl<O> DerefMut for AddonManager<O> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.custom
    }
}

#[derive(Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Snippet {
    #[default]
    Disable,
    Replace,
    Both,
}

#[derive(Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Show {
    Disable,
    Enable,
    #[default]
    Fallback,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Completion<O = ()> {
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub enable: bool,
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub auto_require: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_snippet: Option<Snippet>,
    #[serde(default, skip_serializing_if = "zero")]
    pub display_context: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyword_snippet: Option<Snippet>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postfix: Option<Cow<'static, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_separator: Option<Cow<'static, str>>,
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub show_params: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_word: Option<Show>,
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub workspace_word: bool,

    #[serde(flatten)]
    pub custom: O,
}

impl<O: Default> Default for Completion<O> {
    fn default() -> Self {
        Self {
            enable: true,
            auto_require: true,
            call_snippet: None,
            display_context: 0,
            keyword_snippet: None,
            postfix: None,
            require_separator: None,
            show_params: true,
            show_word: None,
            workspace_word: true,

            custom: Default::default(),
        }
    }
}

impl<O> Deref for Completion<O> {
    type Target = O;
    fn deref(&self) -> &Self::Target {
        &self.custom
    }
}

impl<O> DerefMut for Completion<O> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.custom
    }
}

#[derive(Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum FileState {
    Any,
    Opened,
    None,
    #[default]
    Fallback,
}

#[derive(Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum GroupSeverity {
    Error,
    Warning,
    Information,
    Hint,
    #[default]
    Fallback,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Information,
    Hint,
    #[serde(rename = "Error!")]
    ErrorBang,
    #[serde(rename = "Warning!")]
    WarningBang,
    #[serde(rename = "Information!")]
    InformationBang,
    #[serde(rename = "Hint!")]
    HintBang,
}

impl FromStr for Severity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "error" => Self::Error,
            "warning" => Self::Warning,
            "information" => Self::Information,
            "hint" => Self::Hint,
            "error!" => Self::ErrorBang,
            "warning!" => Self::WarningBang,
            "information!" => Self::InformationBang,
            "hint!" => Self::HintBang,
            other => return Err(format!("invalid diagnostic severity: {other}")),
        })
    }
}

#[derive(Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Files {
    Enable,
    #[default]
    Opened,
    Disable,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum FileStatus {
    Any,
    Opened,
    None,
    #[serde(rename = "Any!")]
    AnyBang,
    #[serde(rename = "Opened!")]
    OpenedBang,
    #[serde(rename = "None!")]
    NoneBang,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Event {
    OnChange,
    OnSave,
    None,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostics<O = ()> {
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub enable: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub disable: Vec<Diagnostic>,
    //pub disable: Diagnostic
    #[serde(default = "Default::default", skip_serializing_if = "Vec::is_empty")]
    pub disable_scheme: Vec<Cow<'static, str>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub globals: Vec<Cow<'static, str>>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub group_file_status: BTreeMap<DiagnosticGroup, FileStatus>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub group_severity: BTreeMap<DiagnosticGroup, GroupSeverity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignored_files: Option<Files>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub library_files: Option<Files>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub needed_file_status: BTreeMap<Diagnostic, FileStatus>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub severity: BTreeMap<Diagnostic, Severity>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unused_local_exclude: Vec<Cow<'static, str>>,
    #[serde(
        default = "diagnostic_serde::workspace_delay",
        skip_serializing_if = "diagnostic_serde::three_minute_validate"
    )]
    pub workspace_delay: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_event: Option<Event>,
    #[serde(
        default = "diagnostic_serde::workspace_rate",
        skip_serializing_if = "diagnostic_serde::full_percent_validate"
    )]
    pub workspace_rate: usize,

    #[serde(flatten)]
    pub custom: O,
}

mod diagnostic_serde {
    #[inline]
    pub fn workspace_delay() -> usize { 3000 }
    #[inline]
    pub fn workspace_rate() -> usize { 100 }
    pub const fn three_minute_validate(ctx: &usize) -> bool { *ctx == 3000 }
    pub const fn full_percent_validate(ctx: &usize) -> bool { *ctx == 100 }
}

impl<O: Default> Default for Diagnostics<O> {
    fn default() -> Self {
        Self {
            enable: true,
            disable: Vec::default(),
            disable_scheme: Vec::default(),
            globals: Vec::default(),
            group_file_status: BTreeMap::default(),
            group_severity: BTreeMap::default(),
            ignored_files: None,
            library_files: None,
            unused_local_exclude: Vec::default(),
            workspace_delay: 3000,
            workspace_event: None,
            workspace_rate: 100,
            needed_file_status: BTreeMap::default(),
            severity: BTreeMap::default(),

            custom: Default::default(),
        }
    }
}

impl<O> Deref for Diagnostics<O> {
    type Target = O;
    fn deref(&self) -> &Self::Target {
        &self.custom
    }
}

impl<O> DerefMut for Diagnostics<O> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.custom
    }
}

#[derive(Default, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Doc<O = ()> {
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub package_name: HashSet<Cow<'static, str>>,

    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub private_name: HashSet<Cow<'static, str>>,

    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub protected_name: HashSet<Cow<'static, str>>,

    #[serde(flatten)]
    pub custom: O,
}

impl<O> Deref for Doc<O> {
    type Target = O;
    fn deref(&self) -> &Self::Target {
        &self.custom
    }
}
impl<O> DerefMut for Doc<O> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.custom
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Format<O = ()> {
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub enable: bool,

    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub default_config: BTreeMap<Cow<'static, str>, Cow<'static, str>>,

    #[serde(flatten)]
    pub custom: O,
}

impl<O: Default> Default for Format<O> {
    fn default() -> Self {
        Self {
            enable: true,
            default_config: BTreeMap::default(),

            custom: Default::default(),
        }
    }
}

impl<O> Deref for Format<O> {
    type Target = O;
    fn deref(&self) -> &Self::Target {
        &self.custom
    }
}
impl<O> DerefMut for Format<O> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.custom
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Index {
    Enable,
    Auto,
    Disable,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum ParamName {
    All,
    Literal,
    Disable,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum SemiColon {
    All,
    SameLine,
    Disable,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Hint<O = ()> {
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub enable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub array_index: Option<Index>,
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub r#await: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param_name: Option<ParamName>,
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub param_type: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semicolon: Option<SemiColon>,
    #[serde(default, skip_serializing_if = "disabled")]
    pub set_type: bool,

    #[serde(flatten)]
    pub custom: O,
}

impl<O: Default> Default for Hint<O> {
    fn default() -> Self {
        Self {
            enable: true,
            array_index: None,
            r#await: true,
            param_name: None,
            param_type: true,
            semicolon: None,
            set_type: false,

            custom: Default::default(),
        }
    }
}

impl<O> Deref for Hint<O> {
    type Target = O;
    fn deref(&self) -> &Self::Target {
        &self.custom
    }
}
impl<O> DerefMut for Hint<O> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.custom
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Hover<O = ()> {
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub enable: bool,
    #[serde(default, skip_serializing_if = "hover_serde::enum_limit_validate")]
    pub enums_limit: usize,
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub expand_alias: bool,
    #[serde(default, skip_serializing_if = "hover_serde::preview_fields_validate")]
    pub preview_fields: usize,
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub view_number: bool,
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub view_string: bool,
    #[serde(default, skip_serializing_if = "hover_serde::view_string_max_validate")]
    pub view_string_max: usize,

    #[serde(flatten)]
    pub custom: O,
}

impl<O> Deref for Hover<O> {
    type Target = O;
    fn deref(&self) -> &Self::Target {
        &self.custom
    }
}
impl<O> DerefMut for Hover<O> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.custom
    }
}

impl<O: Default> Default for Hover<O> {
    fn default() -> Self {
        Self {
            enable: true,
            enums_limit: 5,
            expand_alias: true,
            preview_fields: 50,
            view_number: true,
            view_string: true,
            view_string_max: 1000,

            custom: Default::default(),
        }
    }
}

mod hover_serde {
    pub const fn enum_limit_validate(ctx: &usize) -> bool { *ctx == 5 }
    pub const fn preview_fields_validate(ctx: &usize) -> bool { *ctx == 50 }
    pub const fn view_string_max_validate(ctx: &usize) -> bool { *ctx == 1000 }
}

#[derive(Default, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Misc<O = ()> {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<Cow<'static, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executable_path: Option<Cow<'static, str>>,

    #[serde(flatten)]
    pub custom: O,
}

impl<O> Deref for Misc<O> {
    type Target = O;
    fn deref(&self) -> &Self::Target {
        &self.custom
    }
}
impl<O> DerefMut for Misc<O> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.custom
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Status {
    Default,
    Enable,
    Disable,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Encoding {
    Utf8,
    Ansi,
    Utf16le,
    Utf16be,
}

#[derive(Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Runtime<O = ()> {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub builtin: BTreeMap<Cow<'static, str>, Status>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_encoding: Option<Encoding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Cow<'static, str>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nonstandard_symbol: Vec<Cow<'static, str>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub path: Vec<Cow<'static, str>>,
    #[serde(default, skip_serializing_if = "disabled")]
    pub path_strict: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin: Option<Cow<'static, str>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plugin_args: Vec<Cow<'static, str>>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub special: BTreeMap<Cow<'static, str>, Cow<'static, str>>,
    #[serde(default, skip_serializing_if = "disabled")]
    pub unicode_name: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<Cow<'static, str>>,

    #[serde(flatten)]
    pub custom: O
}

impl<O> Deref for Runtime<O> {
    type Target = O;
    fn deref(&self) -> &Self::Target {
        &self.custom
    }
}
impl<O> DerefMut for Runtime<O> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.custom
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Semantic<O = ()> {
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub enable: bool,
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub annotation: bool,
    #[serde(default, skip_serializing_if = "disabled")]
    pub keyword: bool,
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub variable: bool,

    #[serde(flatten)]
    pub custom: O,
}
impl<O> Deref for Semantic<O> {
    type Target = O;
    fn deref(&self) -> &Self::Target {
        &self.custom
    }
}
impl<O> DerefMut for Semantic<O> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.custom
    }
}
impl<O: Default> Default for Semantic<O> {
    fn default() -> Self {
        Self {
            enable: true,
            annotation: true,
            keyword: false,
            variable: true,

            custom: Default::default(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SignatureHelp<O = ()> {
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub enable: bool,

    #[serde(flatten)]
    pub custom: O,
}
impl<O> Deref for SignatureHelp<O> {
    type Target = O;
    fn deref(&self) -> &Self::Target {
        &self.custom
    }
}
impl<O> DerefMut for SignatureHelp<O> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.custom
    }
}
impl<O: Default> Default for SignatureHelp<O> {
    fn default() -> Self {
        Self {
            enable: true,
            custom: Default::default(),
        }
    }
}

#[derive(Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Spell<O = ()> {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dict: Vec<Cow<'static, str>>,

    #[serde(flatten)]
    pub custom: O,
}
impl<O> Deref for Spell<O> {
    type Target = O;
    fn deref(&self) -> &Self::Target {
        &self.custom
    }
}
impl<O> DerefMut for Spell<O> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.custom
    }
}

#[derive(Default, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Type<O = ()> {
    #[serde(default, skip_serializing_if = "disabled")]
    pub cast_number_to_integer: bool,
    #[serde(default, skip_serializing_if = "disabled")]
    pub weak_nil_check: bool,
    #[serde(default, skip_serializing_if = "disabled")]
    pub weak_union_check: bool,

    #[serde(flatten)]
    pub custom: O,
}
impl<O> Deref for Type<O> {
    type Target = O;
    fn deref(&self) -> &Self::Target {
        &self.custom
    }
}
impl<O> DerefMut for Type<O> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.custom
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Window<O = ()> {
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub progress_bar: bool,
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub status_bar: bool,

    #[serde(flatten)]
    pub custom: O,
}
impl<O> Deref for Window<O> {
    type Target = O;
    fn deref(&self) -> &Self::Target {
        &self.custom
    }
}
impl<O> DerefMut for Window<O> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.custom
    }
}

impl<O: Default> Default for Window<O> {
    fn default() -> Self {
        Self {
            progress_bar: true,
            status_bar: true,
            custom: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum CheckThirdParty {
    Ask,
    Apply,
    ApplyInMemory,
    Disable,
    False,
}

impl Serialize for CheckThirdParty {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Ask => serializer.serialize_str("Ask"),
            Self::Apply => serializer.serialize_str("Apply"),
            Self::ApplyInMemory => serializer.serialize_str("ApplyInMemory"),
            Self::Disable => serializer.serialize_str("Disable"),
            Self::False => serializer.serialize_bool(false),
        }
    }
}

impl<'de> Deserialize<'de> for CheckThirdParty {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct CheckThirdPartyVisitor;
        impl Visitor<'_> for CheckThirdPartyVisitor {
            type Value = CheckThirdParty;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, r#""Ask", "Apply", "ApplyInMemory", "Disable", or false"#)
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_str(v.as_str())
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    "Ask" => Ok(CheckThirdParty::Ask),
                    "Apply" => Ok(CheckThirdParty::Apply),
                    "ApplyInMemory" => Ok(CheckThirdParty::ApplyInMemory),
                    "Disable" => Ok(CheckThirdParty::Disable),
                    other => Err(serde::de::Error::custom(format!(
                        "unknown checkThirdParty value: {other}"
                    ))),
                }
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v {
                    return Err(serde::de::Error::custom(
                        "checkThirdParty cannot be set to `true`",
                    ));
                }
                Ok(CheckThirdParty::False)
            }
        }

        deserializer.deserialize_any(CheckThirdPartyVisitor)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Workspace<O = ()> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_third_party: Option<CheckThirdParty>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ignore_dir: Vec<Cow<'static, str>>,
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub ignore_submodules: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub library: Vec<Cow<'static, str>>,
    #[serde(
        default = "workspace_serde::max_preload",
        skip_serializing_if = "workspace_serde::max_preload_validate"
    )]
    pub max_preload: usize,
    #[serde(
        default = "workspace_serde::preload_file_size",
        skip_serializing_if = "workspace_serde::preload_file_size_validate"
    )]
    pub preload_file_size: usize,
    #[serde(default = "default_true", skip_serializing_if = "enabled")]
    pub use_git_ignore: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub user_third_party: Vec<Cow<'static, str>>,

    /// Collect remaining user defined data
    #[serde(flatten)]
    pub custom: O,
}

impl<O> Deref for Workspace<O> {
    type Target = O;
    fn deref(&self) -> &Self::Target {
        &self.custom
    }
}

impl<O> DerefMut for Workspace<O> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.custom
    }
}

mod workspace_serde {
    pub const fn max_preload() -> usize { 5000 }
    pub const fn preload_file_size() -> usize { 500 }
    pub const fn max_preload_validate(ctx: &usize) -> bool { *ctx == 5000 }
    pub const fn preload_file_size_validate(ctx: &usize) -> bool { *ctx == 500 }
}

impl<O: Default> Default for Workspace<O> {
    fn default() -> Self {
        Self {
            check_third_party: None,
            ignore_dir: Vec::default(),
            ignore_submodules: true,
            library: Vec::default(),
            max_preload: 5000,
            preload_file_size: 500,
            use_git_ignore: true,
            user_third_party: Vec::default(),
            custom: Default::default(),
        }
    }
}

macro_rules! rc {
    {
        $(#[$attr: meta])*
        pub struct $name: ident < $($letter: ident),+; O > { $($rest: tt)* }
    } => {
        $(#[$attr])*
        pub struct $name< $($letter = (),)+ O = ()> { $($rest)* }

        impl<$($letter,)* O> Deref for LuaRc<$($letter,)* O> {
            type Target = O;

            fn deref(&self) -> &Self::Target {
                &self.custom
            }
        }

        impl<$($letter,)* O> DerefMut for LuaRc<$($letter,)* O> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.custom
            }
        }

        impl<$($letter,)* O> LuaRc<$($letter,)* O>
            where
                $($letter: Serialize + DeserializeOwned,)*
                O: Default + Serialize + DeserializeOwned,
        {
            /// Write the instance to a given path
            pub fn write(&self, path: impl AsRef<Path>) -> Result<(), Error> {
                Ok(std::fs::write(path, serde_json::to_string_pretty(self)?)?)
            }

            /// Read a LuaRc instance from a file
            ///
            /// # Args
            ///     - path: The full path to the new luarc file, including the filename
            ///
            /// # Returns
            ///     - `Err`: when it fails to parse the json
            ///     - `Ok(None)`: when no file was found
            ///     - `Ok(Some(_))`: when the file was found and it successfully parsed the json
            ///
            /// **Warning**: Since LuaRc allows for flattened type parsing from the remaining
            /// fields from each section, Using this method directly will required you to specify
            /// each type. USE [[`LuaRc::extend`]] instead
            pub fn read_as(path: impl AsRef<Path>) -> Result<Option<Self>, Error> {
                let path = path.as_ref();
                match std::fs::read(path).ok() {
                    None => Ok(None),
                    Some(bytes) => {
                        Ok(Some(serde_json::from_slice(&bytes)?))
                    }
                }
            }
        }

        impl<$($letter,)* O: Default> Default for LuaRc<$($letter,)* O> {
            fn default() -> Self {
                Self {
                    schema: None,
                    addon_manager: None,
                    completion: None,
                    diagnostics: None,
                    doc: None,
                    format: None,
                    hint: None,
                    hover: None,
                    misc: None,
                    runtime: None,
                    semantic: None,
                    signature_help: None,
                    spell: None,
                    r#type: None,
                    workspace: None,
                    custom: Default::default(),
                }
            }
        }

        impl<$($letter,)* O> std::fmt::Debug for LuaRc<$($letter,)* O>
        where
            $($letter: std::fmt::Debug,)*
            O: std::fmt::Debug,
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct("LuaRc")
                    .field("schema", &self.schema)
                    .field("addon_manager", &self.addon_manager)
                    .field("completion", &self.completion)
                    .field("diagnostics", &self.diagnostics)
                    .field("doc", &self.doc)
                    .field("format", &self.format)
                    .field("hint", &self.hint)
                    .field("hover", &self.hover)
                    .field("misc", &self.misc)
                    .field("runtime", &self.runtime)
                    .field("semantic", &self.semantic)
                    .field("signature_help", &self.signature_help)
                    .field("spell", &self.spell)
                    .field("type", &self.r#type)
                    .field("workspace", &self.workspace)
                    .field("other", &self.custom)
                    .finish()
            }
        }

        impl<$($letter,)* O> PartialEq for LuaRc<$($letter,)* O>
            where
                $($letter: PartialEq,)*
                O: PartialEq,
        {
            #[allow(clippy::unit_arg)]
            fn eq(&self, other: &Self) -> bool {
                self.schema.eq(&other.schema)
                    && self.addon_manager.eq(&other.addon_manager)
                    && self.completion.eq(&other.completion)
                    && self.diagnostics.eq(&other.diagnostics)
                    && self.doc.eq(&other.doc)
                    && self.format.eq(&other.format)
                    && self.hint.eq(&other.hint)
                    && self.hover.eq(&other.hover)
                    && self.misc.eq(&other.misc)
                    && self.runtime.eq(&other.runtime)
                    && self.semantic.eq(&other.semantic)
                    && self.signature_help.eq(&other.signature_help)
                    && self.spell.eq(&other.spell)
                    && self.r#type.eq(&other.r#type)
                    && self.workspace.eq(&other.workspace)
                    && self.custom.eq(&other.custom)
            }
        }

        pub struct LuaRcBuilder<$($letter = (),)* O = ()> {
            _m: PhantomData<($($letter,)* O)>
        }

        impl<$($letter,)* O> LuaRcBuilder<$($letter,)* O>
            where
                $($letter: Serialize + DeserializeOwned,)*
                O: Default + Serialize + DeserializeOwned,
        {
            pub fn read(self, path: impl AsRef<Path>) -> Result<Option<LuaRc<$($letter,)* O>>, Error> {
                LuaRc::read_as(path)
            }
        }
    }
}

rc! {
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all="camelCase")]
    pub struct LuaRc<A, B, C, D, E, F, G, H, I, J, K, L, M, N; O> {
        #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
        pub schema: Option<Cow<'static, str>>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub addon_manager: Option<AddonManager<A>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub completion: Option<Completion<B>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub diagnostics: Option<Diagnostics<C>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub doc: Option<Doc<D>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub format: Option<Format<E>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub hint: Option<Hint<F>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub hover: Option<Hover<G>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub misc: Option<Misc<H>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub runtime: Option<Runtime<I>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub semantic: Option<Semantic<J>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub signature_help: Option<SignatureHelp<K>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub spell: Option<Spell<L>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub r#type: Option<Type<M>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub workspace: Option<Workspace<N>>,

        #[serde(flatten)]
        pub custom: O,
    }
}

impl LuaRc {
    pub fn extend() -> LuaRcBuilder {
        LuaRcBuilder::default()
    }

    /// Read a LuaRc instance from a file
    ///
    /// Fails if the file does not exist.
    ///
    /// # Args
    ///     - path: The full path to the new luarc file, including the filename
    pub fn read(path: impl AsRef<Path>) -> Result<Option<Self>, Error> {
        Self::read_as(path)
    }
}

impl Default for LuaRcBuilder {
    fn default() -> Self {
        Self { _m: PhantomData }
    }
}

#[allow(clippy::type_complexity)]
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O> LuaRcBuilder<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O> {
    #[inline]
    pub fn addon_manager<Z>(self) -> LuaRcBuilder<Z, B, C, D, E, F, G, H, I, J, K, L, M, N, O> { LuaRcBuilder { _m: PhantomData } }
    #[inline]
    pub fn completion<Z>(self) -> LuaRcBuilder<A, Z, C, D, E, F, G, H, I, J, K, L, M, N, O> { LuaRcBuilder { _m: PhantomData } }
    #[inline]
    pub fn diagnostics<Z>(self) -> LuaRcBuilder<A, B, Z, D, E, F, G, H, I, J, K, L, M, Z, O> { LuaRcBuilder { _m: PhantomData } }
    #[inline]
    pub fn doc<Z>(self) -> LuaRcBuilder<A, B, C, Z, E, F, G, H, I, J, K, L, M, N, O> { LuaRcBuilder { _m: PhantomData } }
    #[inline]
    pub fn format<Z>(self) -> LuaRcBuilder<A, B, C, D, Z, F, G, H, I, J, K, L, M, N, O> { LuaRcBuilder { _m: PhantomData } }
    #[inline]
    pub fn hint<Z>(self) -> LuaRcBuilder<A, B, C, D, E, Z, G, H, I, J, K, L, M, N, O> { LuaRcBuilder { _m: PhantomData } }
    #[inline]
    pub fn hover<Z>(self) -> LuaRcBuilder<A, B, C, D, E, F, Z, H, I, J, K, L, M, N, O> { LuaRcBuilder { _m: PhantomData } }
    #[inline]
    pub fn misc<Z>(self) -> LuaRcBuilder<A, B, C, D, E, F, G, Z, I, J, K, L, M, N, O> { LuaRcBuilder { _m: PhantomData } }
    #[inline]
    pub fn runtime<Z>(self) -> LuaRcBuilder<A, B, C, D, E, F, G, H, Z, J, K, L, M, N, O> { LuaRcBuilder { _m: PhantomData } }
    #[inline]
    pub fn semantic<Z>(self) -> LuaRcBuilder<A, B, C, D, E, F, G, H, I, Z, K, L, M, N, O> { LuaRcBuilder { _m: PhantomData } }
    #[inline]
    pub fn signature_help<Z>(self) -> LuaRcBuilder<A, B, C, D, E, F, G, H, I, J, Z, L, M, N, O> { LuaRcBuilder { _m: PhantomData } }
    #[inline]
    pub fn spell<Z>(self) -> LuaRcBuilder<A, B, C, D, E, F, G, H, I, J, K, Z, M, N, O> { LuaRcBuilder { _m: PhantomData } }
    #[inline]
    pub fn ty<Z>(self) -> LuaRcBuilder<A, B, C, D, E, F, G, H, I, J, K, L, Z, N, O> { LuaRcBuilder { _m: PhantomData } }
    #[inline]
    pub fn workspace<Z>(self) -> LuaRcBuilder<A, B, C, D, E, F, G, H, I, J, K, L, M, Z, O> { LuaRcBuilder { _m: PhantomData } }
    #[inline]
    pub fn root<Z>(self) -> LuaRcBuilder<A, B, C, D, E, F, G, H, I, J, K, L, M, N, Z> { LuaRcBuilder { _m: PhantomData } }
}
