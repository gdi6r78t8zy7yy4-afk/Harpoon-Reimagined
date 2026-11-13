import { invoke } from '@tauri-apps/api/core'
import scenario from './data/north-atlantic-intercept.json'
import campaignDefinition from './data/northern-watch.json'
import type { CampaignProgress, CampaignSession, CampaignSettings, CampaignStatus, Contact, SimulationState, Unit } from './types'

declare global {
  interface Window { __TAURI_INTERNALS__?: unknown }
}

type ScenarioUnit = Omit<Unit, 'x' | 'z'> & { position: { x: number; z: number } }
type ScenarioDefense = { id: string; name: string; charges: number; interceptChancePercent: number }
type ScenarioContact = Omit<Contact, 'x' | 'z' | 'lastReportedSeconds' | 'confidenceAtLastReport' | 'rangeUncertaintyNm' | 'bearingUncertaintyDeg' | 'rangeUncertaintyAtLastReport' | 'bearingUncertaintyAtLastReport' | 'reportCount' | 'reportHistory' | 'correlatedContactIds' | 'destroyed' | 'health' | 'defenses'> & { position: { x: number; z: number }; defenses: ScenarioDefense[] }

const initialRangeUncertainty = (confidence: number) => 1 + (100 - confidence) * 0.12
const initialBearingUncertainty = (confidence: number) => 2 + (100 - confidence) * 0.5

