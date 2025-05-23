use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Ambiguity {
    Ambiguity1,
    CountDownLoop,
    DifferentRequires,
    NewfieldCall,
    NewlineCall,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Await {
    AwaitInSync,
    NotYieldable,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Codestyle {
    CodestyleCheck,
    NameStyleCheck,
    SpellCheck,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Conventions {
    GlobalElement,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Duplicate {
    DuplicateIndex,
    DuplicateSetField,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Global {
    GlobalInNilEnv,
    LowercaseGlobal,
    UndefinedEnvChild,
    UndefinedGlobal,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Luadoc {
    CastTypeMismatch,
    CircleDocClass,
    DocFieldNoClass,
    DuplicateDocAlias,
    DuplicateDocField,
    DuplicateDocParam,
    IncompleteSignatureDoc,
    MissingGlobalDoc,
    MissingLocalExportDoc,
    UndefinedDocClass,
    UndefinedDocName,
    UndefinedDocParam,
    UnknownCastVariable,
    UnknownDiagCode,
    UnknownOperator,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Redefined {
    RedefinedLocal,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Strict {
    CloseNonObject,
    Deprecated,
    DiscardReturns,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Strong {
    NoUnknown,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum TypeCheck {
    AssignTypeMismatch,
    CastLocalType,
    CastTypeMismatch,
    InjectField,
    NeedCheckNil,
    ParamTypeMismatch,
    ReturnTypeMismatch,
    UndefinedField,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Unbalanced {
    MissingFields,
    MissingParameter,
    MissingReturn,
    MissingReturnValue,
    RedundantParameter,
    RedundantReturnValue,
    RedundantValue,
    UnbalancedAssignments,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Unused {
    CodeAfterBreak,
    EmptyBlock,
    RedundantReturn,
    TrailingSpace,
    UnreachableCode,
    UnusedFunction,
    UnusedLabel,
    UnusedLocal,
    UnusedVararg,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(untagged)]
pub enum Diagnostic {
    Ambiguity(Ambiguity),
    Await(Await),
    Codestyle(Codestyle),
    Conventions(Conventions),
    Duplicate(Duplicate),
    Global(Global),
    Luadoc(Luadoc),
    Redefined(Redefined),
    Strict(Strict),
    Strong(Strong),
    TypeCheck(TypeCheck),
    Unbalanced(Unbalanced),
    Unused(Unused),
}

impl FromStr for Diagnostic {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, String> {
        if !input.contains(':') {
            return Err("diagnostics must be of the format of <group>:<name>".to_string());
        }

        let (group, name) = input.split_once(':').unwrap();

        Ok(match (group, name) {
            ("ambiguity", "ambiguity-1") => Self::Ambiguity(Ambiguity::Ambiguity1),
            ("ambiguity", "count-down-loop") => Self::Ambiguity(Ambiguity::CountDownLoop),
            ("ambiguity", "different-requires") => Self::Ambiguity(Ambiguity::DifferentRequires),
            ("ambiguity", "newfield-call") => Self::Ambiguity(Ambiguity::NewfieldCall),
            ("ambiguity", "newline-call") => Self::Ambiguity(Ambiguity::NewlineCall),
            ("await", "await-in-sync") => Self::Await(Await::AwaitInSync),
            ("await", "not-yieldable") => Self::Await(Await::NotYieldable),
            ("codestyle", "codestyle-check") => Self::Codestyle(Codestyle::CodestyleCheck),
            ("codestyle", "name-style-check") => Self::Codestyle(Codestyle::NameStyleCheck),
            ("codestyle", "spell-check") => Self::Codestyle(Codestyle::SpellCheck),
            ("conventions", "global-element") => Self::Conventions(Conventions::GlobalElement),
            ("duplicate", "duplicate-index") => Self::Duplicate(Duplicate::DuplicateIndex),
            ("duplicate", "duplicate-set-field") => Self::Duplicate(Duplicate::DuplicateSetField),
            ("global", "global-in-nil-env") => Self::Global(Global::GlobalInNilEnv),
            ("global", "lowercase-global") => Self::Global(Global::LowercaseGlobal),
            ("global", "undefined-env-child") => Self::Global(Global::UndefinedEnvChild),
            ("global", "undefined-global") => Self::Global(Global::UndefinedGlobal),
            ("luadoc", "cast-type-mismatch") => Self::Luadoc(Luadoc::CastTypeMismatch),
            ("luadoc", "circle-doc-class") => Self::Luadoc(Luadoc::CircleDocClass),
            ("luadoc", "doc-field-no-class") => Self::Luadoc(Luadoc::DocFieldNoClass),
            ("luadoc", "duplicate-doc-alias") => Self::Luadoc(Luadoc::DuplicateDocAlias),
            ("luadoc", "DuplicateDocField") => Self::Luadoc(Luadoc::DuplicateDocField),
            ("luadoc", "duplicate-doc-param") => Self::Luadoc(Luadoc::DuplicateDocParam),
            ("luadoc", "incomplete-signature-doc") => Self::Luadoc(Luadoc::IncompleteSignatureDoc),
            ("luadoc", "missing-global-doc") => Self::Luadoc(Luadoc::MissingGlobalDoc),
            ("luadoc", "missing-local-export-doc") => Self::Luadoc(Luadoc::MissingLocalExportDoc),
            ("luadoc", "undefined-doc-class") => Self::Luadoc(Luadoc::UndefinedDocClass),
            ("luadoc", "undefined-doc-name") => Self::Luadoc(Luadoc::UndefinedDocName),
            ("luadoc", "undefined-doc-param") => Self::Luadoc(Luadoc::UndefinedDocParam),
            ("luadoc", "unknown-cast-variable") => Self::Luadoc(Luadoc::UnknownCastVariable),
            ("luadoc", "unknown-diag-code") => Self::Luadoc(Luadoc::UnknownDiagCode),
            ("luadoc", "unknown-operator") => Self::Luadoc(Luadoc::UnknownOperator),
            ("redefined", "redefined-local") => Self::Redefined(Redefined::RedefinedLocal),
            ("strict", "close-non-object") => Self::Strict(Strict::CloseNonObject),
            ("strict", "deprecated") => Self::Strict(Strict::Deprecated),
            ("strict", "discard-returns") => Self::Strict(Strict::DiscardReturns),
            ("strong", "no-unknown") => Self::Strong(Strong::NoUnknown),
            ("typecheck", "assign-type-mismatch") => Self::TypeCheck(TypeCheck::AssignTypeMismatch),
            ("typecheck", "cast-local-type") => Self::TypeCheck(TypeCheck::CastLocalType),
            ("typecheck", "cast-type-mismatch") => Self::TypeCheck(TypeCheck::CastTypeMismatch),
            ("typecheck", "inject-field") => Self::TypeCheck(TypeCheck::InjectField),
            ("typecheck", "need-check-nil") => Self::TypeCheck(TypeCheck::NeedCheckNil),
            ("typecheck", "param-type-mismatch") => Self::TypeCheck(TypeCheck::ParamTypeMismatch),
            ("typecheck", "return-type-mismatch") => Self::TypeCheck(TypeCheck::ReturnTypeMismatch),
            ("typecheck", "undefined-field") => Self::TypeCheck(TypeCheck::UndefinedField),
            ("unbalanced", "missing-fields") => Self::Unbalanced(Unbalanced::MissingFields),
            ("unbalanced", "missing-parameter") => Self::Unbalanced(Unbalanced::MissingParameter),
            ("unbalanced", "missing-return") => Self::Unbalanced(Unbalanced::MissingReturn),
            ("unbalanced", "missing-return-value") => {
                Self::Unbalanced(Unbalanced::MissingReturnValue)
            }
            ("unbalanced", "redundant-parameter") => {
                Self::Unbalanced(Unbalanced::RedundantParameter)
            }
            ("unbalanced", "redundant-return-value") => {
                Self::Unbalanced(Unbalanced::RedundantReturnValue)
            }
            ("unbalanced", "redundant-value") => Self::Unbalanced(Unbalanced::RedundantValue),
            ("unbalanced", "unbalanced-assignments") => {
                Self::Unbalanced(Unbalanced::UnbalancedAssignments)
            }
            ("unused", "code-after-break") => Self::Unused(Unused::CodeAfterBreak),
            ("unused", "empty-block") => Self::Unused(Unused::EmptyBlock),
            ("unused", "redundant-return") => Self::Unused(Unused::RedundantReturn),
            ("unused", "trailing-space") => Self::Unused(Unused::TrailingSpace),
            ("unused", "unreachable-code") => Self::Unused(Unused::UnreachableCode),
            ("unused", "unused-function") => Self::Unused(Unused::UnusedFunction),
            ("unused", "unused-label") => Self::Unused(Unused::UnusedLabel),
            ("unused", "unused-local") => Self::Unused(Unused::UnusedLocal),
            ("unused", "unused-vararg") => Self::Unused(Unused::UnusedVararg),
            (group, name) => return Err(format!("invalid lua diagnostic: {group}:{name}")),
        })
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticGroup {
    Ambiguity,
    Await,
    Codestyle,
    Conventions,
    Duplicate,
    Global,
    Luadoc,
    Redefined,
    Strict,
    Strong,
    TypeCheck,
    Unbalanced,
    Unused,
}
