export type Unit = {
  id: string
  name: string
  className: string
  side: string
  status: string
  x: number
  z: number
  heading: number
  speedKnots: number
  health: number
  sensors: Sensor[]
  weapons: Weapon[]
}

export type Sensor = {
  id: string
  name: string
  kind: 'radar' | 'sonar' | 'esm'
  rangeNm: number
  domains: Array<'surface' | 'air' | 'subsurface'>
}

export type Weapon = {
  id: string
  name: string
  kind: 'missile' | 'torpedo' | 'gun' | 'cyber' | 'asat'
  rangeNm: number
  domains: Array<'surface' | 'air' | 'subsurface' | 'space'>
  ammunition: number
  hitChancePercent: number
  speedKnots: number
  damagePercent: number
  effectDurationSeconds: number
}

export type DefenseSystem = {
  id: string
  name: string
  remainingCharges: number
  interceptChancePercent: number
}

export type SatellitePass = {
  startSeconds: number
  durationSeconds: number
}

export type OrbitalSensor = {
  id: string
  name: string
  rangeNm: number
  domains: Array<'surface' | 'air'>
  passes: SatellitePass[]
}

export type Contact = {
  id: string
  label: string
  classification: string
  confidence: number
  bearing: number
  rangeNm: number
  domain: 'surface' | 'air' | 'subsurface' | 'space'
  x: number
  z: number
  identified: boolean
  health: number
  defenses: DefenseSystem[]
  defensesDisruptedUntilSeconds: number
  defenseDisruptionSource: string | null
  groundTrackHeadingDeg: number
  groundTrackSpeedKnots: number
  orbitalSensor: OrbitalSensor | null
  hardeningSeconds: number
  hardeningUntilSeconds: number
  sensorsDisabledUntilSeconds: number
  sensorDisableSource: string | null
  lastReportedSeconds: number
  confidenceAtLastReport: number
  rangeUncertaintyNm: number
  bearingUncertaintyDeg: number
  rangeUncertaintyAtLastReport: number
  bearingUncertaintyAtLastReport: number
  reportCount: number
  reportHistory: TrackReport[]
  correlatedContactIds: string[]
  destroyed: boolean
}

export type TrackReport = {
  elapsedSeconds: number
  unitId: string
  unitName: string
  sensorId: string
  sensorName: string
  sensorKind: Sensor['kind'] | 'space-based'
}

export type Objective = {
  id: string
  title: string
  description: string
  progress: number
  required: number
  completed: boolean
}

export type SimulationEvent = {
  id: number
  elapsedSeconds: number
  message: string
  source: string
}

export type SimulationState = {
  scenarioId: string
  missionTitle: string
  theatre: string
  tick: number
  elapsedSeconds: number
  running: boolean
  missionComplete: boolean
  units: Unit[]
  contacts: Contact[]
  engagements: Engagement[]
  objectives: Objective[]
  events: SimulationEvent[]
}

export type Engagement = {
  id: number
  elapsedSeconds: number
  impactAtSeconds: number
  unitId: string
  unitName: string
  weaponId: string
  weaponName: string
  weaponKind: Weapon['kind']
  contactId: string
  contactLabel: string
  hitChancePercent: number
  damagePercent: number
  disruptionDurationSeconds: number
  damageAppliedPercent: number
  interceptedBy: string | null
  status: 'in_flight' | 'hit' | 'damaged' | 'disrupted' | 'disabled' | 'miss' | 'intercepted' | 'aborted'
  hit: boolean
}

export type CampaignProgress = {
  campaignId: string
  title: string
  description: string
  scenarioOrder: string[]
  activeScenarioId: string
  completedScenarios: string[]
  scenarioResults: ScenarioResult[]
  escalationLevel: number
}

export type ScenarioResult = {
  scenarioId: string
  completedAtSeconds: number
  contactsClassified: number
  targetsDestroyed: number
  engagementsLaunched: number
  hits: number
  damaged: number
  intercepted: number
  cyberDisruptions: number
  spaceStrikes: number
  satellitesDisabled: number
  misses: number
  aborted: number
}

export type Consequence = {
  id: string
  minEscalation: number
  title: string
  description: string
}

export type Coastline = {
  id: string
  name: string
  points: { x: number; z: number }[]
}

export type CampaignSettings = {
  timeScale: number
  layersVisible: boolean
}

export type CampaignStatus = {
  progress: CampaignProgress
  settings: CampaignSettings
  hasSave: boolean
  savedAt: number | null
}

export type CampaignSession = {
  simulation: SimulationState
  status: CampaignStatus
}