function fuseBearing(prior: number, priorSigma: number, observed: number, observedSigma: number) {
  const priorWeight = 1 / Math.max(0.1, priorSigma) ** 2
  const observedWeight = 1 / Math.max(0.1, observedSigma) ** 2
  const priorRadians = prior * Math.PI / 180
  const observedRadians = observed * Math.PI / 180
  const x = Math.cos(priorRadians) * priorWeight + Math.cos(observedRadians) * observedWeight
  const y = Math.sin(priorRadians) * priorWeight + Math.sin(observedRadians) * observedWeight
  return {
    bearing: (Math.round(Math.atan2(y, x) * 180 / Math.PI) + 360) % 360,
    uncertainty: Math.sqrt(1 / (priorWeight + observedWeight)),
  }
}

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
    contacts: (scenario.contacts as ScenarioContact[]).map(({ position, ...contact }) => ({ ...contact, x: position.x, z: position.z, lastReportedSeconds: scenario.startTimeSeconds, confidenceAtLastReport: contact.confidence, rangeUncertaintyNm: initialRangeUncertainty(contact.confidence), bearingUncertaintyDeg: initialBearingUncertainty(contact.confidence), rangeUncertaintyAtLastReport: initialRangeUncertainty(contact.confidence), bearingUncertaintyAtLastReport: initialBearingUncertainty(contact.confidence), reportCount: 1, reportHistory: [], correlatedContactIds: [], destroyed: false, health: 100, defenses: (contact.defenses ?? []).map(({ charges, ...defense }) => ({ ...defense, remainingCharges: charges })) })),
    engagements: [],
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
  scenarioResults: [],
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
function normalizeBrowserContacts(state: SimulationState) {
  for (const unit of state.units) {
    if (!Array.isArray(unit.weapons)) unit.weapons = []
    for (const weapon of unit.weapons) {
      if (!Number.isFinite(weapon.damagePercent) || weapon.damagePercent <= 0) weapon.damagePercent = 100
    }
  }
  for (const contact of state.contacts) {
    if (!Number.isFinite(contact.lastReportedSeconds) || contact.lastReportedSeconds === 0) {
      contact.lastReportedSeconds = state.elapsedSeconds
      contact.confidenceAtLastReport = contact.confidence
    } else if (!Number.isFinite(contact.confidenceAtLastReport)) {
      contact.confidenceAtLastReport = contact.confidence
    }
    if (!Number.isFinite(contact.reportCount) || contact.reportCount < 1) {
      contact.rangeUncertaintyNm = initialRangeUncertainty(contact.confidence)
      contact.bearingUncertaintyDeg = initialBearingUncertainty(contact.confidence)
      contact.rangeUncertaintyAtLastReport = contact.rangeUncertaintyNm
      contact.bearingUncertaintyAtLastReport = contact.bearingUncertaintyDeg
      contact.reportCount = 1
    }
    if (!Array.isArray(contact.reportHistory)) contact.reportHistory = []
    if (!Array.isArray(contact.correlatedContactIds)) contact.correlatedContactIds = []
    if (typeof contact.destroyed !== 'boolean') contact.destroyed = false
    if (!Number.isFinite(contact.health)) contact.health = 100
    if (!Array.isArray(contact.defenses)) contact.defenses = []
    if (contact.destroyed) contact.health = 0
    else if (contact.health <= 0) contact.destroyed = true
  }
  if (!Array.isArray(state.engagements)) state.engagements = []
  for (const engagement of state.engagements) {
    if (!Number.isFinite(engagement.damagePercent) || engagement.damagePercent <= 0) engagement.damagePercent = 100
    if (!Number.isFinite(engagement.damageAppliedPercent)) engagement.damageAppliedPercent = 0
    if (typeof engagement.interceptedBy !== 'string') engagement.interceptedBy = null
  }
}
normalizeBrowserContacts(browserState)
let browserCampaign = loadedBrowserSave?.campaign ?? defaultCampaign()
if (!Array.isArray(browserCampaign.scenarioResults)) browserCampaign.scenarioResults = []
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
  normalizeBrowserContacts(browserState)
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
  const previousElapsedSeconds = browserState.elapsedSeconds
  const safeSeconds = Math.max(1, Math.min(seconds, 3600))
  browserState.tick += 1
  browserState.elapsedSeconds += safeSeconds
  for (const unit of browserState.units) {
    const radians = unit.heading * Math.PI / 180
    const distance = unit.speedKnots * safeSeconds / 3600
    unit.x += Math.sin(radians) * distance
    unit.z -= Math.cos(radians) * distance
  }
  for (const contact of browserState.contacts) {
    if (contact.identified) continue
    if (contact.lastReportedSeconds === 0) {
      contact.lastReportedSeconds = previousElapsedSeconds
      contact.confidenceAtLastReport = contact.confidence
    }
    const previousAge = previousElapsedSeconds - contact.lastReportedSeconds
    const trackAge = browserState.elapsedSeconds - contact.lastReportedSeconds
    contact.confidence = Math.max(0, contact.confidenceAtLastReport - Math.floor(trackAge / 600) * 5)
    const ageSteps = Math.floor(trackAge / 600)
    contact.rangeUncertaintyNm = Math.min(100, contact.rangeUncertaintyAtLastReport + ageSteps * 0.2)
    contact.bearingUncertaintyDeg = Math.min(180, contact.bearingUncertaintyAtLastReport + ageSteps)
    if (previousAge < 1800 && trackAge >= 1800) addEvent(`Track is stale: ${contact.label}`, 'SENSORS')
  }
  for (const engagement of browserState.engagements) {
    if (engagement.status !== 'in_flight' || engagement.impactAtSeconds > browserState.elapsedSeconds) continue
    const target = browserState.contacts.find((item) => item.id === engagement.contactId)
    if (engagement.hit && target && !target.destroyed) {
      const survival = target.defenses.filter((defense) => defense.remainingCharges > 0).reduce(
        (chance, defense) => Math.floor(chance * (100 - defense.interceptChancePercent) / 100),
        100,
      )
      const contactSalt = [...target.id].reduce((sum, character) => sum + character.charCodeAt(0), 0)
      const defenseRoll = (engagement.impactAtSeconds + engagement.id * 19 + contactSalt) % 100
      if (defenseRoll < 100 - survival) {
        const defense = target.defenses.find((item) => item.remainingCharges > 0)
        if (defense) {
          defense.remainingCharges -= 1
          engagement.interceptedBy = defense.name
        }
        engagement.hit = false
        engagement.status = 'intercepted'
      } else {
        engagement.damageAppliedPercent = Math.min(target.health, engagement.damagePercent)
        target.health = Math.max(0, target.health - engagement.damagePercent)
        target.destroyed = target.health === 0
        engagement.status = target.destroyed ? 'hit' : 'damaged'
      }
    } else if (engagement.hit) {
      engagement.hit = false
      engagement.status = 'aborted'
    } else {
      engagement.status = 'miss'
    }
    const message = engagement.status === 'hit'
      ? `${engagement.weaponName} hit ${engagement.contactLabel}`
      : engagement.status === 'damaged'
        ? `${engagement.weaponName} damaged ${engagement.contactLabel} (${engagement.damageAppliedPercent}% damage)`
        : engagement.status === 'intercepted'
          ? `${engagement.weaponName} intercepted by ${engagement.interceptedBy ?? 'defenses'} protecting ${engagement.contactLabel}`
      : engagement.status === 'miss'
        ? `${engagement.weaponName} missed ${engagement.contactLabel}`
        : `${engagement.weaponName} impact aborted; ${engagement.contactLabel} is no longer active`
    addEvent(message, 'WEAPONS CONTROL')
  }
  updateMissionCompletion()
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

