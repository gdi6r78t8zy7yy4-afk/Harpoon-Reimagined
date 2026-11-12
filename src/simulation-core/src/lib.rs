use harpoon_scenario_format::ScenarioDefinition;
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
}

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
    pub objectives: Vec<ObjectiveProgress>,
    pub events: Vec<SimulationEvent>,
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
            elapsed_seconds: scenario.start_time_seconds,
            running: false,
            mission_complete: false,
            units,
            contacts,
            objectives,
            events: vec![
                SimulationEvent {
                    id: 1,
                    elapsed_seconds: scenario.start_time_seconds,
                    message: "Unidentified surface contact detected".into(),
                    source: "USS ARLEIGH BURKE".into(),
                },
                SimulationEvent {
                    id: 2,
                    elapsed_seconds: scenario.start_time_seconds.saturating_sub(142),
                    message: "Transient acoustic signal reported".into(),
                    source: "USS SEAWOLF".into(),
                },
            ],
        })
    }

    pub fn advance(&mut self, seconds: u64) {
        if self.mission_complete {
            return;
        }
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
        contact.identified = true;
        let report = contact.label.clone();
        self.update_objectives();
        self.push_event(format!("Contact classified: {report}"), "COMMAND".into());
        if self.objectives.iter().all(|objective| objective.completed) {
            self.mission_complete = true;
            self.running = false;
            self.push_event(
                "Mission complete: contact picture established".into(),
                "MISSION CONTROL".into(),
            );
        }
        Ok(())
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
}
