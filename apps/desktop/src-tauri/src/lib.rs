use std::sync::Mutex;

use harpoon_simulation_core::SimulationState;

const STARTER_SCENARIO_ID: &str = "north-atlantic-intercept";
const STARTER_SCENARIO: &str = include_str!("../../../../scenarios/north-atlantic-intercept.json");

struct AppState(Mutex<SimulationState>);

#[tauri::command]
fn simulation_state(state: tauri::State<'_, AppState>) -> Result<SimulationState, String> {
    state
        .0
        .lock()
        .map(|current| current.clone())
        .map_err(|_| "Simulation state is unavailable".to_string())
}

#[tauri::command]
fn load_scenario(
    scenario_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<SimulationState, String> {
    if scenario_id != STARTER_SCENARIO_ID {
        return Err("Scenario not found".into());
    }
    let next = SimulationState::from_scenario_json(STARTER_SCENARIO)?;
    let mut current = state
        .0
        .lock()
        .map_err(|_| "Simulation state is unavailable".to_string())?;
    *current = next;
    Ok(current.clone())
}

#[tauri::command]
fn set_simulation_running(
    running: bool,
    state: tauri::State<'_, AppState>,
) -> Result<SimulationState, String> {
    let mut current = state
        .0
        .lock()
        .map_err(|_| "Simulation state is unavailable".to_string())?;
    if running && current.mission_complete {
        return Err("The scenario is complete; reload it to play again".into());
    }
    current.running = running;
    Ok(current.clone())
}

#[tauri::command]
fn advance_simulation(
    seconds: u64,
    state: tauri::State<'_, AppState>,
) -> Result<SimulationState, String> {
    let mut current = state
        .0
        .lock()
        .map_err(|_| "Simulation state is unavailable".to_string())?;
    current.advance(seconds);
    Ok(current.clone())
}

#[tauri::command]
fn issue_course(
    unit_id: String,
    heading: u16,
    state: tauri::State<'_, AppState>,
) -> Result<SimulationState, String> {
    let mut current = state
        .0
        .lock()
        .map_err(|_| "Simulation state is unavailable".to_string())?;
    current.issue_course(&unit_id, heading)?;
    Ok(current.clone())
}

#[tauri::command]
fn issue_speed(
    unit_id: String,
    speed_knots: u8,
    state: tauri::State<'_, AppState>,
) -> Result<SimulationState, String> {
    let mut current = state
        .0
        .lock()
        .map_err(|_| "Simulation state is unavailable".to_string())?;
    current.issue_speed(&unit_id, speed_knots)?;
    Ok(current.clone())
}

#[tauri::command]
fn classify_contact(
    contact_id: String,
    identity: String,
    state: tauri::State<'_, AppState>,
) -> Result<SimulationState, String> {
    let mut current = state
        .0
        .lock()
        .map_err(|_| "Simulation state is unavailable".to_string())?;
    current.classify_contact(&contact_id, &identity)?;
    Ok(current.clone())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState(Mutex::new(SimulationState::default())))
        .invoke_handler(tauri::generate_handler![
            simulation_state,
            load_scenario,
            set_simulation_running,
            advance_simulation,
            issue_course,
            issue_speed,
            classify_contact
        ])
        .run(tauri::generate_context!())
        .expect("error while running Harpoon Reimagined");
}
