use std::{
    fs,
    path::PathBuf,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use harpoon_scenario_format::CampaignDefinition;
use harpoon_simulation_core::SimulationState;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};

const STARTER_SCENARIO_ID: &str = "north-atlantic-intercept";
const STARTER_SCENARIO: &str = include_str!("../../../../scenarios/north-atlantic-intercept.json");
const STARTER_CAMPAIGN: &str = include_str!("../../../../campaigns/northern-watch.json");
const SAVE_SLOT: &str = "campaign";
const SAVE_VERSION: u16 = 1;

struct AppState {
    simulation: Mutex<SimulationState>,
    settings: Mutex<CampaignSettings>,
    database: Mutex<Connection>,
    campaign: Mutex<CampaignProgress>,
    saved_at: Mutex<Option<u64>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CampaignSettings {
    time_scale: u8,
    layers_visible: bool,
}

impl Default for CampaignSettings {
    fn default() -> Self {
        Self {
            time_scale: 5,
            layers_visible: true,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CampaignProgress {
    campaign_id: String,
    title: String,
    description: String,
    scenario_order: Vec<String>,
    active_scenario_id: String,
    completed_scenarios: Vec<String>,
    #[serde(default)]
    scenario_results: Vec<ScenarioResult>,
    #[serde(default)]
    escalation_level: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScenarioResult {
    scenario_id: String,
    completed_at_seconds: u64,
    contacts_classified: u16,
    targets_destroyed: u16,
    engagements_launched: u16,
    hits: u16,
    damaged: u16,
    intercepted: u16,
    #[serde(default)]
    cyber_disruptions: u16,
    #[serde(default)]
    space_strikes: u16,
    #[serde(default)]
    satellites_disabled: u16,
    misses: u16,
    aborted: u16,
}

impl Default for CampaignProgress {
    fn default() -> Self {
        let definition = CampaignDefinition::parse(STARTER_CAMPAIGN)
            .expect("the built-in Northern Watch campaign must be valid");
        Self {
            campaign_id: definition.campaign_id,
            title: definition.title,
            description: definition.description,
            scenario_order: definition.scenarios,
            active_scenario_id: STARTER_SCENARIO_ID.into(),
            completed_scenarios: Vec::new(),
            scenario_results: Vec::new(),
            escalation_level: 0,
        }
    }
}

fn record_scenario_result(progress: &mut CampaignProgress, simulation: &SimulationState) {
    if !simulation.mission_complete {
        return;
    }
    if !progress
        .completed_scenarios
        .contains(&simulation.scenario_id)
    {
        progress
            .completed_scenarios
            .push(simulation.scenario_id.clone());
    }

    let count_status = |status: &str| {
        simulation
            .engagements
            .iter()
            .filter(|engagement| engagement.status == status)
            .count()
            .min(u16::MAX as usize) as u16
    };
    let result = ScenarioResult {
        scenario_id: simulation.scenario_id.clone(),
        completed_at_seconds: simulation.elapsed_seconds,
        contacts_classified: simulation
            .contacts
            .iter()
            .filter(|contact| contact.identified)
            .count()
            .min(u16::MAX as usize) as u16,
        targets_destroyed: simulation
            .contacts
            .iter()
            .filter(|contact| contact.destroyed)
            .count()
            .min(u16::MAX as usize) as u16,
        engagements_launched: simulation.engagements.len().min(u16::MAX as usize) as u16,
        hits: count_status("hit"),
        damaged: count_status("damaged"),
        intercepted: count_status("intercepted"),
        cyber_disruptions: count_status("disrupted"),
        space_strikes: simulation
            .engagements
            .iter()
            .filter(|engagement| engagement.weapon_kind == "asat")
            .count()
            .min(u16::MAX as usize) as u16,
        satellites_disabled: count_status("disabled"),
        misses: count_status("miss"),
        aborted: count_status("aborted"),
    };
    if let Some(existing) = progress
        .scenario_results
        .iter_mut()
        .find(|existing| existing.scenario_id == result.scenario_id)
    {
        *existing = result;
    } else {
        progress.scenario_results.push(result);
    }
    progress.escalation_level = escalation_from_results(&progress.scenario_results);
}

// First use per scenario counts: cyber adds 1, disabling a satellite adds 2.
// Repeat strikes within one scenario do not inflate the level further.
fn escalation_from_results(results: &[ScenarioResult]) -> u32 {
    results
        .iter()
        .map(|result| {
            u32::from(result.cyber_disruptions > 0) + u32::from(result.satellites_disabled > 0) * 2
        })
        .sum()
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CampaignSave {
    save_version: u16,
    saved_at: u64,
    campaign: CampaignProgress,
    #[serde(default)]
    settings: CampaignSettings,
    simulation: SimulationState,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CampaignStatus {
    progress: CampaignProgress,
    settings: CampaignSettings,
    has_save: bool,
    saved_at: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CampaignSession {
    simulation: SimulationState,
    status: CampaignStatus,
}

impl AppState {
    fn status(&self) -> Result<CampaignStatus, String> {
        let progress = self
            .campaign
            .lock()
            .map_err(|_| "Campaign progress is unavailable".to_string())?
            .clone();
        let settings = self
            .settings
            .lock()
            .map_err(|_| "Campaign settings are unavailable".to_string())?
            .clone();
        let saved_at = *self
            .saved_at
            .lock()
            .map_err(|_| "Campaign save status is unavailable".to_string())?;
        Ok(CampaignStatus {
            progress,
            settings,
            has_save: saved_at.is_some(),
            saved_at,
        })
    }
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

fn read_campaign_save(database: &Connection) -> Result<Option<CampaignSave>, String> {
    let payload = database
        .query_row(
            "SELECT payload_json FROM campaign_saves WHERE slot = ?1",
            [SAVE_SLOT],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| format!("Could not read campaign save: {error}"))?;
    let Some(payload) = payload else {
        return Ok(None);
    };
    let save: CampaignSave = serde_json::from_str(&payload)
        .map_err(|error| format!("Campaign save is invalid: {error}"))?;
    if save.save_version != SAVE_VERSION {
        return Err(format!(
            "Campaign save version {} is not supported",
            save.save_version
        ));
    }
    // Older saves may predate escalation (or the space domain), leaving a stale
    // level on disk. Recompute from recorded results so the field self-heals.
    let mut save = save;
    save.campaign.escalation_level = escalation_from_results(&save.campaign.scenario_results);
    Ok(Some(save))
}

#[tauri::command]
fn simulation_state(state: State<'_, AppState>) -> Result<SimulationState, String> {
    state
        .simulation
        .lock()
        .map(|current| current.clone())
        .map_err(|_| "Simulation state is unavailable".to_string())
}

#[tauri::command]
fn campaign_status(state: State<'_, AppState>) -> Result<CampaignStatus, String> {
    state.status()
}

#[tauri::command]
fn save_campaign(
    time_scale: u8,
    layers_visible: bool,
    state: State<'_, AppState>,
) -> Result<CampaignStatus, String> {
    if !matches!(time_scale, 1 | 5 | 15 | 30) {
        return Err("Time scale must be 1, 5, 15, or 30".into());
    }
    let settings = CampaignSettings {
        time_scale,
        layers_visible,
    };
    *state
        .settings
        .lock()
        .map_err(|_| "Campaign settings are unavailable".to_string())? = settings.clone();
    let mut simulation = state
        .simulation
        .lock()
        .map_err(|_| "Simulation state is unavailable".to_string())?
        .clone();
    simulation.running = false;
    let mut campaign = state
        .campaign
        .lock()
        .map_err(|_| "Campaign progress is unavailable".to_string())?
        .clone();
    campaign.active_scenario_id = simulation.scenario_id.clone();
    let saved_at = now_millis();
    let save = CampaignSave {
        save_version: SAVE_VERSION,
        saved_at,
        campaign: campaign.clone(),
        settings: settings.clone(),
        simulation,
    };
    let payload = serde_json::to_string(&save)
        .map_err(|error| format!("Could not encode campaign save: {error}"))?;
    let saved_at_db = i64::try_from(saved_at)
        .map_err(|_| "Save timestamp is outside the SQLite integer range".to_string())?;
    let database = state
        .database
        .lock()
        .map_err(|_| "Save database is unavailable".to_string())?;
    database
        .execute(
            "INSERT INTO campaign_saves (slot, save_version, saved_at_ms, payload_json) \
             VALUES (?1, ?2, ?3, ?4) \
             ON CONFLICT(slot) DO UPDATE SET \
                save_version = excluded.save_version, \
                saved_at_ms = excluded.saved_at_ms, \
                payload_json = excluded.payload_json",
            params![SAVE_SLOT, SAVE_VERSION, saved_at_db, payload],
        )
        .map_err(|error| format!("Could not save campaign: {error}"))?;
    *state
        .saved_at
        .lock()
        .map_err(|_| "Campaign save status is unavailable".to_string())? = Some(saved_at);
    *state
        .campaign
        .lock()
        .map_err(|_| "Campaign progress is unavailable".to_string())? = campaign.clone();
    Ok(CampaignStatus {
        progress: campaign,
        settings,
        has_save: true,
        saved_at: Some(saved_at),
    })
}

#[tauri::command]
fn load_campaign(state: State<'_, AppState>) -> Result<CampaignSession, String> {
    let save = {
        let database = state
            .database
            .lock()
            .map_err(|_| "Save database is unavailable".to_string())?;
        read_campaign_save(&database)?.ok_or_else(|| "No saved campaign was found".to_string())?
    };
    let mut simulation = save.simulation;
    simulation.running = false;
    simulation.normalize_legacy_track_metadata();
    *state
        .simulation
        .lock()
        .map_err(|_| "Simulation state is unavailable".to_string())? = simulation.clone();
    *state
        .campaign
        .lock()
        .map_err(|_| "Campaign progress is unavailable".to_string())? = save.campaign.clone();
    *state
        .settings
        .lock()
        .map_err(|_| "Campaign settings are unavailable".to_string())? = save.settings.clone();
    *state
        .saved_at
        .lock()
        .map_err(|_| "Campaign save status is unavailable".to_string())? = Some(save.saved_at);
    Ok(CampaignSession {
        simulation,
        status: CampaignStatus {
            progress: save.campaign,
            settings: save.settings,
            has_save: true,
            saved_at: Some(save.saved_at),
        },
    })
}

#[tauri::command]
fn load_scenario(
    scenario_id: String,
    state: State<'_, AppState>,
) -> Result<SimulationState, String> {
    if scenario_id != STARTER_SCENARIO_ID {
        return Err("Scenario not found".into());
    }
    let next = SimulationState::from_scenario_json(STARTER_SCENARIO)?;
    let mut current = state
        .simulation
        .lock()
        .map_err(|_| "Simulation state is unavailable".to_string())?;
    *current = next;
    state
        .campaign
        .lock()
        .map_err(|_| "Campaign progress is unavailable".to_string())?
        .active_scenario_id = scenario_id;
    Ok(current.clone())
}

#[tauri::command]
fn set_simulation_running(
    running: bool,
    state: State<'_, AppState>,
) -> Result<SimulationState, String> {
    let mut current = state
        .simulation
        .lock()
        .map_err(|_| "Simulation state is unavailable".to_string())?;
    if running && current.mission_complete {
        return Err("The scenario is complete; reload it to play again".into());
    }
    current.running = running;
    Ok(current.clone())
}

#[tauri::command]
fn advance_simulation(seconds: u64, state: State<'_, AppState>) -> Result<SimulationState, String> {
    let mut current = state
        .simulation
        .lock()
        .map_err(|_| "Simulation state is unavailable".to_string())?;
    current.advance(seconds);
    if current.mission_complete {
        let mut progress = state
            .campaign
            .lock()
            .map_err(|_| "Campaign progress is unavailable".to_string())?;
        record_scenario_result(&mut progress, &current);
    }
    Ok(current.clone())
}

#[tauri::command]
fn issue_course(
    unit_id: String,
    heading: u16,
    state: State<'_, AppState>,
) -> Result<SimulationState, String> {
    let mut current = state
        .simulation
        .lock()
        .map_err(|_| "Simulation state is unavailable".to_string())?;
    current.issue_course(&unit_id, heading)?;
    Ok(current.clone())
}

#[tauri::command]
fn issue_speed(
    unit_id: String,
    speed_knots: u8,
    state: State<'_, AppState>,
) -> Result<SimulationState, String> {
    let mut current = state
        .simulation
        .lock()
        .map_err(|_| "Simulation state is unavailable".to_string())?;
    current.issue_speed(&unit_id, speed_knots)?;
    Ok(current.clone())
}

#[tauri::command]
fn classify_contact(
    contact_id: String,
    identity: String,
    state: State<'_, AppState>,
) -> Result<SimulationState, String> {
    let mut current = state
        .simulation
        .lock()
        .map_err(|_| "Simulation state is unavailable".to_string())?;
    current.classify_contact(&contact_id, &identity)?;
    if current.mission_complete {
        let mut progress = state
            .campaign
            .lock()
            .map_err(|_| "Campaign progress is unavailable".to_string())?;
        record_scenario_result(&mut progress, &current);
    }
    Ok(current.clone())
}

#[tauri::command]
fn sensor_sweep(
    unit_id: String,
    sensor_id: String,
    state: State<'_, AppState>,
) -> Result<SimulationState, String> {
    let mut current = state
        .simulation
        .lock()
        .map_err(|_| "Simulation state is unavailable".to_string())?;
    current.sensor_sweep(&unit_id, &sensor_id)?;
    Ok(current.clone())
}

#[tauri::command]
fn correlate_contacts(
    contact_id: String,
    other_contact_id: String,
    correlated: bool,
    state: State<'_, AppState>,
) -> Result<SimulationState, String> {
    let mut current = state
        .simulation
        .lock()
        .map_err(|_| "Simulation state is unavailable".to_string())?;
    current.correlate_contacts(&contact_id, &other_contact_id, correlated)?;
    Ok(current.clone())
}

#[tauri::command]
fn fire_weapon(
    unit_id: String,
    weapon_id: String,
    contact_id: String,
    state: State<'_, AppState>,
) -> Result<SimulationState, String> {
    let mut current = state
        .simulation
        .lock()
        .map_err(|_| "Simulation state is unavailable".to_string())?;
    current.fire_weapon(&unit_id, &weapon_id, &contact_id)?;
    Ok(current.clone())
}

fn initialize_database(
    path: PathBuf,
) -> Result<(Connection, Option<CampaignSave>), Box<dyn std::error::Error>> {
    fs::create_dir_all(&path)?;
    let database = Connection::open(path.join("harpoon.sqlite3"))?;
    database.execute_batch(
        "PRAGMA foreign_keys = ON;
         CREATE TABLE IF NOT EXISTS campaign_saves (
             slot TEXT PRIMARY KEY NOT NULL,
             save_version INTEGER NOT NULL,
             saved_at_ms INTEGER NOT NULL,
             payload_json TEXT NOT NULL
         );",
    )?;
    let save = read_campaign_save(&database).map_err(std::io::Error::other)?;
    Ok((database, save))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let (database, save) = initialize_database(app_data_dir)?;
            let simulation = save
                .as_ref()
                .map(|saved| {
                    let mut simulation = saved.simulation.clone();
                    simulation.running = false;
                    simulation.normalize_legacy_track_metadata();
                    simulation
                })
                .unwrap_or_default();
            let campaign = save
                .as_ref()
                .map(|saved| saved.campaign.clone())
                .unwrap_or_default();
            let saved_at = save.as_ref().map(|saved| saved.saved_at);
            let settings = save
                .as_ref()
                .map(|saved| saved.settings.clone())
                .unwrap_or_default();
            app.manage(AppState {
                simulation: Mutex::new(simulation),
                settings: Mutex::new(settings),
                database: Mutex::new(database),
                campaign: Mutex::new(campaign),
                saved_at: Mutex::new(saved_at),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            simulation_state,
            campaign_status,
            save_campaign,
            load_campaign,
            load_scenario,
            set_simulation_running,
            advance_simulation,
            issue_course,
            issue_speed,
            classify_contact,
            sensor_sweep,
            correlate_contacts,
            fire_weapon
        ])
        .run(tauri::generate_context!())
        .expect("error while running Harpoon Reimagined");
}

#[cfg(test)]
mod tests {
    use super::{
        initialize_database, now_millis, read_campaign_save, record_scenario_result,
        CampaignProgress, CampaignSave, CampaignSettings, ScenarioResult, SAVE_VERSION,
    };
    use harpoon_simulation_core::{Engagement, SimulationState};
    use rusqlite::params;

    #[test]
    fn scenario_result_records_outcomes_once_and_updates_replays() {
        let mut simulation = SimulationState::default();
        simulation.mission_complete = true;
        simulation.elapsed_seconds = 12_345;
        simulation.contacts[0].identified = true;
        simulation.contacts[1].destroyed = true;
        simulation.engagements = [
            ("hit", "missile"),
            ("damaged", "missile"),
            ("intercepted", "asat"),
            ("disrupted", "cyber"),
            ("disabled", "asat"),
            ("miss", "missile"),
            ("aborted", "missile"),
        ]
        .into_iter()
        .enumerate()
        .map(|(index, (status, kind))| Engagement {
            id: index as u64 + 1,
            elapsed_seconds: 100,
            impact_at_seconds: 200,
            unit_id: "ddg-01".into(),
            unit_name: "USS ARLEIGH BURKE".into(),
            weapon_id: format!("weapon-{index}"),
            weapon_name: format!("Weapon {index}"),
            weapon_kind: kind.into(),
            contact_id: "c-101".into(),
            contact_label: "Hostile surface".into(),
            hit_chance_percent: 80,
            damage_percent: 50,
            disruption_duration_seconds: if matches!(status, "disrupted" | "disabled") {
                900
            } else {
                0
            },
            damage_applied_percent: 50,
            intercepted_by: None,
            status: status.into(),
            hit: matches!(status, "hit" | "damaged" | "disrupted" | "disabled"),
        })
        .collect();
        let mut campaign = CampaignProgress::default();

        record_scenario_result(&mut campaign, &simulation);
        record_scenario_result(&mut campaign, &simulation);

        assert_eq!(
            campaign.completed_scenarios,
            [simulation.scenario_id.clone()]
        );
        assert_eq!(campaign.scenario_results.len(), 1);
        let result = &campaign.scenario_results[0];
        assert_eq!(result.completed_at_seconds, 12_345);
        assert_eq!(result.contacts_classified, 1);
        assert_eq!(result.targets_destroyed, 1);
        assert_eq!(result.engagements_launched, 7);
        assert_eq!(result.hits, 1);
        assert_eq!(result.damaged, 1);
        assert_eq!(result.intercepted, 1);
        assert_eq!(result.cyber_disruptions, 1);
        assert_eq!(result.space_strikes, 2);
        assert_eq!(result.satellites_disabled, 1);
        assert_eq!(result.misses, 1);
        assert_eq!(result.aborted, 1);
        assert_eq!(campaign.escalation_level, 3);

        simulation.elapsed_seconds += 60;
        record_scenario_result(&mut campaign, &simulation);
        assert_eq!(campaign.completed_scenarios.len(), 1);
        assert_eq!(campaign.scenario_results.len(), 1);
        assert_eq!(campaign.scenario_results[0].completed_at_seconds, 12_405);
    }

    #[test]
    fn escalation_level_counts_first_use_per_scenario_without_double_counting() {
        let engagement = |kind: &str, status: &str| Engagement {
            id: 1,
            elapsed_seconds: 100,
            impact_at_seconds: 200,
            unit_id: "ddg-01".into(),
            unit_name: "USS ARLEIGH BURKE".into(),
            weapon_id: "w-1".into(),
            weapon_name: "Weapon One".into(),
            weapon_kind: kind.into(),
            contact_id: "c-101".into(),
            contact_label: "Hostile surface".into(),
            hit_chance_percent: 80,
            damage_percent: 0,
            disruption_duration_seconds: 900,
            damage_applied_percent: 0,
            intercepted_by: None,
            status: status.into(),
            hit: true,
        };

        let mut campaign = CampaignProgress::default();
        let mut simulation = SimulationState::default();
        simulation.mission_complete = true;
        simulation.engagements = vec![engagement("asat", "disabled")];
        record_scenario_result(&mut campaign, &simulation);
        assert_eq!(campaign.escalation_level, 2);

        // A second scenario that only used cyber adds one more level.
        simulation.scenario_id = "second-scenario".into();
        simulation.engagements = vec![engagement("cyber", "disrupted")];
        record_scenario_result(&mut campaign, &simulation);
        assert_eq!(campaign.escalation_level, 3);

        // Re-recording the same scenario replaces its result without inflating.
        record_scenario_result(&mut campaign, &simulation);
        assert_eq!(campaign.escalation_level, 3);
    }

    #[test]
    fn older_campaign_progress_defaults_to_no_scenario_results() {
        let legacy = serde_json::json!({
            "campaignId": "north-atlantic-patrol",
            "title": "Northern Watch",
            "description": "Legacy save",
            "scenarioOrder": ["north-atlantic-intercept"],
            "activeScenarioId": "north-atlantic-intercept",
            "completedScenarios": []
        });
        let progress: CampaignProgress =
            serde_json::from_value(legacy).expect("legacy campaign progress loads");
        assert!(progress.scenario_results.is_empty());
    }

    #[test]
    fn older_scenario_results_default_to_no_cyber_disruptions() {
        let legacy = serde_json::json!({
            "scenarioId": "north-atlantic-intercept",
            "completedAtSeconds": 900,
            "contactsClassified": 2,
            "targetsDestroyed": 1,
            "engagementsLaunched": 3,
            "hits": 1,
            "damaged": 1,
            "intercepted": 1,
            "misses": 0,
            "aborted": 0
        });
        let result: ScenarioResult =
            serde_json::from_value(legacy.clone()).expect("pre-cyber scenario result loads");
        assert_eq!(result.cyber_disruptions, 0);

        let mut with_space = legacy.clone();
        with_space["spaceStrikes"] = serde_json::json!(2);
        with_space["satellitesDisabled"] = serde_json::json!(1);
        let result: ScenarioResult =
            serde_json::from_value(with_space).expect("post-space scenario result loads");
        assert_eq!(result.space_strikes, 2);
        assert_eq!(result.satellites_disabled, 1);

        let legacy_progress: CampaignProgress = serde_json::from_value(serde_json::json!({
            "campaignId": "north-atlantic-patrol",
            "title": "Northern Watch",
            "description": "Legacy save",
            "scenarioOrder": ["north-atlantic-intercept"],
            "activeScenarioId": "north-atlantic-intercept",
            "completedScenarios": []
        }))
        .expect("pre-escalation campaign progress loads");
        assert_eq!(legacy_progress.escalation_level, 0);
    }

    #[test]
    fn sqlite_round_trip_preserves_campaign_simulation_and_settings() {
        let directory = std::env::temp_dir().join(format!(
            "harpoon-save-test-{}-{}",
            std::process::id(),
            now_millis()
        ));
        let (database, existing) = initialize_database(directory.clone()).expect("db initializes");
        assert!(existing.is_none());

        let mut campaign = CampaignProgress::default();
        campaign.completed_scenarios.push("intro".into());
        campaign.scenario_results.push(ScenarioResult {
            scenario_id: "intro".into(),
            completed_at_seconds: 1_800,
            contacts_classified: 2,
            targets_destroyed: 1,
            engagements_launched: 4,
            hits: 1,
            damaged: 1,
            intercepted: 1,
            cyber_disruptions: 1,
            space_strikes: 2,
            satellites_disabled: 1,
            misses: 0,
            aborted: 0,
        });
        let settings = CampaignSettings {
            time_scale: 15,
            layers_visible: false,
        };
        let mut simulation = SimulationState::default();
        simulation.advance(120);
        let saved_at = now_millis();
        let save = CampaignSave {
            save_version: SAVE_VERSION,
            saved_at,
            campaign: campaign.clone(),
            settings: settings.clone(),
            simulation: simulation.clone(),
        };
        let payload = serde_json::to_string(&save).expect("save serializes");
        database
            .execute(
                "INSERT INTO campaign_saves (slot, save_version, saved_at_ms, payload_json) \
                 VALUES ('campaign', ?1, ?2, ?3)",
                params![SAVE_VERSION, saved_at as i64, payload],
            )
            .expect("save writes to SQLite");

        let restored = read_campaign_save(&database)
            .expect("save is readable")
            .expect("save exists");
        assert_eq!(
            restored.campaign.completed_scenarios,
            campaign.completed_scenarios
        );
        assert_eq!(restored.campaign.scenario_results.len(), 1);
        assert_eq!(restored.campaign.scenario_results[0].scenario_id, "intro");
        assert_eq!(restored.campaign.scenario_results[0].intercepted, 1);
        assert_eq!(restored.campaign.scenario_results[0].cyber_disruptions, 1);
        assert_eq!(restored.campaign.scenario_results[0].space_strikes, 2);
        assert_eq!(restored.campaign.scenario_results[0].satellites_disabled, 1);
        assert_eq!(restored.settings.time_scale, 15);
        assert!(!restored.settings.layers_visible);
        assert_eq!(restored.simulation.tick, simulation.tick);

        drop(database);
        std::fs::remove_dir_all(directory).expect("test database cleanup succeeds");
    }

    #[test]
    fn recomputes_escalation_when_loading_a_legacy_save() {
        let directory = std::env::temp_dir().join(format!(
            "harpoon-save-legacy-test-{}-{}",
            std::process::id(),
            now_millis()
        ));
        let (database, _) = initialize_database(directory.clone()).expect("db initializes");

        // A save written before escalation existed has no escalationLevel key
        // even though its recorded results imply a level.
        let mut campaign = CampaignProgress::default();
        campaign.completed_scenarios.push("intro".into());
        campaign.scenario_results.push(ScenarioResult {
            scenario_id: "intro".into(),
            completed_at_seconds: 1_800,
            contacts_classified: 2,
            targets_destroyed: 0,
            engagements_launched: 3,
            hits: 0,
            damaged: 0,
            intercepted: 0,
            cyber_disruptions: 1,
            space_strikes: 1,
            satellites_disabled: 1,
            misses: 1,
            aborted: 0,
        });

        let save = CampaignSave {
            save_version: SAVE_VERSION,
            saved_at: now_millis(),
            campaign,
            settings: CampaignSettings::default(),
            simulation: SimulationState::default(),
        };
        let mut payload = serde_json::to_value(&save).expect("save serializes");
        payload["campaign"]
            .as_object_mut()
            .unwrap()
            .remove("escalationLevel");
        database
            .execute(
                "INSERT INTO campaign_saves (slot, save_version, saved_at_ms, payload_json) \
                 VALUES ('campaign', ?1, ?2, ?3)",
                params![SAVE_VERSION, save.saved_at as i64, payload.to_string()],
            )
            .expect("legacy save writes to SQLite");

        let restored = read_campaign_save(&database)
            .expect("save is readable")
            .expect("save exists");
        assert_eq!(restored.campaign.escalation_level, 3);

        drop(database);
        std::fs::remove_dir_all(directory).expect("test database cleanup succeeds");
    }

    #[test]
    fn rejects_a_save_with_an_unsupported_version() {
        let directory = std::env::temp_dir().join(format!(
            "harpoon-save-version-test-{}-{}",
            std::process::id(),
            now_millis()
        ));
        let (database, _) = initialize_database(directory.clone()).expect("db initializes");
        let save = CampaignSave {
            save_version: SAVE_VERSION + 1,
            saved_at: now_millis(),
            campaign: CampaignProgress::default(),
            settings: CampaignSettings::default(),
            simulation: SimulationState::default(),
        };
        database
            .execute(
                "INSERT INTO campaign_saves (slot, save_version, saved_at_ms, payload_json) \
                 VALUES ('campaign', ?1, ?2, ?3)",
                params![
                    save.save_version,
                    save.saved_at as i64,
                    serde_json::to_string(&save).unwrap()
                ],
            )
            .expect("unsupported save can be represented in db");

        let error = read_campaign_save(&database).unwrap_err();
        assert!(error.contains("is not supported"));

        drop(database);
        std::fs::remove_dir_all(directory).expect("test database cleanup succeeds");
    }
}
