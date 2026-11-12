import { invoke } from '@tauri-apps/api/core'
import type { SimulationState } from './types'

declare global {
  interface Window { __TAURI_INTERNALS__?: unknown }
}

const browserState: SimulationState = {
  tick: 0,
  elapsedSeconds: 15_510,
  running: false,
  units: [
    { id: 'ddg-01', name: 'USS Arleigh Burke', className: 'Flight III · DDG', side: 'friendly', status: 'Underway', x: -5, z: 2, heading: 42, speedKnots: 18, health: 100 },
    { id: 'ssn-01', name: 'USS Seawolf', className: 'Seawolf class · SSN', side: 'friendly', status: 'Silent running', x: -8, z: -5, heading: 68, speedKnots: 6, health: 100 },
    { id: 'ffg-01', name: 'HNoMS Fridtjof Nansen', className: 'Fridtjof Nansen · FFG', side: 'friendly', status: 'Screening', x: -1, z: 5, heading: 18, speedKnots: 14, health: 100 },
  ],
  contacts: [
    { id: 'c-101', label: 'UNKNOWN SURFACE', classification: 'Surface contact', confidence: 72, bearing: 284, rangeNm: 38.4, domain: 'surface', x: 5, z: -1.5 },
    { id: 'c-102', label: 'AIR CONTACT', classification: 'Fast mover · probable', confidence: 46, bearing: 31, rangeNm: 62.1, domain: 'air', x: 3, z: -7 },
    { id: 'c-103', label: 'SUBSURFACE', classification: 'Transient acoustic', confidence: 31, bearing: 198, rangeNm: 17.8, domain: 'subsurface', x: 1.5, z: 7 },
  ],
}

const desktop = () => typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

export async function loadState() {
  return desktop() ? invoke<SimulationState>('simulation_state') : structuredClone(browserState)
}

export async function setRunning(running: boolean) {
  if (desktop()) return invoke<SimulationState>('set_simulation_running', { running })
  browserState.running = running
  return structuredClone(browserState)
}

export async function advance(seconds = 60) {
  if (desktop()) return invoke<SimulationState>('advance_simulation', { seconds })
  browserState.tick += 1
  browserState.elapsedSeconds += seconds
  for (const unit of browserState.units) {
    const radians = unit.heading * Math.PI / 180
    const distance = unit.speedKnots * seconds / 3600
    unit.x += Math.sin(radians) * distance
    unit.z -= Math.cos(radians) * distance
  }
  return structuredClone(browserState)
}