export async function sensorSweep(unitId: string, sensorId: string) {
  if (desktop()) return invoke<SimulationState>('sensor_sweep', { unitId, sensorId })
  if (browserState.missionComplete) throw new Error('Scenario complete; reload it to sweep sensors')
  const unit = browserState.units.find((item) => item.id === unitId)
  if (!unit) throw new Error('Unit not found')
  const sensor = unit.sensors.find((item) => item.id === sensorId)
  if (!sensor) throw new Error('Sensor not found on this unit')
  let reports = 0
  for (const contact of browserState.contacts) {
    if (contact.identified) continue
    const dx = contact.x - unit.x
    const dz = contact.z - unit.z
    const distanceNm = Math.hypot(dx, dz) * 5
    const capable = sensor.domains.includes(contact.domain) && distanceNm <= sensor.rangeNm
    if (!capable) continue
    const confidenceGain = sensor.kind === 'radar' ? 10 : sensor.kind === 'sonar' ? 7 : 5
    const observedBearing = ((Math.round(Math.atan2(dx, -dz) * 180 / Math.PI) % 360) + 360) % 360
    const bearingSigma = sensor.kind === 'radar' ? 2 : sensor.kind === 'sonar' ? 6 : 10
    const fusedBearing = fuseBearing(contact.bearing, contact.bearingUncertaintyDeg, observedBearing, bearingSigma)
    contact.bearing = fusedBearing.bearing
    contact.bearingUncertaintyDeg = fusedBearing.uncertainty
    if (sensor.kind === 'radar') {
      const priorVariance = Math.max(0.1, contact.rangeUncertaintyNm) ** 2
      const observedVariance = 0.5 ** 2
      const priorWeight = 1 / priorVariance
      const observedWeight = 1 / observedVariance
      contact.rangeNm = (contact.rangeNm * priorWeight + distanceNm * observedWeight) / (priorWeight + observedWeight)
      contact.rangeUncertaintyNm = Math.sqrt(1 / (priorWeight + observedWeight))
    }
    contact.confidence = Math.min(99, contact.confidence + confidenceGain)
    contact.lastReportedSeconds = browserState.elapsedSeconds
    contact.confidenceAtLastReport = contact.confidence
    contact.rangeUncertaintyAtLastReport = contact.rangeUncertaintyNm
    contact.bearingUncertaintyAtLastReport = contact.bearingUncertaintyDeg
    contact.reportCount += 1
    contact.reportHistory.push({ elapsedSeconds: browserState.elapsedSeconds, unitId, unitName: unit.name, sensorId, sensorName: sensor.name, sensorKind: sensor.kind })
    contact.reportHistory = contact.reportHistory.slice(-20)
    reports += 1
  }
  addEvent(reports ? `${sensor.name} sweep complete: ${reports} contact report(s) updated` : `${sensor.name} sweep complete: no contacts updated`, unit.name)
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
  contact.lastReportedSeconds = browserState.elapsedSeconds
  contact.confidenceAtLastReport = 100
  contact.rangeUncertaintyAtLastReport = contact.rangeUncertaintyNm
  contact.bearingUncertaintyAtLastReport = contact.bearingUncertaintyDeg
  contact.identified = true
  for (const objective of browserState.objectives) {
    const progress = browserState.contacts.filter((item) => item.identified).length
    objective.progress = Math.min(progress, objective.required)
    objective.completed = objective.progress >= objective.required
  }
  addEvent(`Contact classified: ${contact.label}`, 'COMMAND')
  updateMissionCompletion()
  return structuredClone(browserState)
}

export async function correlateContacts(contactId: string, otherContactId: string, correlated: boolean) {
  if (desktop()) return invoke<SimulationState>('correlate_contacts', { contactId, otherContactId, correlated })
  if (browserState.missionComplete) throw new Error('Scenario complete; reload it to manage contact correlation')
  if (contactId === otherContactId) throw new Error('A contact cannot be correlated with itself')
  const contact = browserState.contacts.find((item) => item.id === contactId)
  const other = browserState.contacts.find((item) => item.id === otherContactId)
  if (!contact || !other) throw new Error('Contact not found')
  const updatePeer = (peer: Contact, peerId: string) => {
    peer.correlatedContactIds = correlated
      ? [...new Set([...peer.correlatedContactIds, peerId])].sort()
      : peer.correlatedContactIds.filter((id) => id !== peerId)
  }
  updatePeer(contact, otherContactId)
  updatePeer(other, contactId)
  addEvent(`Tracks ${correlated ? 'correlated' : 'separated'}: ${contact.label} / ${other.label}`, 'COMMAND')
  return structuredClone(browserState)
}

