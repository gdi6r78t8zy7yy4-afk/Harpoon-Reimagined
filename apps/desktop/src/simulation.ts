import { invoke } from '@tauri-apps/api/core'
import scenario from './data/north-atlantic-intercept.json'
import campaignDefinition from './data/northern-watch.json'
import type { CampaignProgress, CampaignSession, CampaignSettings, CampaignStatus, Contact, SimulationState, Unit } from './types'

declare global {
  interface Window { __TAURI_INTERNALS__?: unknown }
}

type ScenarioUnit = Omit<Unit, 'x' | 'z'> & { position: { x: number; z: number } }
type ScenarioContact = Omit<Contact, 'x' | 'z'> & { position: { x: number; z: number } }

function createBrowserState(): SimulationState {
  return {
    scenarioId: scenario.id,
    missionTitle: scenario.title,
    theatre: scenario.theatre,
    tick: 0,
    elapsedSeconds: scenario.startTimeSeconds,
    running: false,
    missionComplete: false,
    units: (scenario.units as ScenarioUnit[]).map(({ position, ...unit }) => ({ ...unit, x: position.x, z: position.z })),
    contacts: (scenario.contacts as ScenarioContact[]).map(({ position, ...contact }) => ({ ...contact, x: position.x, z: position.z })),
    objectives: scenario.objectives.map((objective) => ({ ...objective, progress: 0, completed: false })),
    events: [
      { id: 1, elapsedSeconds: scenario.startTimeSeconds, message: 'Unidentified surface contact detected', source: 'USS ARLEIGH BURKE' },
      { id: 2, elapsedSeconds: scenario.startTimeSeconds - 142, message: 'Transient acoustic signal reported', source: 'USS SEAWOLF' },
    ],
  }
}

const desktop = () => typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
const saveKey = 'harpoon.campaign-save.v1'
const defaultCampaign = (): CampaignProgress => ({
  campaignId: campaignDefinition.campaignId,
  title: campaignDefinition.title,
  description: campaignDefinition.description,
  scenarioOrder: campaignDefinition.scenarios,
  activeScenarioId: campaignDefinition.scenarios[0] ?? scenario.id,
  completedScenarios: [],
})

type BrowserSave = { saveVersion: number; savedAt: number; campaign: CampaignProgress; settings?: CampaignSettings; simulation: SimulationState }

function readBrowserSave(): BrowserSave | null {
  if (typeof window === 'undefined') return null
  try {
    const saved = window.localStorage.getItem(saveKey)
    if (!saved) return null
    const decoded = JSON.parse(saved) as BrowserSave
    return decoded.saveVersion === 1 ? decoded : null
  } catch {
    return null
  }
}

const loadedBrowserSave = readBrowserSave()
let browserState = loadedBrowserSave?.simulation ?? createBrowserState()
browserState.running = false
let browserCampaign = loadedBrowserSave?.campaign ?? defaultCampaign()
let browserSavedAt: number | null = loadedBrowserSave?.savedAt ?? null
let browserSettings: CampaignSettings = loadedBrowserSave?.settings ?? { timeScale: 5, layersVisible: true }

function browserStatus(): CampaignStatus {
  return { progress: structuredClone(browserCampaign), settings: structuredClone(browserSettings), hasSave: browserSavedAt !== null, savedAt: browserSavedAt }
}

export async function loadState() {
  return desktop() ? invoke<SimulationState>('simulation_state') : structuredClone(browserState)
}

export async function loadScenario(scenarioId: string) {
  if (desktop()) return invoke<SimulationState>('load_scenario', { scenarioId })
  if (scenarioId !== scenario.id) throw new Error('Scenario not found')
  browserState = createBrowserState()
  browserCampaign.activeScenarioId = scenarioId
  return structuredClone(browserState)
}

export async function campaignStatus() {
  return desktop() ? invoke<CampaignStatus>('campaign_status') : browserStatus()
}

export async function saveCampaign(settings: CampaignSettings) {
  if (desktop()) return invoke<CampaignStatus>('save_campaign', settings)
  browserSettings = structuredClone(settings)
  browserSavedAt = Date.now()
  const save: BrowserSave = { saveVersion: 1, savedAt: browserSavedAt, campaign: browserCampaign, settings: browserSettings, simulation: { ...browserState, running: false } }
  try { window.localStorage.setItem(saveKey, JSON.stringify(save)) } catch { throw new Error('Campaign save could not be written in this browser') }
  return browserStatus()
}

