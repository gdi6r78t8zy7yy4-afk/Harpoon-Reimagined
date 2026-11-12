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
