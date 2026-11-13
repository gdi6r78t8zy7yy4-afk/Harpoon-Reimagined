use harpoon_scenario_format::{ScenarioDefinition, SensorDefinition, WeaponDefinition};
use serde::{Deserialize, Serialize};

const STARTER_SCENARIO: &str = include_str!("../../../scenarios/north-atlantic-intercept.json");

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Unit {
    pub id: String,
    pub name: String,
    pub class_name: String,
    pub side: String,
    pub status: String,
    pub x: f32,
    pub z: f32,
    pub heading: f32,
    pub speed_knots: u8,
    pub health: u8,
    pub sensors: Vec<SensorDefinition>,
    #[serde(default)]
    pub weapons: Vec<WeaponDefinition>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    pub id: String,
    pub label: String,
    pub classification: String,
    pub confidence: u8,
    pub bearing: u16,
    pub range_nm: f32,
    pub domain: String,
    pub x: f32,
    pub z: f32,
    pub identified: bool,
    #[serde(default)]
    pub last_reported_seconds: u64,
    #[serde(default)]
    pub confidence_at_last_report: u8,
    #[serde(default)]
    pub range_uncertainty_nm: f32,
    #[serde(default)]
    pub bearing_uncertainty_deg: f32,
    #[serde(default)]
    pub range_uncertainty_at_last_report: f32,
    #[serde(default)]
    pub bearing_uncertainty_at_last_report: f32,
    #[serde(default)]
    pub report_count: u32,
    #[serde(default)]
    pub report_history: Vec<TrackReport>,
    #[serde(default)]
    pub correlated_contact_ids: Vec<String>,
    #[serde(default)]
    pub destroyed: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackReport {
    pub elapsed_seconds: u64,
    pub unit_id: String,
    pub unit_name: String,
    pub sensor_id: String,
    pub sensor_name: String,
    pub sensor_kind: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Engagement {
    pub id: u64,
    pub elapsed_seconds: u64,
    pub impact_at_seconds: u64,
    pub unit_id: String,
    pub unit_name: String,
    pub weapon_id: String,
    pub weapon_name: String,
    pub contact_id: String,
    pub contact_label: String,
    pub hit_chance_percent: u8,
    pub status: String,
    pub hit: bool,
}

const MAX_TRACK_REPORT_HISTORY: usize = 20;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectiveProgress {
    pub id: String,
    pub title: String,
    pub description: String,
    pub progress: u8,
    pub required: u8,
    pub completed: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationEvent {
    pub id: u64,
    pub elapsed_seconds: u64,
    pub message: String,
    pub source: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationState {
    pub scenario_id: String,
    pub mission_title: String,
    pub theatre: String,
    pub tick: u64,
    pub elapsed_seconds: u64,
    pub running: bool,
    pub mission_complete: bool,
    pub units: Vec<Unit>,
    pub contacts: Vec<Contact>,
    #[serde(default)]
    pub engagements: Vec<Engagement>,
    pub objectives: Vec<ObjectiveProgress>,
    pub events: Vec<SimulationEvent>,
}

fn initial_range_uncertainty(confidence: u8) -> f32 {
    1.0 + f32::from(100_u8.saturating_sub(confidence)) * 0.12
}

fn initial_bearing_uncertainty(confidence: u8) -> f32 {
    2.0 + f32::from(100_u8.saturating_sub(confidence)) * 0.5
}

fn fuse_bearing(prior: u16, prior_sigma: f32, observed: u16, observed_sigma: f32) -> (u16, f32) {
    let prior_sigma = prior_sigma.clamp(0.1, 180.0);
    let observed_sigma = observed_sigma.clamp(0.1, 180.0);
    let prior_weight = 1.0 / prior_sigma.powi(2);
    let observed_weight = 1.0 / observed_sigma.powi(2);
    let prior_radians = f32::from(prior).to_radians();
    let observed_radians = f32::from(observed).to_radians();
    let x = prior_radians.cos() * prior_weight + observed_radians.cos() * observed_weight;
    let y = prior_radians.sin() * prior_weight + observed_radians.sin() * observed_weight;
    let fused = y.atan2(x).to_degrees().rem_euclid(360.0).round() as u16 % 360;
    let sigma = (1.0 / (prior_weight + observed_weight)).sqrt();
    (fused, sigma)
}

impl Default for SimulationState {
    fn default() -> Self {
        Self::from_scenario_json(STARTER_SCENARIO)
            .expect("the built-in North Atlantic scenario must be valid")
    }
}

impl SimulationState {
    pub fn from_scenario_json(json: &str) -> Result<Self, String> {
        let scenario = ScenarioDefinition::parse(json).map_err(|error| error.to_string())?;
        let start_time_seconds = scenario.start_time_seconds;
        let units = scenario
            .units
            .into_iter()
            .map(|unit| Unit {
                id: unit.id,
                name: unit.name,
                class_name: unit.class_name,
                side: unit.side,
                status: unit.status,
                x: unit.position.x,
                z: unit.position.z,
                heading: unit.heading,
                speed_knots: unit.speed_knots,
                health: unit.health,
                sensors: unit.sensors,
                weapons: unit.weapons,
            })
            .collect();
        let contacts = scenario
            .contacts
            .into_iter()
            .map(|contact| Contact {
                id: contact.id,
                label: contact.label,
                classification: contact.classification,
                confidence: contact.confidence,
                bearing: contact.bearing,
                range_nm: contact.range_nm,
                domain: contact.domain,
                x: contact.position.x,
                z: contact.position.z,
                identified: contact.identified,
                last_reported_seconds: start_time_seconds,
                confidence_at_last_report: contact.confidence,
                range_uncertainty_nm: initial_range_uncertainty(contact.confidence),
                bearing_uncertainty_deg: initial_bearing_uncertainty(contact.confidence),
                range_uncertainty_at_last_report: initial_range_uncertainty(contact.confidence),
                bearing_uncertainty_at_last_report: initial_bearing_uncertainty(contact.confidence),
                report_count: 1,
                report_history: Vec::new(),
                correlated_contact_ids: Vec::new(),
                destroyed: false,
            })
            .collect();
        let objectives = scenario
            .objectives
            .into_iter()
            .map(|objective| ObjectiveProgress {
                id: objective.id,
                title: objective.title,
                description: objective.description,
                progress: 0,
                required: objective.required,
                completed: false,
            })
            .collect();

        Ok(Self {
            scenario_id: scenario.id,
            mission_title: scenario.title,
            theatre: scenario.theatre,
            tick: 0,
            elapsed_seconds: start_time_seconds,
            running: false,
            mission_complete: false,
            units,
            contacts,
            engagements: Vec::new(),
            objectives,
            events: vec![
                SimulationEvent {
                    id: 1,
                    elapsed_seconds: start_time_seconds,
                    message: "Unidentified surface contact detected".into(),
                    source: "USS ARLEIGH BURKE".into(),
                },
                SimulationEvent {
                    id: 2,
                    elapsed_seconds: start_time_seconds.saturating_sub(142),
                    message: "Transient acoustic signal reported".into(),
                    source: "USS SEAWOLF".into(),
                },
            ],
        })
    }

    pub fn normalize_legacy_track_metadata(&mut self) {
        for contact in &mut self.contacts {
            if contact.last_reported_seconds == 0 {
                contact.last_reported_seconds = self.elapsed_seconds;
                contact.confidence_at_last_report = contact.confidence;
            }
            if contact.report_count == 0 {
                contact.range_uncertainty_nm = initial_range_uncertainty(contact.confidence);
                contact.bearing_uncertainty_deg = initial_bearing_uncertainty(contact.confidence);
                contact.range_uncertainty_at_last_report = contact.range_uncertainty_nm;
                contact.bearing_uncertainty_at_last_report = contact.bearing_uncertainty_deg;
                contact.report_count = 1;
            }
        }
    }

    pub fn advance(&mut self, seconds: u64) {
        if self.mission_complete {
            return;
        }
        let previous_elapsed_seconds = self.elapsed_seconds;
        let seconds = seconds.clamp(1, 3600);
        self.tick += 1;
        self.elapsed_seconds += seconds;
        for unit in &mut self.units {
            if unit.speed_knots > 0 {
                let radians = unit.heading.to_radians();
                let distance = f32::from(unit.speed_knots) * seconds as f32 / 3600.0;
                unit.x += radians.sin() * distance;
                unit.z -= radians.cos() * distance;
            }
        }
        let mut stale_tracks = Vec::new();
        for contact in &mut self.contacts {
            if contact.identified {
                continue;
            }
            // Saves created before track aging was added have no report timestamps.
            if contact.last_reported_seconds == 0 {
                contact.last_reported_seconds = previous_elapsed_seconds;
                contact.confidence_at_last_report = contact.confidence;
            }
            let previous_age =
                previous_elapsed_seconds.saturating_sub(contact.last_reported_seconds);
            let track_age = self
                .elapsed_seconds
                .saturating_sub(contact.last_reported_seconds);
            let decay_steps = track_age / 600;
            contact.confidence = contact
                .confidence_at_last_report
                .saturating_sub(decay_steps.saturating_mul(5).min(u64::from(u8::MAX)) as u8);
            contact.range_uncertainty_nm =
                (contact.range_uncertainty_at_last_report + decay_steps as f32 * 0.2).min(100.0);
            contact.bearing_uncertainty_deg =
                (contact.bearing_uncertainty_at_last_report + decay_steps as f32).min(180.0);
            if previous_age < 1800 && track_age >= 1800 {
                stale_tracks.push(contact.label.clone());
            }
        }
        for label in stale_tracks {
            self.push_event(format!("Track is stale: {label}"), "SENSORS".into());
        }
        let mut impact_events = Vec::new();
        for engagement in &mut self.engagements {
            if engagement.status != "in_flight"
                || engagement.impact_at_seconds > self.elapsed_seconds
            {
                continue;
            }
            if engagement.hit {
                if let Some(contact) = self
                    .contacts
                    .iter_mut()
                    .find(|contact| contact.id == engagement.contact_id)
                {
                    if contact.destroyed {
                        engagement.hit = false;
                        engagement.status = "aborted".into();
                    } else {
                        contact.destroyed = true;
                        engagement.status = "hit".into();
                    }
                } else {
                    engagement.hit = false;
                    engagement.status = "aborted".into();
                }
            } else {
                engagement.status = "miss".into();
            }
            impact_events.push((
                engagement.weapon_name.clone(),
                engagement.contact_label.clone(),
                engagement.status.clone(),
            ));
        }
        for (weapon_name, contact_label, status) in impact_events {
            let message = match status.as_str() {
                "hit" => format!("{weapon_name} hit {contact_label}"),
                "miss" => format!("{weapon_name} missed {contact_label}"),
                _ => format!("{weapon_name} impact aborted; {contact_label} is no longer active"),
            };
            self.push_event(message, "WEAPONS CONTROL".into());
        }
        self.maybe_complete_mission();
    }

    pub fn issue_course(&mut self, unit_id: &str, heading: u16) -> Result<(), String> {
        if self.mission_complete {
            return Err("The scenario is complete; reload it to issue new orders".into());
        }
        if heading > 359 {
            return Err("Course must be between 000° and 359°".into());
        }
        let unit = self
            .units
            .iter_mut()
            .find(|unit| unit.id == unit_id)
            .ok_or_else(|| "Unit not found".to_string())?;
        unit.heading = f32::from(heading);
        let name = unit.name.clone();
        self.push_event(format!("New course set to {heading:03}°"), name);
        Ok(())
    }

    pub fn issue_speed(&mut self, unit_id: &str, speed_knots: u8) -> Result<(), String> {
        if self.mission_complete {
            return Err("The scenario is complete; reload it to issue new orders".into());
        }
        if speed_knots > 50 {
            return Err("Speed must be between 0 and 50 knots".into());
        }
        let unit = self
            .units
            .iter_mut()
            .find(|unit| unit.id == unit_id)
            .ok_or_else(|| "Unit not found".to_string())?;
        unit.speed_knots = speed_knots;
        let name = unit.name.clone();
        self.push_event(format!("Speed changed to {speed_knots} kt"), name);
        Ok(())
    }

    pub fn sensor_sweep(&mut self, unit_id: &str, sensor_id: &str) -> Result<usize, String> {
        if self.mission_complete {
            return Err("The scenario is complete; reload it to sweep sensors".into());
        }
        let unit = self
            .units
            .iter()
            .find(|unit| unit.id == unit_id)
            .ok_or_else(|| "Unit not found".to_string())?;
        let unit_name = unit.name.clone();
        let unit_x = unit.x;
        let unit_z = unit.z;
        let sensor = unit
            .sensors
            .iter()
            .find(|sensor| sensor.id == sensor_id)
            .ok_or_else(|| "Sensor not found on this unit".to_string())?;
        let sensor_name = sensor.name.clone();
        let sensor_kind = sensor.kind.clone();
        let sensor_range = sensor.range_nm;
        let sensor_domains = sensor.domains.clone();
        let mut reports = 0;

        for contact in &mut self.contacts {
            if contact.identified {
                continue;
            }
            let dx = contact.x - unit_x;
            let dz = contact.z - unit_z;
            let distance_nm = dx.hypot(dz) * 5.0;
            if !sensor_domains
                .iter()
                .any(|domain| domain == &contact.domain)
                || distance_nm > sensor_range
            {
                continue;
            }
            let confidence_gain = match sensor_kind.as_str() {
                "radar" => 10,
                "sonar" => 7,
                "esm" => 5,
                _ => 0,
            };
            let observed_bearing =
                (dx.atan2(-dz).to_degrees().rem_euclid(360.0).round() as u16) % 360;
            let (bearing_sigma, range_sigma): (f32, Option<f32>) = match sensor_kind.as_str() {
                "radar" => (2.0, Some(0.5)),
                "sonar" => (6.0, None),
                "esm" => (10.0, None),
                _ => (30.0, None),
            };
            let (fused_bearing, fused_bearing_sigma) = fuse_bearing(
                contact.bearing,
                contact.bearing_uncertainty_deg,
                observed_bearing,
                bearing_sigma,
            );
            contact.bearing = fused_bearing;
            contact.bearing_uncertainty_deg = fused_bearing_sigma;
            if let Some(observed_range_sigma) = range_sigma {
                let prior_variance = contact.range_uncertainty_nm.max(0.1).powi(2);
                let observed_variance = observed_range_sigma.powi(2);
                let prior_weight = 1.0 / prior_variance;
                let observed_weight = 1.0 / observed_variance;
                contact.range_nm = (contact.range_nm * prior_weight
                    + distance_nm * observed_weight)
                    / (prior_weight + observed_weight);
                contact.range_uncertainty_nm = (1.0 / (prior_weight + observed_weight)).sqrt();
            }
            contact.confidence = contact.confidence.saturating_add(confidence_gain).min(99);
            contact.last_reported_seconds = self.elapsed_seconds;
            contact.confidence_at_last_report = contact.confidence;
            contact.range_uncertainty_at_last_report = contact.range_uncertainty_nm;
            contact.bearing_uncertainty_at_last_report = contact.bearing_uncertainty_deg;
            contact.report_count = contact.report_count.saturating_add(1);
            contact.report_history.push(TrackReport {
                elapsed_seconds: self.elapsed_seconds,
                unit_id: unit_id.to_string(),
                unit_name: unit_name.clone(),
                sensor_id: sensor_id.to_string(),
                sensor_name: sensor_name.clone(),
                sensor_kind: sensor_kind.clone(),
            });
            if contact.report_history.len() > MAX_TRACK_REPORT_HISTORY {
                contact.report_history.remove(0);
            }
            reports += 1;
        }

        let message = if reports == 0 {
            "Sensor sweep complete: no contacts updated".to_string()
        } else {
            format!("{sensor_name} sweep complete: {reports} contact report(s) updated")
        };
        self.push_event(message, unit_name);
        Ok(reports)
    }

    pub fn correlate_contacts(
        &mut self,
        contact_id: &str,
        other_contact_id: &str,
        correlated: bool,
    ) -> Result<(), String> {
        if contact_id == other_contact_id {
            return Err("A contact cannot be correlated with itself".into());
        }
        let contact_index = self
            .contacts
            .iter()
            .position(|contact| contact.id == contact_id)
            .ok_or_else(|| "Contact not found".to_string())?;
        let other_index = self
            .contacts
            .iter()
            .position(|contact| contact.id == other_contact_id)
            .ok_or_else(|| "Contact not found".to_string())?;
        let contact_label = self.contacts[contact_index].label.clone();
        let other_label = self.contacts[other_index].label.clone();

        for (index, peer_id) in [(contact_index, other_contact_id), (other_index, contact_id)] {
            let peers = &mut self.contacts[index].correlated_contact_ids;
            if correlated {
                if !peers.iter().any(|id| id == peer_id) {
                    peers.push(peer_id.to_string());
                    peers.sort();
                }
            } else {
                peers.retain(|id| id != peer_id);
            }
        }

        let message = if correlated {
            format!("Tracks correlated: {contact_label} / {other_label}")
        } else {
            format!("Tracks separated: {contact_label} / {other_label}")
        };
        self.push_event(message, "COMMAND".into());
        Ok(())
    }

    pub fn fire_weapon(
        &mut self,
        unit_id: &str,
        weapon_id: &str,
        contact_id: &str,
    ) -> Result<(), String> {
        if self.mission_complete {
            return Err("The scenario is complete; reload it before engaging".into());
        }
        let unit_index = self
            .units
            .iter()
            .position(|unit| unit.id == unit_id)
            .ok_or_else(|| "Unit not found".to_string())?;
        if self.units[unit_index].side != "friendly" {
            return Err("Only friendly units can engage targets".into());
        }
        let weapon_index = self.units[unit_index]
            .weapons
            .iter()
            .position(|weapon| weapon.id == weapon_id)
            .ok_or_else(|| "Weapon not found on this unit".to_string())?;
        let weapon = self.units[unit_index].weapons[weapon_index].clone();
        let unit_name = self.units[unit_index].name.clone();
        if weapon.ammunition == 0 {
            return Err("No ammunition remains for this weapon".into());
        }

        let contact_index = self
            .contacts
            .iter()
            .position(|contact| contact.id == contact_id)
            .ok_or_else(|| "Contact not found".to_string())?;
        let contact = &self.contacts[contact_index];
        if contact.destroyed {
            return Err("This contact has already been destroyed".into());
        }
        if !contact.identified || contact.classification != "Hostile platform" {
            return Err("Only identified hostile contacts can be engaged".into());
        }
        if !weapon
            .domains
            .iter()
            .any(|domain| domain == &contact.domain)
        {
            return Err("This weapon cannot engage the contact's domain".into());
        }
        if contact.range_nm > weapon.range_nm {
            return Err("Contact is outside this weapon's estimated range".into());
        }

        let contact_label = contact.label.clone();
        let confidence_adjustment = (i16::from(contact.confidence) - 50) / 5;
        let range_penalty = ((contact.range_nm / weapon.range_nm) * 15.0).round() as i16;
        let hit_chance_percent = (i16::from(weapon.hit_chance_percent) + confidence_adjustment
            - range_penalty)
            .clamp(5, 95) as u8;
        let engagement_id = self.engagements.last().map_or(1, |item| item.id + 1);
        let flight_seconds = (contact.range_nm / f32::from(weapon.speed_knots) * 3600.0)
            .ceil()
            .max(1.0) as u64;
        let id_salt = unit_id
            .bytes()
            .chain(weapon_id.bytes())
            .chain(contact_id.bytes())
            .fold(0_u64, |sum, byte| sum.wrapping_add(u64::from(byte)));
        let roll = (self
            .elapsed_seconds
            .wrapping_add(self.tick.wrapping_mul(17))
            .wrapping_add(engagement_id.wrapping_mul(31))
            .wrapping_add(id_salt))
            % 100;
        let hit = roll < u64::from(hit_chance_percent);

        self.units[unit_index].weapons[weapon_index].ammunition -= 1;
        self.engagements.push(Engagement {
            id: engagement_id,
            elapsed_seconds: self.elapsed_seconds,
            impact_at_seconds: self.elapsed_seconds.saturating_add(flight_seconds),
            unit_id: unit_id.to_string(),
            unit_name: unit_name.clone(),
            weapon_id: weapon_id.to_string(),
            weapon_name: weapon.name.clone(),
            contact_id: contact_id.to_string(),
            contact_label: contact_label.clone(),
            hit_chance_percent,
            status: "in_flight".into(),
            hit,
        });
        if self.engagements.len() > 50 {
            self.engagements.remove(0);
        }
        self.push_event(
            format!(
                "{} launched at {contact_label} ({hit_chance_percent}% solution)",
                weapon.name
            ),
            unit_name,
        );
        Ok(())
    }

    pub fn classify_contact(&mut self, contact_id: &str, identity: &str) -> Result<(), String> {
        if self.mission_complete {
            return Err("The scenario is complete; reload it to classify more contacts".into());
        }
        if !matches!(identity, "hostile" | "neutral") {
            return Err("Contact identity must be hostile or neutral".into());
        }
        let contact = self
            .contacts
            .iter_mut()
            .find(|contact| contact.id == contact_id)
            .ok_or_else(|| "Contact not found".to_string())?;
        if contact.identified {
            return Err("This contact has already been classified".into());
        }
        let label = if identity == "hostile" {
            format!("HOSTILE {}", contact.domain.to_uppercase())
        } else {
            format!("NEUTRAL {}", contact.domain.to_uppercase())
        };
        contact.label = label;
        contact.classification = if identity == "hostile" {
            "Hostile platform".into()
        } else {
            "Neutral platform".into()
        };
        contact.confidence = 100;
        contact.last_reported_seconds = self.elapsed_seconds;
        contact.confidence_at_last_report = 100;
        contact.range_uncertainty_at_last_report = contact.range_uncertainty_nm;
        contact.bearing_uncertainty_at_last_report = contact.bearing_uncertainty_deg;
        contact.identified = true;
        let report = contact.label.clone();
        self.update_objectives();
        self.push_event(format!("Contact classified: {report}"), "COMMAND".into());
        self.maybe_complete_mission();
        Ok(())
    }

    fn maybe_complete_mission(&mut self) {
        let objectives_complete = self.objectives.iter().all(|objective| objective.completed);
        let weapons_resolved = self
            .engagements
            .iter()
            .all(|engagement| engagement.status != "in_flight");
        if objectives_complete && weapons_resolved && !self.mission_complete {
            self.mission_complete = true;
            self.running = false;
            self.push_event(
                "Mission complete: contact picture established".into(),
                "MISSION CONTROL".into(),
            );
        }
    }

    fn update_objectives(&mut self) {
        let classified = self
            .contacts
            .iter()
            .filter(|contact| contact.identified)
            .count() as u8;
        for objective in &mut self.objectives {
            objective.progress = classified.min(objective.required);
            objective.completed = objective.progress >= objective.required;
        }
    }

    fn push_event(&mut self, message: String, source: String) {
        let next_id = self.events.last().map_or(1, |event| event.id + 1);
        self.events.push(SimulationEvent {
            id: next_id,
            elapsed_seconds: self.elapsed_seconds,
            message,
            source,
        });
        if self.events.len() > 12 {
            self.events.remove(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SimulationState;

    #[test]
    fn movement_advances_time_and_moves_units_by_heading() {
        let mut state = SimulationState::default();
        let before = state.units[0].z;
        let start_time = state.elapsed_seconds;

        state.advance(60);

        assert_eq!(state.tick, 1);
        assert_eq!(state.elapsed_seconds, start_time + 60);
        assert!(state.units[0].z < before);
    }

    #[test]
    fn classifying_contacts_completes_objective_and_stops_simulation() {
        let mut state = SimulationState::default();
        state.running = true;

        state.classify_contact("c-101", "hostile").unwrap();
        assert!(!state.mission_complete);
        assert_eq!(state.objectives[0].progress, 1);

        state.classify_contact("c-102", "neutral").unwrap();
        assert!(state.mission_complete);
        assert!(!state.running);
        assert!(state.objectives[0].completed);
    }

    #[test]
    fn rejects_invalid_or_repeated_orders() {
        let mut state = SimulationState::default();

        assert!(state.issue_course("ddg-01", 360).is_err());
        assert!(state.issue_speed("ddg-01", 51).is_err());
        assert!(state.classify_contact("missing", "hostile").is_err());
        state.classify_contact("c-101", "hostile").unwrap();
        assert!(state.classify_contact("c-101", "hostile").is_err());
    }

    #[test]
    fn sensor_sweep_updates_only_contacts_within_range_and_sensor_domain() {
        let mut state = SimulationState::default();
        let original_confidence = state.contacts[0].confidence;
        let reports = state.sensor_sweep("ddg-01", "spyd-1").unwrap();

        assert_eq!(reports, 2);
        assert_eq!(state.contacts[0].confidence, original_confidence + 10);
        assert!(state.contacts[0].range_nm < 60.0);
        assert_eq!(state.contacts[2].confidence, 31);
        assert_eq!(state.contacts[0].report_history.len(), 1);
        assert_eq!(state.contacts[0].report_history[0].sensor_id, "spyd-1");
        assert!(state
            .events
            .last()
            .unwrap()
            .message
            .contains("2 contact report"));
    }

    #[test]
    fn sonar_updates_bearing_but_keeps_the_existing_range_estimate() {
        let mut state = SimulationState::default();
        let old_range = state.contacts[2].range_nm;
        let old_bearing_uncertainty = state.contacts[2].bearing_uncertainty_deg;
        let old_report_count = state.contacts[2].report_count;

        assert_eq!(state.sensor_sweep("ssn-01", "bqq-2").unwrap(), 1);
        assert_eq!(state.contacts[2].confidence, 38);
        assert_eq!(state.contacts[2].range_nm, old_range);
        assert!(state.contacts[2].bearing_uncertainty_deg < old_bearing_uncertainty);
        assert_eq!(state.contacts[2].report_count, old_report_count + 1);
        assert_eq!(
            state.sensor_sweep("ddg-01", "bqq-2").unwrap_err(),
            "Sensor not found on this unit"
        );
    }

    #[test]
    fn radar_reports_fuse_range_and_reduce_position_uncertainty() {
        let mut state = SimulationState::default();
        let old_range_uncertainty = state.contacts[0].range_uncertainty_nm;
        let old_bearing_uncertainty = state.contacts[0].bearing_uncertainty_deg;
        let old_report_count = state.contacts[0].report_count;

        state.sensor_sweep("ddg-01", "spyd-1").unwrap();

        assert!(state.contacts[0].range_uncertainty_nm < old_range_uncertainty);
        assert!(state.contacts[0].bearing_uncertainty_deg < old_bearing_uncertainty);
        assert_eq!(state.contacts[0].report_count, old_report_count + 1);
    }

    #[test]
    fn bearing_fusion_handles_reports_across_zero_degrees() {
        let (bearing, uncertainty) = super::fuse_bearing(359, 10.0, 1, 10.0);

        assert!(bearing <= 1 || bearing >= 359);
        assert!(uncertainty < 10.0);
    }

    #[test]
    fn contact_correlation_is_symmetric_and_can_be_removed() {
        let mut state = SimulationState::default();
        let first_id = state.contacts[0].id.clone();
        let second_id = state.contacts[1].id.clone();

        state
            .correlate_contacts(&first_id, &second_id, true)
            .unwrap();
        assert_eq!(
            state.contacts[0].correlated_contact_ids,
            vec![second_id.clone()]
        );
        assert_eq!(
            state.contacts[1].correlated_contact_ids,
            vec![first_id.clone()]
        );

        state
            .correlate_contacts(&first_id, &second_id, false)
            .unwrap();
        assert!(state.contacts[0].correlated_contact_ids.is_empty());
        assert!(state.contacts[1].correlated_contact_ids.is_empty());
        assert!(state
            .correlate_contacts(&first_id, &first_id, true)
            .is_err());
    }

    #[test]
    fn weapon_engagement_requires_a_classified_hostile_and_consumes_ammunition() {
        let mut state = SimulationState::default();
        assert!(state.fire_weapon("ddg-01", "harpoon", "c-101").is_err());

        state.classify_contact("c-101", "hostile").unwrap();
        let ammunition_before = state.units[0].weapons[1].ammunition;
        state.fire_weapon("ddg-01", "harpoon", "c-101").unwrap();

        assert_eq!(state.units[0].weapons[1].ammunition, ammunition_before - 1);
        assert_eq!(state.engagements.len(), 1);
        assert_eq!(state.engagements[0].contact_id, "c-101");
        assert_eq!(state.engagements[0].status, "in_flight");
        assert!(!state.contacts[0].destroyed);
        assert!(state.fire_weapon("ddg-01", "sm-6", "c-101").is_err());

        let seconds_until_impact = state.engagements[0].impact_at_seconds - state.elapsed_seconds;
        state.advance(seconds_until_impact);
        assert_eq!(state.contacts[0].destroyed, state.engagements[0].hit);
        assert_eq!(
            state.engagements[0].status,
            if state.engagements[0].hit {
                "hit"
            } else {
                "miss"
            }
        );
    }

    #[test]
    fn m7_fields_default_when_loading_a_pre_weapon_save() {
        let mut old_save = serde_json::to_value(SimulationState::default()).unwrap();
        let object = old_save.as_object_mut().unwrap();
        object.remove("engagements");
        for unit in object.get_mut("units").unwrap().as_array_mut().unwrap() {
            unit.as_object_mut().unwrap().remove("weapons");
        }
        for contact in object.get_mut("contacts").unwrap().as_array_mut().unwrap() {
            contact.as_object_mut().unwrap().remove("destroyed");
        }

        let restored: SimulationState = serde_json::from_value(old_save).unwrap();
        assert!(restored.engagements.is_empty());
        assert!(restored.units.iter().all(|unit| unit.weapons.is_empty()));
        assert!(restored.contacts.iter().all(|contact| !contact.destroyed));
    }

    #[test]
    fn unrefreshed_track_confidence_decays_and_a_new_report_restores_it() {
        let mut state = SimulationState::default();
        let initial_uncertainty = state.contacts[0].range_uncertainty_nm;

        state.advance(600);
        assert_eq!(state.contacts[0].confidence, 67);
        assert!(state.contacts[0].range_uncertainty_nm > initial_uncertainty);

        state.sensor_sweep("ddg-01", "spyd-1").unwrap();
        assert_eq!(state.contacts[0].confidence, 77);
        state.advance(600);
        assert_eq!(state.contacts[0].confidence, 72);
    }

    #[test]
    fn tracks_emit_one_stale_event_when_they_cross_the_stale_threshold() {
        let mut state = SimulationState::default();

        state.advance(1800);

        assert_eq!(
            state
                .events
                .iter()
                .filter(|event| event.message.starts_with("Track is stale:"))
                .count(),
            3
        );
        state.advance(60);
        assert_eq!(
            state
                .events
                .iter()
                .filter(|event| event.message.starts_with("Track is stale:"))
                .count(),
            3
        );
    }

    #[test]
    fn older_contacts_deserialize_with_default_track_metadata() {
        let contact = SimulationState::default().contacts.remove(0);
        let mut old_save = serde_json::to_value(contact).unwrap();
        old_save
            .as_object_mut()
            .unwrap()
            .remove("lastReportedSeconds");
        old_save
            .as_object_mut()
            .unwrap()
            .remove("confidenceAtLastReport");
        for field in [
            "rangeUncertaintyNm",
            "bearingUncertaintyDeg",
            "rangeUncertaintyAtLastReport",
            "bearingUncertaintyAtLastReport",
            "reportCount",
            "reportHistory",
            "correlatedContactIds",
        ] {
            old_save.as_object_mut().unwrap().remove(field);
        }

        let restored: super::Contact = serde_json::from_value(old_save).unwrap();
        assert_eq!(restored.last_reported_seconds, 0);
        assert_eq!(restored.confidence_at_last_report, 0);
        assert_eq!(restored.report_count, 0);
        assert!(restored.report_history.is_empty());
        assert!(restored.correlated_contact_ids.is_empty());
    }

    #[test]
    fn legacy_track_metadata_normalizes_to_save_time() {
        let mut state = SimulationState::default();
        state.contacts[0].last_reported_seconds = 0;
        state.contacts[0].confidence_at_last_report = 0;
        state.contacts[0].report_count = 0;

        state.normalize_legacy_track_metadata();

        assert_eq!(
            state.contacts[0].last_reported_seconds,
            state.elapsed_seconds
        );
        assert_eq!(
            state.contacts[0].confidence_at_last_report,
            state.contacts[0].confidence
        );
        assert!(state.contacts[0].range_uncertainty_nm > 0.0);
        assert_eq!(state.contacts[0].report_count, 1);
    }
}
