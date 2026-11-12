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
        }
    }
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
        if !progress.completed_scenarios.contains(&current.scenario_id) {
            progress
                .completed_scenarios
                .push(current.scenario_id.clone());
        }
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
            sensor_sweep
        ])
        .run(tauri::generate_context!())
        .expect("error while running Harpoon Reimagined");
}

#[cfg(test)]
mod tests {
    use super::{
        initialize_database, now_millis, read_campaign_save, CampaignProgress, CampaignSave,
        CampaignSettings, SAVE_VERSION,
    };
    use harpoon_simulation_core::SimulationState;
    use rusqlite::params;

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
        assert_eq!(restored.settings.time_scale, 15);
        assert!(!restored.settings.layers_visible);
        assert_eq!(restored.simulation.tick, simulation.tick);

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