export async function loadCampaign(): Promise<CampaignSession> {
  if (desktop()) return invoke<CampaignSession>('load_campaign')
  const save = readBrowserSave()
  if (!save) throw new Error('No saved campaign was found')
  browserState = { ...save.simulation, running: false }
  browserCampaign = save.campaign
  browserSettings = save.settings ?? { timeScale: 5, layersVisible: true }
  browserSavedAt = save.savedAt
  return { simulation: structuredClone(browserState), status: browserStatus() }
}

export async function setRunning(running: boolean) {
  if (desktop()) return invoke<SimulationState>('set_simulation_running', { running })
  if (running && browserState.missionComplete) throw new Error('Scenario complete; reload it to play again')
  browserState.running = running
  return structuredClone(browserState)
}

export async function advance(seconds = 60) {
  if (desktop()) return invoke<SimulationState>('advance_simulation', { seconds })
  if (browserState.missionComplete) return structuredClone(browserState)
  const safeSeconds = Math.max(1, Math.min(seconds, 3600))
  browserState.tick += 1
  browserState.elapsedSeconds += safeSeconds
  for (const unit of browserState.units) {
    const radians = unit.heading * Math.PI / 180
    const distance = unit.speedKnots * safeSeconds / 3600
    unit.x += Math.sin(radians) * distance
    unit.z -= Math.cos(radians) * distance
  }
  return structuredClone(browserState)
}

export async function issueCourse(unitId: string, heading: number) {
  if (desktop()) return invoke<SimulationState>('issue_course', { unitId, heading })
  if (browserState.missionComplete) throw new Error('Scenario complete; reload it to play again')
  const unit = browserState.units.find((item) => item.id === unitId)
  if (!unit) throw new Error('Unit not found')
  if (heading < 0 || heading > 359) throw new Error('Course must be between 000° and 359°')
  unit.heading = heading
  addEvent(`New course set to ${heading.toString().padStart(3, '0')}°`, unit.name)
  return structuredClone(browserState)
}

export async function issueSpeed(unitId: string, speedKnots: number) {
  if (desktop()) return invoke<SimulationState>('issue_speed', { unitId, speedKnots })
  if (browserState.missionComplete) throw new Error('Scenario complete; reload it to play again')
  const unit = browserState.units.find((item) => item.id === unitId)
  if (!unit) throw new Error('Unit not found')
  if (speedKnots < 0 || speedKnots > 50) throw new Error('Speed must be between 0 and 50 knots')
  unit.speedKnots = speedKnots
  addEvent(`Speed changed to ${speedKnots} kt`, unit.name)
  return structuredClone(browserState)
}

export async function classifyContact(contactId: string, identity: 'hostile' | 'neutral') {
  if (desktop()) return invoke<SimulationState>('classify_contact', { contactId, identity })
  if (browserState.missionComplete) throw new Error('Scenario complete; reload it to play again')
  const contact = browserState.contacts.find((item) => item.id === contactId)
  if (!contact) throw new Error('Contact not found')
  if (contact.identified) throw new Error('This contact has already been classified')
  contact.label = `${identity.toUpperCase()} ${contact.domain.toUpperCase()}`
  contact.classification = identity === 'hostile' ? 'Hostile platform' : 'Neutral platform'
  contact.confidence = 100
  contact.identified = true
  for (const objective of browserState.objectives) {
    const progress = browserState.contacts.filter((item) => item.identified).length
    objective.progress = Math.min(progress, objective.required)
    objective.completed = objective.progress >= objective.required
  }
  addEvent(`Contact classified: ${contact.label}`, 'COMMAND')
  if (browserState.objectives.every((objective) => objective.completed)) {
    browserState.missionComplete = true
    browserState.running = false
    if (!browserCampaign.completedScenarios.includes(browserState.scenarioId)) browserCampaign.completedScenarios.push(browserState.scenarioId)
    addEvent('Mission complete: contact picture established', 'MISSION CONTROL')
  }
  return structuredClone(browserState)
}

function addEvent(message: string, source: string) {
  const id = (browserState.events.at(-1)?.id ?? 0) + 1
  browserState.events.push({ id, elapsedSeconds: browserState.elapsedSeconds, message, source })
  browserState.events = browserState.events.slice(-12)
}
