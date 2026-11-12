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
}

export type SimulationState = {
  tick: number
  elapsedSeconds: number
  running: boolean
  units: Unit[]
  contacts: Contact[]
}
