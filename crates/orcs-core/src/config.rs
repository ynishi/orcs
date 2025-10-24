use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum PersonaSource {
    System,
    User,
}

impl Default for PersonaSource {
    fn default() -> Self {
        PersonaSource::User
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct ConfigRoot {
    #[serde(rename = "persona")]
    pub personas: Vec<PersonaConfig>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PersonaConfig {
    pub id: String,
    pub name: String,
    pub role: String,
    pub background: String,
    pub communication_style: String,
    #[serde(default)]
    pub default_participant: bool,
    #[serde(default)]
    pub source: PersonaSource,
}