export async function fireWeapon(unitId: string, weaponId: string, contactId: string) {
  if (desktop()) return invoke<SimulationState>('fire_weapon', { unitId, weaponId, contactId })
  if (browserState.missionComplete) throw new Error('Scenario complete; reload it before engaging targets')
  const unit = browserState.units.find((item) => item.id === unitId)
  if (!unit) throw new Error('Unit not found')
  if (unit.side !== 'friendly') throw new Error('Only friendly units can engage targets')
  const weapon = unit.weapons.find((item) => item.id === weaponId)
  if (!weapon) throw new Error('Weapon not found on this unit')
  const contact = browserState.contacts.find((item) => item.id === contactId)
  if (!contact) throw new Error('Contact not found')
  if (contact.destroyed) throw new Error('This contact has already been destroyed')
  if (!contact.identified || contact.classification !== 'Hostile platform') throw new Error('Only identified hostile contacts can be engaged')
  if (!weapon.domains.includes(contact.domain)) throw new Error("This weapon cannot engage the contact's domain")
  if (contact.rangeNm > weapon.rangeNm) throw new Error("Contact is outside this weapon's estimated range")
  if (weapon.ammunition <= 0) throw new Error('No ammunition remains for this weapon')
  const unitIndex = browserState.units.findIndex((item) => item.id === unitId)
  const weaponIndex = browserState.units[unitIndex].weapons.findIndex((item) => item.id === weaponId)
  const confidenceAdjustment = Math.trunc((contact.confidence - 50) / 5)
  const rangePenalty = Math.round(contact.rangeNm / weapon.rangeNm * 15)
  const hitChancePercent = Math.max(5, Math.min(95, weapon.hitChancePercent + confidenceAdjustment - rangePenalty))
  const id = (browserState.engagements.at(-1)?.id ?? 0) + 1
  const flightSeconds = Math.max(1, Math.ceil(contact.rangeNm / weapon.speedKnots * 3600))
  const idSalt = [...`${unitId}${weaponId}${contactId}`].reduce((sum, character) => sum + character.charCodeAt(0), 0)
  const roll = (browserState.elapsedSeconds + browserState.tick * 17 + id * 31 + idSalt) % 100
  const hit = roll < hitChancePercent
  browserState.units[unitIndex].weapons[weaponIndex].ammunition -= 1
  browserState.engagements.push({ id, elapsedSeconds: browserState.elapsedSeconds, impactAtSeconds: browserState.elapsedSeconds + flightSeconds, unitId, unitName: unit.name, weaponId, weaponName: weapon.name, contactId, contactLabel: contact.label, hitChancePercent, damagePercent: weapon.damagePercent, damageAppliedPercent: 0, interceptedBy: null, status: 'in_flight', hit })
  browserState.engagements = browserState.engagements.slice(-50)
  addEvent(`${weapon.name} launched at ${contact.label} (${hitChancePercent}% solution)`, unit.name)
  return structuredClone(browserState)
}

function updateMissionCompletion() {
  if (browserState.missionComplete
    || !browserState.objectives.every((objective) => objective.completed)
    || browserState.engagements.some((engagement) => engagement.status === 'in_flight')) return
  browserState.missionComplete = true
  browserState.running = false
  if (!browserCampaign.completedScenarios.includes(browserState.scenarioId)) browserCampaign.completedScenarios.push(browserState.scenarioId)
  const countStatus = (status: SimulationState['engagements'][number]['status']) => browserState.engagements.filter((engagement) => engagement.status === status).length
  const result = {
    scenarioId: browserState.scenarioId,
    completedAtSeconds: browserState.elapsedSeconds,
    contactsClassified: browserState.contacts.filter((contact) => contact.identified).length,
    targetsDestroyed: browserState.contacts.filter((contact) => contact.destroyed).length,
    engagementsLaunched: browserState.engagements.length,
    hits: countStatus('hit'),
    damaged: countStatus('damaged'),
    intercepted: countStatus('intercepted'),
    misses: countStatus('miss'),
    aborted: countStatus('aborted'),
  }
  const existingResult = browserCampaign.scenarioResults.findIndex((item) => item.scenarioId === result.scenarioId)
  if (existingResult >= 0) browserCampaign.scenarioResults[existingResult] = result
  else browserCampaign.scenarioResults.push(result)
  addEvent('Mission complete: contact picture established', 'MISSION CONTROL')
}

function addEvent(message: string, source: string) {
  const id = (browserState.events.at(-1)?.id ?? 0) + 1
  browserState.events.push({ id, elapsedSeconds: browserState.elapsedSeconds, message, source })
  browserState.events = browserState.events.slice(-12)
}
