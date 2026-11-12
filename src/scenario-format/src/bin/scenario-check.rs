use harpoon_scenario_format::{CampaignDefinition, ScenarioDefinition};
use std::{env, fs, path::PathBuf, process};

fn main() {
    if let Err(error) = run() {
        eprintln!("scenario-check: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args_os().skip(1);
    let kind = args
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or_else(usage)?;
    let path = args.next().map(PathBuf::from).ok_or_else(usage)?;
    if args.next().is_some() {
        return Err(usage());
    }

    let json = fs::read_to_string(&path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    match kind.as_str() {
        "scenario" => ScenarioDefinition::parse(&json).map(|_| ()),
        "campaign" => CampaignDefinition::parse(&json).map(|_| ()),
        _ => return Err(usage()),
    }
    .map_err(|error| format!("{}: {error}", path.display()))?;

    println!("Valid {kind}: {}", path.display());
    Ok(())
}

fn usage() -> String {
    "usage: scenario-check <scenario|campaign> <path.json>".into()
}
