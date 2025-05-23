use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone, Copy, strum::Display)]
#[strum(serialize_all = "camelCase")]
pub enum Refs {
    Tags,
    Heads,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ReferenceObject {
    pub sha: String,
    pub r#type: String,
    pub url: String,
}


#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Reference {
    /// The name of the ref with the `refs/heads/` or `refs/tags/` prefix stripped
    #[serde(deserialize_with = "deserialize_ref_name")]
    pub r#ref: String,
    pub node_id: String,
    pub url: String,
    pub object: ReferenceObject
}

fn deserialize_ref_name<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>
{
    let value = String::deserialize(deserializer)?;

    if let Some(value) = value.strip_prefix("refs/heads/") {
        Ok(value.to_string())
    } else if let Some(value) = value.strip_prefix("refs/tags/") {
        Ok(value.to_string())
    } else {
        Ok(value.to_string())
    }
}
