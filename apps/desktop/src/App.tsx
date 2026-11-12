import { useCallback, useEffect, useMemo, useState } from 'react'
import {
  Activity, Anchor, Bell, ChevronDown, ChevronRight, CircleHelp, Crosshair, Eye,
  Gauge, Globe2, Layers3, LocateFixed, Maximize2, Menu, Minus, MoreHorizontal,
  Pause, Play, Plus, Radio, Search, Settings2, Shield, Ship, SlidersHorizontal,
  Sparkles, Target, Waves, X, Save, FolderOpen,
} from 'lucide-react'
import { TacticalScene } from './components/TacticalScene'
import { advance, campaignStatus, classifyContact, correlateContacts, issueCourse, issueSpeed, loadCampaign, loadScenario, loadState, saveCampaign, sensorSweep, setRunning } from './simulation'
import type { CampaignStatus, Contact, SimulationState, Unit } from './types'

const timeLabel = (seconds: number) => {
  const hours = Math.floor(seconds / 3600).toString().padStart(2, '0')
  const minutes = Math.floor((seconds % 3600) / 60).toString().padStart(2, '0')
  const secs = (seconds % 60).toString().padStart(2, '0')
  return `${hours}:${minutes}:${secs}`
}

const trackAgeSeconds = (contact: Contact, now: number) => Math.max(0, now - contact.lastReportedSeconds)
const trackAgeLabel = (contact: Contact, now: number) => {
  if (contact.identified) return 'IDENTIFIED'
  const age = trackAgeSeconds(contact, now)
  if (age < 600) return 'FRESH'
  if (age < 1800) return `AGING · ${Math.floor(age / 60)}M`
  return `STALE · ${Math.floor(age / 60)}M`
}

