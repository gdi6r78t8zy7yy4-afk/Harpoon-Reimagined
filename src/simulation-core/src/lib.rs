use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
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

#[derive(Clone, Debug, Serialize)]
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
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationState {
    pub tick: u64,
    pub elapsed_seconds: u64,
    pub running: bool,
    pub units: Vec<Unit>,
    pub contacts: Vec<Contact>,
}

impl Default for SimulationState {
    fn default() -> Self {
        Self {
            tick: 0,
            elapsed_seconds: 15_510,
            running: false,
            units: vec![
                Unit {
                    id: "ddg-01".into(),
                    name: "USS Arleigh Burke".into(),
                    class_name: "Flight III · DDG".into(),
                    side: "friendly".into(),
                    status: "Underway".into(),
                    x: -5.0,
                    z: 2.0,
                    heading: 42.0,
                    speed_knots: 18,
                    health: 100,
                },
                Unit {
                    id: "ssn-01".into(),
                    name: "USS Seawolf".into(),
                    class_name: "Seawolf class · SSN".into(),
                    side: "friendly".into(),
                    status: "Silent running".into(),
                    x: -8.0,
                    z: -5.0,
                    heading: 68.0,
                    speed_knots: 6,
                    health: 100,
                },
                Unit {
                    id: "ffg-01".into(),
                    name: "HNoMS Fridtjof Nansen".into(),
                    class_name: "Fridtjof Nansen · FFG".into(),
                    side: "friendly".into(),
                    status: "Screening".into(),
                    x: -1.0,
                    z: 5.0,
                    heading: 18.0,
                    speed_knots: 14,
                    health: 100,
                },
            ],
            contacts: vec![
                Contact {
                    id: "c-101".into(),
                    label: "UNKNOWN SURFACE".into(),
                    classification: "Surface contact".into(),
                    confidence: 72,
                    bearing: 284,
                    range_nm: 38.4,
                    domain: "surface".into(),
                    x: 5.0,
                    z: -1.5,
                },
                Contact {
                    id: "c-102".into(),
                    label: "AIR CONTACT".into(),
                    classification: "Fast mover · probable".into(),
                    confidence: 46,
                    bearing: 031,
                    range_nm: 62.1,
                    domain: "air".into(),
                    x: 3.0,
                    z: -7.0,
                },
                Contact {
                    id: "c-103".into(),
                    label: "SUBSURFACE".into(),
                    classification: "Transient acoustic".into(),
                    confidence: 31,
                    bearing: 198,
                    range_nm: 17.8,
                    domain: "subsurface".into(),
                    x: 1.5,
                    z: 7.0,
                },
            ],
        }
    }
}

impl SimulationState {
    pub fn advance(&mut self, seconds: u64) {
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
}
