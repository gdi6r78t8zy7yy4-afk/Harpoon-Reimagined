use harpoon_simulation_core::SimulationState;
use std::sync::Mutex;

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
fn set_simulation_running(
    running: bool,
    state: tauri::State<'_, AppState>,
) -> Result<SimulationState, String> {
    let mut current = state
        .0
        .lock()
        .map_err(|_| "Simulation state is unavailable".to_string())?;
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
    current.advance(seconds.clamp(1, 3600));
    Ok(current.clone())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState(Mutex::new(SimulationState::default())))
        .invoke_handler(tauri::generate_handler![
            simulation_state,
            set_simulation_running,
            advance_simulation
        ])
        .run(tauri::generate_context!())
        .expect("error while running Harpoon Reimagined");
}
