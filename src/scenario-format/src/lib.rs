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
    #[serde(default)]
    pub sensors: Vec<SensorDefinition>,
    #[serde(default)]
    pub weapons: Vec<WeaponDefinition>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeaponDefinition {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub range_nm: f32,
    pub domains: Vec<String>,
    pub ammunition: u16,
    pub hit_chance_percent: u8,
    pub speed_knots: u16,
    #[serde(default = "default_weapon_damage_percent")]
    pub damage_percent: u8,
    #[serde(default)]
    pub effect_duration_seconds: u16,
}

fn default_weapon_damage_percent() -> u8 {
    100
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DefenseSystemDefinition {
    pub id: String,
    pub name: String,
    pub charges: u8,
    pub intercept_chance_percent: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SensorDefinition {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub range_nm: f32,
    pub domains: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SatellitePassDefinition {
    pub start_seconds: u64,
    pub duration_seconds: u16,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrbitalSensorDefinition {
    pub id: String,
    pub name: String,
    pub range_nm: f32,
    pub domains: Vec<String>,
    pub passes: Vec<SatellitePassDefinition>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundTrackDefinition {
    pub heading: f32,
    pub speed_knots: u16,
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
    #[serde(default)]
    pub defenses: Vec<DefenseSystemDefinition>,
    #[serde(default)]
    pub ground_track: Option<GroundTrackDefinition>,
    #[serde(default)]
    pub orbital_sensor: Option<OrbitalSensorDefinition>,
    #[serde(default)]
    pub hardening_seconds: Option<u16>,
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
pub struct CoastlineDefinition {
    pub id: String,
    pub name: String,
    pub points: Vec<Position>,
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
    #[serde(default)]
    pub coastlines: Vec<CoastlineDefinition>,
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
            let mut sensor_ids = HashSet::new();
            for sensor in &unit.sensors {
                if sensor.id.trim().is_empty()
                    || sensor.name.trim().is_empty()
                    || !sensor_ids.insert(sensor.id.as_str())
                {
                    return invalid("Sensor ids and names must be non-empty and sensor ids unique");
                }
                if !matches!(sensor.kind.as_str(), "radar" | "sonar" | "esm")
                    || !sensor.range_nm.is_finite()
                    || sensor.range_nm <= 0.0
                    || sensor.range_nm > 1000.0
                    || sensor.domains.is_empty()
                    || sensor
                        .domains
                        .iter()
                        .any(|domain| !matches!(domain.as_str(), "surface" | "air" | "subsurface"))
                {
                    return invalid("Sensor kind, range, or domain is outside supported limits");
                }
            }
            let mut weapon_ids = HashSet::new();
            for weapon in &unit.weapons {
                if weapon.id.trim().is_empty()
                    || weapon.name.trim().is_empty()
                    || !weapon_ids.insert(weapon.id.as_str())
                {
                    return invalid(
                        "Weapon ids and names must be non-empty and ids unique per unit",
                    );
                }
                let is_effect = matches!(weapon.kind.as_str(), "cyber" | "asat");
                let domains_valid = if weapon.kind == "asat" {
                    weapon.domains.len() == 1 && weapon.domains[0] == "space"
                } else {
                    !weapon.domains.is_empty()
                        && weapon.domains.iter().all(|domain| {
                            matches!(domain.as_str(), "surface" | "air" | "subsurface")
                        })
                };
                if !matches!(
                    weapon.kind.as_str(),
                    "missile" | "torpedo" | "gun" | "cyber" | "asat"
                ) || !weapon.range_nm.is_finite()
                    || weapon.range_nm <= 0.0
                    || weapon.range_nm > 1000.0
                    || !domains_valid
                    || weapon.ammunition == 0
                    || weapon.hit_chance_percent == 0
                    || weapon.hit_chance_percent > 100
                    || weapon.speed_knots == 0
                    || weapon.speed_knots > 5000
                    || weapon.damage_percent > 100
                    || (is_effect
                        && (weapon.damage_percent != 0
                            || !(60..=7200).contains(&weapon.effect_duration_seconds)))
                    || (!is_effect
                        && (weapon.damage_percent == 0 || weapon.effect_duration_seconds != 0))
                {
                    return invalid("Weapon type, range, ammunition, accuracy, speed, damage, effect duration, or domain is outside supported limits");
                }
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
            if !matches!(contact.domain.as_str(), "surface" | "air" | "subsurface" | "space") {
                return invalid("Contact domain must be surface, air, subsurface, or space");
            }
            if !valid_position(&contact.position) {
                return invalid("Contact position must be finite and within 50 map units");
            }
            let mut defense_ids = HashSet::new();
            for defense in &contact.defenses {
                if defense.id.trim().is_empty()
                    || defense.name.trim().is_empty()
                    || !defense_ids.insert(defense.id.as_str())
                    || defense.charges == 0
                    || defense.charges > 20
                    || defense.intercept_chance_percent == 0
                    || defense.intercept_chance_percent > 100
                {
                    return invalid("Defensive system identity, charges, or interception chance is outside supported limits");
                }
            }

            if contact.domain == "space" {
                if let Some(ground_track) = &contact.ground_track {
                    if !(0.0..360.0).contains(&ground_track.heading)
                        || ground_track.speed_knots == 0
                        || ground_track.speed_knots > 600
                    {
                        return invalid("Ground-track heading or drift speed is outside supported limits");
                    }
                }
                if contact.hardening_seconds.unwrap_or(600) > 3600 {
                    return invalid("Hardening cooldown must not exceed 3600 seconds");
                }
                if let Some(sensor) = &contact.orbital_sensor {
                    if sensor.id.trim().is_empty() || sensor.name.trim().is_empty() {
                        return invalid("Orbital sensor ids and names must not be empty");
                    }
                    if !sensor.range_nm.is_finite()
                        || sensor.range_nm <= 0.0
                        || sensor.range_nm > 1000.0
                        || sensor.domains.is_empty()
                        || sensor
                            .domains
                            .iter()
                            .any(|domain| !matches!(domain.as_str(), "surface" | "air"))
                    {
                        return invalid("Orbital sensor range or domain is outside supported limits");
                    }
                    if sensor.passes.is_empty() || sensor.passes.len() > 8 {
                        return invalid("A satellite needs between 1 and 8 pass windows");
                    }
                    let mut previous_end = 0_u64;
                    for (index, pass) in sensor.passes.iter().enumerate() {
                        let end =
                            pass.start_seconds.saturating_add(u64::from(pass.duration_seconds));
                        if pass.duration_seconds < 60
                            || pass.duration_seconds > 3600
                            || end > 86_400
                            || (index > 0 && pass.start_seconds < previous_end)
                        {
                            return invalid("Satellite pass windows must be 60 to 3600 seconds, sorted, non-overlapping, and within 24 hours");
                        }
                        previous_end = end;
                    }
                }
            } else if contact.ground_track.is_some()
                || contact.orbital_sensor.is_some()
                || contact.hardening_seconds.is_some()
            {
                return invalid("Ground-track motion, orbital sensors, and hardening apply only to space contacts");
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

        let mut coastline_ids = HashSet::new();
        for coastline in &self.coastlines {
            if coastline.id.trim().is_empty()
                || coastline.name.trim().is_empty()
                || !coastline_ids.insert(coastline.id.as_str())
            {
                return invalid("Coastline ids and names must be non-empty and unique");
            }
            if coastline.points.len() < 3
                || coastline.points.len() > 64
                || coastline.points.iter().any(|point| !valid_position(point))
            {
                return invalid(
                    "Coastline polygons need between 3 and 64 finite points within 50 map units",
                );
            }
        }
        if self.coastlines.len() > 12 {
            return invalid("A scenario cannot contain more than 12 coastlines");
        }
        let starts_on_land = self
            .units
            .iter()
            .map(|unit| &unit.position)
            .chain(self.contacts.iter().map(|contact| &contact.position))
            .any(|position| {
                self.coastlines
                    .iter()
                    .any(|coastline| point_in_polygon(position, &coastline.points))
            });
        if starts_on_land {
            return invalid("Units and contacts must not start on land");
        }
        Ok(())
    }
}

fn point_in_polygon(position: &Position, polygon: &[Position]) -> bool {
    let mut inside = false;
    for (index, vertex) in polygon.iter().enumerate() {
        let next = &polygon[(index + 1) % polygon.len()];
        if (vertex.z > position.z) != (next.z > position.z)
            && position.x
                < (next.x - vertex.x) * (position.z - vertex.z) / (next.z - vertex.z) + vertex.x
        {
            inside = !inside;
        }
    }
    inside
}

fn valid_position(position: &Position) -> bool {
    position.x.is_finite()
        && position.z.is_finite()
        && position.x.abs() <= 50.0
        && position.z.abs() <= 50.0
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignConsequenceDefinition {
    pub id: String,
    pub min_escalation: u16,
    pub title: String,
    pub description: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignDefinition {
    pub schema_version: u16,
    pub campaign_id: String,
    pub title: String,
    pub description: String,
    pub scenarios: Vec<String>,
    #[serde(default)]
    pub consequences: Vec<CampaignConsequenceDefinition>,
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
        if self.consequences.len() > 8 {
            return Err(ScenarioError(
                "A campaign cannot contain more than 8 consequences".into(),
            ));
        }
        let mut consequence_ids = HashSet::new();
        for consequence in &self.consequences {
            if consequence.id.trim().is_empty()
                || consequence.title.trim().is_empty()
                || consequence.description.trim().is_empty()
                || !consequence_ids.insert(consequence.id.as_str())
            {
                return Err(ScenarioError(
                    "Campaign consequence ids, titles, and descriptions must be non-empty and unique".into(),
                ));
            }
            if consequence.min_escalation == 0 {
                return Err(ScenarioError(
                    "Campaign consequence thresholds must be at least escalation level 1".into(),
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{CampaignDefinition, ScenarioDefinition};
    use serde_json::json;

    const SCENARIO: &str = include_str!("../../../scenarios/north-atlantic-intercept.json");
    const CAMPAIGN: &str = include_str!("../../../campaigns/northern-watch.json");

    #[test]
    fn parses_starter_scenario() {
        let scenario = ScenarioDefinition::parse(SCENARIO).expect("starter scenario is valid");
        assert_eq!(scenario.id, "north-atlantic-intercept");
        assert_eq!(scenario.units.len(), 3);
        assert_eq!(scenario.contacts.len(), 4);
        assert_eq!(scenario.contacts[3].domain, "space");
        assert_eq!(scenario.coastlines.len(), 3);
    }

    #[test]
    fn coastlines_are_optional_and_polygons_stay_valid() {
        let mut without_land: serde_json::Value = serde_json::from_str(SCENARIO).unwrap();
        without_land.as_object_mut().unwrap().remove("coastlines");
        let parsed = ScenarioDefinition::parse(&without_land.to_string())
            .expect("scenarios authored before coastlines remain valid");
        assert!(parsed.coastlines.is_empty());

        let mut degenerate: serde_json::Value = serde_json::from_str(SCENARIO).unwrap();
        degenerate["coastlines"][0]["points"] = json!([{ "x": 1.0, "z": 1.0 }]);
        let error = ScenarioDefinition::parse(&degenerate.to_string()).unwrap_err();
        assert!(error
            .to_string()
            .contains("Coastline polygons need between 3 and 64"));

        let mut duplicate: serde_json::Value = serde_json::from_str(SCENARIO).unwrap();
        duplicate["coastlines"][1]["id"] = json!("norwegian-coast");
        let error = ScenarioDefinition::parse(&duplicate.to_string()).unwrap_err();
        assert!(error
            .to_string()
            .contains("Coastline ids and names must be non-empty and unique"));
    }

    #[test]
    fn rejects_units_and_contacts_starting_on_land() {
        let mut value: serde_json::Value = serde_json::from_str(SCENARIO).unwrap();
        value["units"][0]["position"] = json!({ "x": 30.0, "z": 0.0 });
        let error = ScenarioDefinition::parse(&value.to_string()).unwrap_err();
        assert!(error
            .to_string()
            .contains("Units and contacts must not start on land"));

        value["units"][0]["position"] = json!({ "x": -5.0, "z": 2.0 });
        value["contacts"][0]["position"] = json!({ "x": 11.0, "z": -10.0 });
        let error = ScenarioDefinition::parse(&value.to_string()).unwrap_err();
        assert!(error
            .to_string()
            .contains("Units and contacts must not start on land"));

        value["contacts"][0]["position"] = json!({ "x": 5.0, "z": -1.5 });
        ScenarioDefinition::parse(&value.to_string()).expect("sea starts are valid");
    }

    fn space_contact() -> serde_json::Value {
        json!({
            "id": "sat-1",
            "label": "COSMOS 2547",
            "classification": "Unknown",
            "confidence": 60,
            "bearing": 90,
            "rangeNm": 300.0,
            "domain": "space",
            "position": { "x": -30.0, "z": -30.0 },
            "identified": false,
            "groundTrack": { "heading": 85.0, "speedKnots": 240 },
            "hardeningSeconds": 900,
            "orbitalSensor": {
                "id": "sar-1",
                "name": "SAR swath",
                "rangeNm": 600.0,
                "domains": ["surface", "air"],
                "passes": [
                    { "startSeconds": 3600, "durationSeconds": 900 },
                    { "startSeconds": 7200, "durationSeconds": 900 }
                ]
            }
        })
    }

    #[test]
    fn space_contacts_validate_ground_tracks_and_pass_windows() {
        let mut value: serde_json::Value = serde_json::from_str(SCENARIO).unwrap();
        value["contacts"][0] = space_contact();
        let parsed =
            ScenarioDefinition::parse(&value.to_string()).expect("satellite contact is valid");
        assert_eq!(parsed.contacts[0].domain, "space");

        value["contacts"][0]["groundTrack"]["speedKnots"] = json!(700);
        let error = ScenarioDefinition::parse(&value.to_string()).unwrap_err();
        assert!(error
            .to_string()
            .contains("Ground-track heading or drift speed"));

        value["contacts"][0]["groundTrack"]["speedKnots"] = json!(240);
        value["contacts"][0]["orbitalSensor"]["passes"][1]["startSeconds"] = json!(4000);
        let error = ScenarioDefinition::parse(&value.to_string()).unwrap_err();
        assert!(error
            .to_string()
            .contains("pass windows must be 60 to 3600"));

        value["contacts"][0]["orbitalSensor"]["passes"][1]["startSeconds"] = json!(7200);
        value["contacts"][0]["orbitalSensor"]["domains"] = json!(["surface", "subsurface"]);
        let error = ScenarioDefinition::parse(&value.to_string()).unwrap_err();
        assert!(error
            .to_string()
            .contains("Orbital sensor range or domain"));

        value["contacts"][0]["orbitalSensor"]["domains"] = json!(["surface"]);
        value["contacts"][1]["groundTrack"] = json!({ "heading": 90.0, "speedKnots": 240 });
        let error = ScenarioDefinition::parse(&value.to_string()).unwrap_err();
        assert!(error
            .to_string()
            .contains("apply only to space contacts"));
    }

    #[test]
    fn asat_weapons_are_space_only_effect_weapons() {
        ScenarioDefinition::parse(SCENARIO).expect("starter asat loadout is valid");

        let mut value: serde_json::Value = serde_json::from_str(SCENARIO).unwrap();
        value["units"][0]["weapons"][3]["domains"] = json!(["space", "air"]);
        let error = ScenarioDefinition::parse(&value.to_string()).unwrap_err();
        assert!(error
            .to_string()
            .contains("Weapon type, range, ammunition"));

        value["units"][0]["weapons"][3]["domains"] = json!(["space"]);
        value["units"][0]["weapons"][3]["damagePercent"] = json!(50);
        let error = ScenarioDefinition::parse(&value.to_string()).unwrap_err();
        assert!(error
            .to_string()
            .contains("Weapon type, range, ammunition"));

        value["units"][0]["weapons"][3]["damagePercent"] = json!(0);
        value["units"][0]["weapons"][3]["effectDurationSeconds"] = json!(30);
        let error = ScenarioDefinition::parse(&value.to_string()).unwrap_err();
        assert!(error
            .to_string()
            .contains("Weapon type, range, ammunition"));

        value["units"][0]["weapons"][3]["effectDurationSeconds"] = json!(1800);
        value["units"][0]["weapons"][2]["domains"] = json!(["surface", "space"]);
        let error = ScenarioDefinition::parse(&value.to_string()).unwrap_err();
        assert!(error
            .to_string()
            .contains("Weapon type, range, ammunition"));
    }

    #[test]
    fn campaign_consequences_are_optional_and_validated() {
        let campaign =
            CampaignDefinition::parse(CAMPAIGN).expect("starter campaign consequences are valid");
        assert_eq!(campaign.consequences.len(), 2);

        let mut without: serde_json::Value = serde_json::from_str(CAMPAIGN).unwrap();
        without.as_object_mut().unwrap().remove("consequences");
        let campaign = CampaignDefinition::parse(&without.to_string())
            .expect("campaigns authored before consequences remain valid");
        assert!(campaign.consequences.is_empty());

        let mut value: serde_json::Value = serde_json::from_str(CAMPAIGN).unwrap();
        value["consequences"][0]["minEscalation"] = json!(0);
        let error = CampaignDefinition::parse(&value.to_string()).unwrap_err();
        assert!(error
            .to_string()
            .contains("thresholds must be at least escalation level 1"));

        value["consequences"][0]["minEscalation"] = json!(2);
        let duplicate = value["consequences"][0].clone();
        value["consequences"].as_array_mut().unwrap().push(duplicate);
        let error = CampaignDefinition::parse(&value.to_string()).unwrap_err();
        assert!(error
            .to_string()
            .contains("consequence ids, titles, and descriptions"));
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
    fn rejects_sensor_profiles_outside_supported_limits() {
        let invalid_range = SCENARIO.replace("\"rangeNm\": 120", "\"rangeNm\": 0");
        let error = ScenarioDefinition::parse(&invalid_range).unwrap_err();
        assert!(error.to_string().contains("Sensor kind, range, or domain"));
    }

    #[test]
    fn parses_weapon_loadouts_and_rejects_invalid_accuracy() {
        let scenario = ScenarioDefinition::parse(SCENARIO).expect("weapon loadouts are valid");
        assert_eq!(scenario.units[0].weapons.len(), 4);
        assert_eq!(scenario.units[0].weapons[2].kind, "cyber");
        assert_eq!(scenario.units[0].weapons[3].kind, "asat");

        let legacy_scenario = SCENARIO.replace(", \"damagePercent\": 80", "");
        let legacy =
            ScenarioDefinition::parse(&legacy_scenario).expect("older weapon data remains valid");
        assert_eq!(legacy.units[0].weapons[0].damage_percent, 100);

        let invalid_accuracy =
            SCENARIO.replace("\"hitChancePercent\": 78", "\"hitChancePercent\": 101");
        let error = ScenarioDefinition::parse(&invalid_accuracy).unwrap_err();
        assert!(error
            .to_string()
            .to_string()
            .contains("Weapon type, range, ammunition, accuracy, speed, damage"));
    }

    #[test]
    fn cyber_weapons_require_a_bounded_disruption_duration_and_no_hull_damage() {
        let cyber = SCENARIO
            .replacen("\"kind\": \"missile\"", "\"kind\": \"cyber\"", 1)
            .replace("\"damagePercent\": 80", "\"damagePercent\": 0, \"effectDurationSeconds\": 900");
        let parsed = ScenarioDefinition::parse(&cyber).expect("cyber weapon is valid");
        assert_eq!(parsed.units[0].weapons[0].kind, "cyber");
        assert_eq!(parsed.units[0].weapons[0].effect_duration_seconds, 900);

        let invalid_duration = cyber.replace("\"effectDurationSeconds\": 900", "\"effectDurationSeconds\": 0");
        assert!(ScenarioDefinition::parse(&invalid_duration).is_err());
        let invalid_damage = cyber.replace("\"damagePercent\": 0", "\"damagePercent\": 1");
        assert!(ScenarioDefinition::parse(&invalid_damage).is_err());
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