function App() {
  const [state, setState] = useState<SimulationState | null>(null)
  const [campaign, setCampaign] = useState<CampaignStatus | null>(null)
  const [selectedId, setSelectedId] = useState('ddg-01')
  const [tab, setTab] = useState<'forces' | 'sensors'>('forces')
  const [scale, setScale] = useState(5)
  const [showLayers, setShowLayers] = useState(true)
  const [notice, setNotice] = useState('')
  const [orderPanel, setOrderPanel] = useState<'course' | 'speed' | 'sensors' | null>(null)

  useEffect(() => {
    void Promise.all([loadState(), campaignStatus()]).then(([initialState, initialCampaign]) => {
      setState(initialState)
      setCampaign(initialCampaign)
      setScale(initialCampaign.settings.timeScale)
      setShowLayers(initialCampaign.settings.layersVisible)
    })
  }, [])

  const tick = useCallback(async (seconds: number) => {
    const next = await advance(seconds)
    setState((current) => current ? { ...next, running: current.running } : next)
  }, [])

  useEffect(() => {
    if (!state?.running) return
    const timer = window.setInterval(() => { void tick(scale * 60) }, 1000)
    return () => window.clearInterval(timer)
  }, [state?.running, scale, tick])

  const selected: Unit | Contact | undefined = useMemo(
    () => state?.units.find((unit) => unit.id === selectedId) ?? state?.contacts.find((contact) => contact.id === selectedId),
    [state, selectedId],
  )

  const runStateAction = async (action: () => Promise<SimulationState>, message: string) => {
    try {
      const next = await action()
      setState(next)
      if (next.missionComplete) setCampaign(await campaignStatus())
      setOrderPanel(null)
      setNotice(message)
    } catch (error) {
      setNotice(error instanceof Error ? error.message : 'Order could not be completed')
    }
    window.setTimeout(() => setNotice(''), 2600)
  }

  const reloadScenario = async () => {
    await runStateAction(() => loadScenario('north-atlantic-intercept'), 'Scenario reloaded')
    setSelectedId('ddg-01')
  }

  const toggleRun = async () => {
    if (!state || state.missionComplete) return
    const next = await setRunning(!state.running)
    setState(next)
    setNotice(next.running ? 'Simulation resumed' : 'Simulation paused')
    window.setTimeout(() => setNotice(''), 2200)
  }

  const saveCampaignNow = async () => {
    try {
      const status = await saveCampaign({ timeScale: scale, layersVisible: showLayers })
      setCampaign(status)
      setNotice('Campaign saved on this Mac')
    } catch (error) {
      setNotice(error instanceof Error ? error.message : 'Campaign save failed')
    }
    window.setTimeout(() => setNotice(''), 2600)
  }

  const loadSavedCampaign = async () => {
    try {
      const loaded = await loadCampaign()
      setState(loaded.simulation)
      setCampaign(loaded.status)
      setScale(loaded.status.settings.timeScale)
      setShowLayers(loaded.status.settings.layersVisible)
      setSelectedId('ddg-01')
      setOrderPanel(null)
      setNotice('Campaign restored')
    } catch (error) {
      setNotice(error instanceof Error ? error.message : 'Campaign could not be loaded')
    }
    window.setTimeout(() => setNotice(''), 2600)
  }

  const savedAtLabel = campaign?.savedAt
    ? new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' }).format(campaign.savedAt)
    : 'No save on this device'

  const zoomMap = (factor: number) => window.dispatchEvent(new CustomEvent('harpoon:zoom', { detail: factor }))



  if (!state) return <div className="boot-screen"><div className="boot-mark"><Anchor size={22} /></div><span>INITIALIZING TACTICAL SYSTEMS</span><div className="boot-line" /></div>

  return (
    <div className="app-shell">
      <header className="topbar">
        <div className="brand-lockup">
          <div className="brand-mark"><Anchor size={19} strokeWidth={1.8} /></div>
          <div className="brand-name"><strong>HARPOON</strong><span>REIMAGINED</span></div>
          <span className="build-tag">PRE-ALPHA</span>
        </div>
        <div className="top-divider" />
        <button className="scenario-select" onClick={() => void reloadScenario()} title="Reload the starter scenario"><span className="scenario-kicker">SCENARIO 01</span><span>{state.missionTitle}</span><ChevronDown size={14} /></button>
        <div className="topbar-spacer" />
        <div className="connection"><span className="connection-dot" />SIMULATION ONLINE</div>
        <button className="icon-button top-icon" aria-label="Notifications"><Bell size={17} /><i /></button>
        <button className="avatar-button" aria-label="Commander profile">C</button>
      </header>

      <main className="command-layout">
        <aside className="left-rail panel">
          <div className="panel-heading"><div><span className="eyebrow">OPERATIONS</span><h1>Command</h1></div><button className="icon-button subtle" aria-label="Collapse menu"><Menu size={17} /></button></div>
          <div className="theatre-card"><div className="theatre-glyph"><Globe2 size={18} /></div><div><span className="eyebrow">AREA OF OPERATIONS</span><strong>{state.theatre}</strong><small>North Atlantic · 64° 12′ N</small></div><ChevronRight size={15} className="muted-icon" /></div>
          <div className="section-row"><span className="eyebrow">TASK GROUP</span><button className="text-button" onClick={() => { setNotice('Force composition is defined by the scenario'); window.setTimeout(() => setNotice(''), 2400) }}><Plus size={13} /> ADD UNIT</button></div>
          <div className="tab-switch"><button className={tab === 'forces' ? 'active' : ''} onClick={() => setTab('forces')}><Ship size={14} /> Forces <span>03</span></button><button className={tab === 'sensors' ? 'active' : ''} onClick={() => setTab('sensors')}><Radio size={14} /> Sensors <span>03</span></button></div>
          <div className="unit-list">
            {tab === 'forces' ? state.units.map((unit, index) => (
              <button key={unit.id} onClick={() => setSelectedId(unit.id)} className={`unit-card ${selectedId === unit.id ? 'selected' : ''}`}>
                <div className="unit-symbol"><Ship size={16} /><span>{String(index + 1).padStart(2, '0')}</span></div>
                <div className="unit-copy"><strong>{unit.name}</strong><span>{unit.className}</span><small><i />{unit.status}</small></div>
                <MoreHorizontal size={16} className="unit-more" />
              </button>
            )) : state.contacts.map((contact) => (
              <button key={contact.id} onClick={() => setSelectedId(contact.id)} className={`unit-card contact-card ${selectedId === contact.id ? 'selected' : ''}`}>
                <div className={`contact-symbol ${contact.domain}`}><Crosshair size={16} /></div>
                <div className="unit-copy"><strong>{contact.label}</strong><span>{contact.classification}</span><small><i />{contact.confidence}% confidence · {trackAgeLabel(contact, state.elapsedSeconds)}</small></div>
              </button>
            ))}
          </div>
          <div className="left-spacer" />
          <div className="threat-summary"><div className="threat-icon"><Activity size={15} /></div><div><span className="eyebrow">THREAT POSTURE</span><strong>Elevated</strong></div><span className="threat-bars"><i /><i /><i /><i /><i /></span></div>
          <div className="profile-row"><div className="profile-avatar">C</div><div><strong>Commander</strong><span>Task Group North</span></div><Settings2 size={16} className="muted-icon" /></div>
        </aside>

        <section className="mission-stage">
          <div className="stage-toolbar"><div className="stage-title"><span className="live-indicator" /><span>TACTICAL PICTURE</span><span className="stage-separator">/</span><span className="stage-muted">NORTH ATLANTIC</span></div><div className="toolbar-actions"><button className="toolbar-button" onClick={() => setTab('sensors')}><Search size={15} /><span>Find</span><kbd>⌘ K</kbd></button><button className="toolbar-button" onClick={() => setShowLayers(!showLayers)}><Layers3 size={15} /><span>Layers</span></button><button className="square-button" aria-label="Expand view"><Maximize2 size={15} /></button></div></div>
          <div className="scene-wrap">
            {state.missionComplete && <div className="mission-complete-banner"><div className="complete-mark"><Shield size={17} /></div><div><span className="eyebrow">SCENARIO COMPLETE</span><strong>Contact picture established</strong><small>Task Group North classified the required contacts.</small></div><button onClick={() => void reloadScenario()}>REPLAY SCENARIO</button></div>}
            <TacticalScene units={state.units} contacts={state.contacts} selectedId={selectedId} onSelect={setSelectedId} />
            <div className="map-vignette" />
            <div className="map-coordinates top-left"><span>64° 12′ 08″ N</span><span>002° 41′ 33″ W</span></div>
            <div className="map-coordinates top-right"><span>THEATRE / GIUK GAP</span><span className="coord-muted">NATO SEA AREA 4A</span></div>
            <div className="range-ring ring-one" /><div className="range-ring ring-two" />
            <div className="bearing-line" />
            <div className="map-label label-west">NORWEGIAN SEA</div><div className="map-label label-east">NORTH ATLANTIC OCEAN</div>
            <div className="north-indicator"><span>N</span><div /></div>
            {showLayers && <div className="layer-legend"><span><i className="legend-friendly" /> FRIENDLY</span><span><i className="legend-surface" /> SURFACE</span><span><i className="legend-air" /> AIR</span><span><i className="legend-sub" /> SUBSURFACE</span></div>}
            <div className="map-controls"><button aria-label="Zoom in" onClick={() => zoomMap(0.8)}><Plus size={15} /></button><button aria-label="Zoom out" onClick={() => zoomMap(1.25)}><Minus size={15} /></button><div /><button aria-label="Center map" onClick={() => window.dispatchEvent(new Event('harpoon:center'))}><LocateFixed size={15} /></button></div>
            <div className="map-scale"><span>0</span><i /><span>25 NM</span></div>
            <div className="scene-caption"><span className="caption-dot" /> LIVE SIMULATION <span>·</span> {state.units.length} FRIENDLY UNITS <span>·</span> {state.contacts.length} CONTACTS</div>
          </div>
          <div className="timeline-bar"><div className="timeline-label"><span className="eyebrow">MISSION TIME</span><strong>{timeLabel(state.elapsedSeconds)}</strong><span className="utc-tag">ZULU</span></div><div className="timeline-track"><div className="timeline-tick active"/><div className="timeline-tick"/><div className="timeline-tick"/><div className="timeline-tick"/><div className="timeline-tick"/><div className="track-line"><i style={{ width: `${Math.min(16 + state.tick * 2, 74)}%` }} /></div><div className="track-labels"><span>START</span><span>PHASE 01</span><span>ENGAGEMENT WINDOW</span><span>+06:00 HRS</span></div></div><button className="timeline-expand"><ChevronDown size={15} /></button></div>
          <div className="event-strip"><div className="event-label"><Activity size={14} /><span>ACTIVITY</span></div>{[...state.events].slice(-2).reverse().map((event) => <div className="event-item" key={event.id}><span className="event-time">{timeLabel(event.elapsedSeconds)}</span><i className={`event-dot ${event.message.startsWith('Contact classified') ? 'mint' : event.source === 'USS SEAWOLF' ? 'violet' : 'amber'}`} /><span>{event.message}</span><span className="event-source">{event.source}</span></div>)}<button className="event-more" aria-label="More activity"><ChevronRight size={15} /></button></div>
        </section>

        <aside className="right-rail panel">
          <div className="right-heading"><div><span className="eyebrow">COMMAND OVERVIEW</span><h2>Situation</h2></div><button className="icon-button subtle" aria-label="More situation options"><MoreHorizontal size={18} /></button></div>
          <div className="readiness-card"><div className="readiness-top"><span className="eyebrow">GROUP READINESS</span><span className="readiness-grade">NOMINAL</span></div><div className="readiness-score"><strong>92</strong><span>%</span><div className="readiness-ring"><svg viewBox="0 0 44 44"><circle cx="22" cy="22" r="18" /><circle className="progress" cx="22" cy="22" r="18" /></svg><Shield size={15} /></div></div><div className="readiness-foot"><span><i /> 3 of 3 units operational</span><span>+2.4%</span></div></div>
          <div className="objective-card">{state.objectives.map((objective) => <div key={objective.id}><div className="objective-heading"><span className="eyebrow">MISSION OBJECTIVE</span><span className={objective.completed ? 'objective-complete' : 'objective-progress'}>{objective.completed ? 'COMPLETE' : `${objective.progress}/${objective.required}`}</span></div><strong>{objective.title}</strong><p>{objective.description}</p><div className="objective-track"><i style={{ width: `${(objective.progress / objective.required) * 100}%` }} /></div></div>)}</div>
          <div className="campaign-card"><div className="campaign-card-heading"><div><span className="eyebrow">ACTIVE CAMPAIGN</span><strong>{campaign?.progress.title ?? 'Northern Watch'}</strong></div><span className="campaign-count">{campaign?.progress.completedScenarios.length ?? 0}/{campaign?.progress.scenarioOrder.length ?? 1}</span></div><div className="campaign-save-state">{campaign?.progress.description ?? ''}</div><div className="campaign-save-state">{campaign?.hasSave ? 'Saved' : 'Unsaved progress'} <span>·</span> {savedAtLabel}</div><div className="campaign-actions"><button onClick={() => void saveCampaignNow()}><Save size={13} /> SAVE</button><button disabled={!campaign?.hasSave} onClick={() => void loadSavedCampaign()}><FolderOpen size={13} /> LOAD</button></div></div>
          <div className="intel-heading"><span className="eyebrow">CONTACT PICTURE</span><button className="icon-button subtle" aria-label="Filter contacts"><SlidersHorizontal size={14} /></button></div>
          <div className="contact-list">
            {state.contacts.map((contact) => <button key={contact.id} className={`intel-card ${selectedId === contact.id ? 'active' : ''}`} onClick={() => setSelectedId(contact.id)}>
              <div className={`intel-glyph ${contact.domain}`}><Crosshair size={17} /></div><div className="intel-main"><div className="intel-title"><strong>{contact.label}</strong><span className={`confidence confidence-${contact.confidence > 60 ? 'high' : contact.confidence > 40 ? 'mid' : 'low'}`}>{contact.confidence}%</span></div><span className="intel-class">{contact.classification}</span><div className="intel-meta"><span>{contact.bearing.toString().padStart(3, '0')}° BRG</span><span>{contact.rangeNm.toFixed(1)} NM</span><span>{contact.domain.toUpperCase()}</span><span className={`track-age ${trackAgeLabel(contact, state.elapsedSeconds).startsWith('STALE') ? 'stale' : ''}`}>{trackAgeLabel(contact, state.elapsedSeconds)}</span></div><div className="intel-uncertainty">ESTIMATE ±{contact.rangeUncertaintyNm.toFixed(1)} NM · ±{Math.round(contact.bearingUncertaintyDeg)}° · {contact.reportCount} REPORTS</div></div>
            </button>)}
          </div>
          <button className="all-contacts" onClick={() => setTab('sensors')}>VIEW ALL CONTACTS <ChevronRight size={14} /></button>
          <div className="selected-heading"><span className="eyebrow">SELECTED ENTITY</span><button className="icon-button subtle" aria-label="Selected entity options"><MoreHorizontal size={17} /></button></div>
          {selected ? <div className="selected-card">
            <div className="selected-card-top"><div className={`selected-icon ${'side' in selected ? 'friendly' : selected.domain}`}>
              {'side' in selected ? <Ship size={20} /> : <Target size={20} />}
            </div><div><strong>{'name' in selected ? selected.name : selected.label}</strong><span>{'className' in selected ? selected.className : selected.classification}</span></div></div>
            <div className="selected-stats"><div><span className="eyebrow">HEADING</span><strong>{'heading' in selected ? `${Math.round(selected.heading).toString().padStart(3, '0')}°` : `${selected.bearing.toString().padStart(3, '0')}°`}</strong></div><div><span className="eyebrow">SPEED</span><strong>{'speedKnots' in selected ? `${selected.speedKnots} <small>KT</small>` : `${selected.rangeNm.toFixed(1)} <small>NM</small>`}</strong></div><div><span className="eyebrow">STATUS</span><strong className="status-ready">{'status' in selected ? selected.status.toUpperCase() : `${selected.confidence}% CONF.`}</strong></div></div>
            {'domain' in selected && <div className="track-management">
              <div className="track-sources"><span className="eyebrow">RECENT SENSOR REPORTS</span>{selected.reportHistory.length ? selected.reportHistory.slice(-3).reverse().map((report) => <span key={`${report.elapsedSeconds}-${report.sensorId}-${report.unitId}`}>{timeLabel(report.elapsedSeconds)} · {report.sensorName} / {report.unitName}</span>) : <small>No sensor reports yet</small>}</div>
              <div className="track-correlation"><span className="eyebrow">TRACK CORRELATION</span>
                {selected.correlatedContactIds.map((peerId) => {
                  const peer = state.contacts.find((contact) => contact.id === peerId)
                  return peer && <button key={peer.id} disabled={state.missionComplete} onClick={() => void runStateAction(() => correlateContacts(selected.id, peer.id, false), `Separated ${peer.label}`)}>SEPARATE · {peer.label}</button>
                })}
                {state.contacts.filter((contact) => contact.id !== selected.id && !selected.correlatedContactIds.includes(contact.id)).map((peer) => <button key={peer.id} disabled={state.missionComplete} onClick={() => void runStateAction(() => correlateContacts(selected.id, peer.id, true), `Correlated ${peer.label}`)}>CORRELATE · {peer.label}</button>)}
              </div>
            </div>}
            {'health' in selected && <div className="health-row"><span>HULL INTEGRITY</span><div><i style={{ width: `${selected.health}%` }} /></div><strong>{selected.health}%</strong></div>}
            <div className="order-actions">{'speedKnots' in selected ? <><button disabled={state.missionComplete} onClick={() => setOrderPanel(orderPanel === 'course' ? null : 'course')}><LocateFixed size={14} /> COURSE</button><button disabled={state.missionComplete} onClick={() => setOrderPanel(orderPanel === 'speed' ? null : 'speed')}><Gauge size={14} /> SPEED</button><button disabled={state.missionComplete} onClick={() => setOrderPanel(orderPanel === 'sensors' ? null : 'sensors')}><Radio size={14} /> SENSORS</button></> : <><button disabled={selected.identified || state.missionComplete} onClick={() => void runStateAction(() => classifyContact(selected.id, 'hostile'), 'Contact classified as hostile')}><Target size={14} /> {selected.identified ? 'CLASSIFIED' : 'HOSTILE'}</button><button disabled={selected.identified || state.missionComplete} onClick={() => void runStateAction(() => classifyContact(selected.id, 'neutral'), 'Contact classified as neutral')}><Eye size={14} /> NEUTRAL</button></>}</div>
            {orderPanel === 'sensors' && 'speedKnots' in selected && <div className="order-picker"><span className="eyebrow">ONBOARD SENSORS</span><div className="sensor-picker">{selected.sensors.map((sensor) => <button key={sensor.id} disabled={state.missionComplete} onClick={() => void runStateAction(() => sensorSweep(selected.id, sensor.id), `${sensor.name} sweep complete`)}><strong>{sensor.name}</strong><small>{sensor.kind.toUpperCase()} · {sensor.rangeNm} NM · {sensor.domains.join(' / ').toUpperCase()}</small></button>)}</div></div>}
            {orderPanel !== 'sensors' && orderPanel && 'speedKnots' in selected && <div className="order-picker"><span className="eyebrow">{orderPanel === 'course' ? 'SET COURSE' : 'SET SPEED'}</span><div>{orderPanel === 'course' ? [0, 45, 90, 135, 180, 225, 270, 315].map((heading) => <button key={heading} disabled={state.missionComplete} onClick={() => void runStateAction(() => issueCourse(selected.id, heading), `Course set to ${heading.toString().padStart(3, '0')}°`)}>{heading.toString().padStart(3, '0')}°</button>) : [0, 5, 10, 15, 20, 25].map((speed) => <button key={speed} disabled={state.missionComplete} onClick={() => void runStateAction(() => issueSpeed(selected.id, speed), `Speed set to ${speed} kt`)}>{speed} KT</button>)}</div></div>}
          </div> : <div className="empty-selection">Select a unit or contact to inspect.</div>}
          <div className="intel-note"><Sparkles size={14} /><span>Sensor reports fuse into persistent tracks. Estimate uncertainty grows as tracks age.</span></div>
        </aside>
      </main>

      <footer className="sim-footer"><div className="footer-left"><span className="footer-status"><i /> SIMULATION {state.missionComplete ? 'COMPLETE' : state.running ? 'RUNNING' : 'PAUSED'}</span><span className="footer-divider"/><span>BUILD 0.1.0 · PROTOTYPE</span></div><div className="sim-controls"><button className={`play-button ${state.running ? 'is-running' : ''}`} onClick={() => void toggleRun()} disabled={state.missionComplete}>{state.running ? <Pause size={15} fill="currentColor" /> : <Play size={15} fill="currentColor" />}<span>{state.missionComplete ? 'COMPLETE' : state.running ? 'PAUSE' : 'RESUME'}</span></button><div className="speed-control"><Gauge size={14} /><span>TIME SCALE</span>{[1, 5, 15, 30].map((value) => <button key={value} disabled={state.missionComplete} className={scale === value ? 'active' : ''} onClick={() => setScale(value)}>{value}×</button>)}</div><button className="step-button" disabled={state.missionComplete} onClick={() => void tick(60)} title="Advance one minute"><ChevronRight size={15} /><span>+1 MIN</span></button></div><div className="footer-right"><Waves size={14} /><span>SEA STATE 3</span><span className="footer-divider"/><span>WIND 12 KT NW</span><button className="help-button" aria-label="Help"><CircleHelp size={16} /></button></div></footer>
      {notice && <div className="toast"><span><i />{notice}</span><button onClick={() => setNotice('')} aria-label="Dismiss"><X size={14} /></button></div>}
    </div>
  )
}

export default App
