use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub x: f32,
    pub z: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioUnit {
    pub id: String,
    pub name: String,
    pub class_name: String,
    pub side: String,
    pub status: String,
    pub position: Position,
    pub heading: f32,
    pub speed_knots: u8,
    pub health: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioContact {
    pub id: String,
    pub label: String,
    pub classification: String,
    pub confidence: u8,
    pub bearing: u16,
    pub range_nm: f32,
    pub domain: String,
    pub position: Position,
    pub identified: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioObjective {
    pub id: String,
    pub title: String,
    pub description: String,
    pub kind: String,
    pub required: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioDefinition {
    pub schema_version: u16,
    pub id: String,
    pub title: String,
    pub theatre: String,
    pub start_time_seconds: u64,
    pub units: Vec<ScenarioUnit>,
    pub contacts: Vec<ScenarioContact>,
    pub objectives: Vec<ScenarioObjective>,
}

#[derive(Debug)]
pub struct ScenarioError(pub String);

impl fmt::Display for ScenarioError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ScenarioError {}

impl ScenarioDefinition {
    pub fn parse(json: &str) -> Result<Self, ScenarioError> {
        let scenario: Self = serde_json::from_str(json)
            .map_err(|error| ScenarioError(format!("Invalid scenario JSON: {error}")))?;
        scenario.validate()?;
        Ok(scenario)
    }

    pub fn validate(&self) -> Result<(), ScenarioError> {
        let invalid = |message: &str| Err(ScenarioError(message.to_string()));
        if self.schema_version != 1 {
            return invalid("Unsupported scenario schema version; expected version 1");
        }
        if self.id.trim().is_empty()
            || self.title.trim().is_empty()
            || self.theatre.trim().is_empty()
        {
            return invalid("Scenario id, title, and theatre must not be empty");
        }
        if self.units.is_empty() || self.units.len() > 16 {
            return invalid("A scenario must contain between 1 and 16 units");
        }
        if self.contacts.len() > 32 {
            return invalid("A scenario cannot contain more than 32 contacts");
        }
        if self.objectives.is_empty() || self.objectives.len() > 8 {
            return invalid("A scenario must contain between 1 and 8 objectives");
        }

        let mut ids = HashSet::new();
        for unit in &self.units {
            if unit.id.trim().is_empty() || !ids.insert(unit.id.as_str()) {
                return invalid("Unit and contact ids must be non-empty and unique");
            }
            if unit.name.trim().is_empty() || unit.class_name.trim().is_empty() {
                return invalid("Unit names and classes must not be empty");
            }
            if !matches!(unit.side.as_str(), "friendly" | "hostile" | "neutral") {
                return invalid("Unit side must be friendly, hostile, or neutral");
            }
            if !(0.0..360.0).contains(&unit.heading)
                || unit.speed_knots > 50
                || !(1..=100).contains(&unit.health)
            {
                return invalid("Unit heading, speed, or health is outside supported limits");
            }
            if !valid_position(&unit.position) {
                return invalid("Unit position must be finite and within 50 map units");
            }
        }

        for contact in &self.contacts {
            if contact.id.trim().is_empty() || !ids.insert(contact.id.as_str()) {
                return invalid("Unit and contact ids must be non-empty and unique");
            }
            if contact.label.trim().is_empty() || contact.classification.trim().is_empty() {
                return invalid("Contact labels and classifications must not be empty");
            }
            if contact.confidence > 100
                || contact.bearing > 359
                || !contact.range_nm.is_finite()
                || contact.range_nm <= 0.0
            {
                return invalid(
                    "Contact confidence, bearing, or range is outside supported limits",
                );
            }
            if !matches!(contact.domain.as_str(), "surface" | "air" | "subsurface") {
                return invalid("Contact domain must be surface, air, or subsurface");
            }
            if !valid_position(&contact.position) {
                return invalid("Contact position must be finite and within 50 map units");
            }
        }

        let mut objective_ids = HashSet::new();
        for objective in &self.objectives {
            if objective.id.trim().is_empty()
                || objective.title.trim().is_empty()
                || !objective_ids.insert(objective.id.as_str())
            {
                return invalid(
                    "Objective ids and titles must be non-empty and objective ids unique",
                );
            }
            if objective.kind != "classify_contacts"
                || objective.required == 0
                || usize::from(objective.required) > self.contacts.len()
            {
                return invalid("Objective kind or required contact count is unsupported");
            }
        }
        Ok(())
    }
}

fn valid_position(position: &Position) -> bool {
    position.x.is_finite()
        && position.z.is_finite()
        && position.x.abs() <= 50.0
        && position.z.abs() <= 50.0
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignDefinition {
    pub schema_version: u16,
    pub campaign_id: String,
    pub title: String,
    pub description: String,
    pub scenarios: Vec<String>,
}

impl CampaignDefinition {
    pub fn parse(json: &str) -> Result<Self, ScenarioError> {
        let campaign: Self = serde_json::from_str(json)
            .map_err(|error| ScenarioError(format!("Invalid campaign JSON: {error}")))?;
        campaign.validate()?;
        Ok(campaign)
    }

    pub fn validate(&self) -> Result<(), ScenarioError> {
        if self.schema_version != 1 {
            return Err(ScenarioError(
                "Unsupported campaign schema version; expected version 1".into(),
            ));
        }
        if self.campaign_id.trim().is_empty()
            || self.title.trim().is_empty()
            || self.description.trim().is_empty()
        {
            return Err(ScenarioError(
                "Campaign id, title, and description must not be empty".into(),
            ));
        }
        if self.scenarios.is_empty() || self.scenarios.len() > 32 {
            return Err(ScenarioError(
                "A campaign must contain between 1 and 32 scenarios".into(),
            ));
        }
        let mut ids = HashSet::new();
        if self
            .scenarios
            .iter()
            .any(|scenario_id| scenario_id.trim().is_empty() || !ids.insert(scenario_id.as_str()))
        {
            return Err(ScenarioError(
                "Campaign scenario ids must be non-empty and unique".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{CampaignDefinition, ScenarioDefinition};

    const SCENARIO: &str = include_str!("../../../scenarios/north-atlantic-intercept.json");
    const CAMPAIGN: &str = include_str!("../../../campaigns/northern-watch.json");

    #[test]
    fn parses_starter_scenario() {
        let scenario = ScenarioDefinition::parse(SCENARIO).expect("starter scenario is valid");
        assert_eq!(scenario.id, "north-atlantic-intercept");
        assert_eq!(scenario.units.len(), 3);
        assert_eq!(scenario.contacts.len(), 3);
    }

    #[test]
    fn rejects_duplicate_unit_and_contact_ids() {
        let duplicate_id = SCENARIO.replace("\"id\": \"c-101\"", "\"id\": \"ddg-01\"");
        let error = ScenarioDefinition::parse(&duplicate_id).unwrap_err();
        assert!(error
            .to_string()
            .contains("ids must be non-empty and unique"));
    }

    #[test]
    fn parses_starter_campaign_and_rejects_duplicate_scenarios() {
        let campaign = CampaignDefinition::parse(CAMPAIGN).expect("starter campaign is valid");
        assert_eq!(campaign.campaign_id, "north-atlantic-patrol");
        assert_eq!(campaign.scenarios, ["north-atlantic-intercept"]);

        let duplicate = CAMPAIGN.replace(
            "\"scenarios\": [\"north-atlantic-intercept\"]",
            "\"scenarios\": [\"north-atlantic-intercept\", \"north-atlantic-intercept\"]",
        );
        let error = CampaignDefinition::parse(&duplicate).unwrap_err();
        assert!(error.to_string().contains("must be non-empty and unique"));
    }
}
