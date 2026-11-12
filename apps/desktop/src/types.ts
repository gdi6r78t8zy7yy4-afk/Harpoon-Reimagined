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
}

export type Sensor = {
  id: string
  name: string
  kind: 'radar' | 'sonar' | 'esm'
  rangeNm: number
  domains: Array<'surface' | 'air' | 'subsurface'>
}

export type Contact = {
  id: string
  label: string
  classification: string
  confidence: number
  bearing: number
  rangeNm: number
  domain: 'surface' | 'air' | 'subsurface'
  x: number
  z: number
  identified: boolean
  lastReportedSeconds: number
  confidenceAtLastReport: number
  rangeUncertaintyNm: number
  bearingUncertaintyDeg: number
  rangeUncertaintyAtLastReport: number
  bearingUncertaintyAtLastReport: number
  reportCount: number
  reportHistory: TrackReport[]
  correlatedContactIds: string[]
}

export type TrackReport = {
  elapsedSeconds: number
  unitId: string
  unitName: string
  sensorId: string
  sensorName: string
  sensorKind: Sensor['kind']
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
  objectives: Objective[]
  events: SimulationEvent[]
}

export type CampaignProgress = {
  campaignId: string
  title: string
  description: string
  scenarioOrder: string[]
  activeScenarioId: string
  completedScenarios: string[]
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
