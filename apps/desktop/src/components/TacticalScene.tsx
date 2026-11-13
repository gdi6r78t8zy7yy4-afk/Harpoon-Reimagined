import { useEffect, useRef } from 'react'
import { ArcRotateCamera } from '@babylonjs/core/Cameras/arcRotateCamera'
import { DirectionalLight } from '@babylonjs/core/Lights/directionalLight'
import { HemisphericLight } from '@babylonjs/core/Lights/hemisphericLight'
import { Color3, Color4 } from '@babylonjs/core/Maths/math.color'
import { Vector3 } from '@babylonjs/core/Maths/math.vector'
import { MeshBuilder } from '@babylonjs/core/Meshes/meshBuilder'
import { Mesh } from '@babylonjs/core/Meshes/mesh'
import { StandardMaterial } from '@babylonjs/core/Materials/standardMaterial'
import { Engine } from '@babylonjs/core/Engines/engine'
import { Scene } from '@babylonjs/core/scene'
import type { Contact, Engagement, Unit } from '../types'

type Props = { units: Unit[]; contacts: Contact[]; engagements: Engagement[]; elapsedSeconds: number; selectedId: string; onSelect: (id: string) => void }

function ship(scene: Scene, unit: Unit, selected: boolean) {
  const group = new Mesh(`unit-${unit.id}`, scene)
  group.position.set(unit.x, 0.08, unit.z)
  group.rotation.y = (unit.heading - 90) * Math.PI / 180
  const hull = MeshBuilder.CreateBox(`hull-${unit.id}`, { width: 0.52, height: 0.13, depth: 1.7 }, scene)
  hull.parent = group
  const hullMaterial = new StandardMaterial(`hull-mat-${unit.id}`, scene)
  hullMaterial.diffuseColor = selected ? new Color3(0.55, 0.9, 0.81) : new Color3(0.42, 0.62, 0.62)
  hullMaterial.emissiveColor = selected ? new Color3(0.04, 0.19, 0.15) : new Color3(0.015, 0.045, 0.05)
  hull.material = hullMaterial
  const bridge = MeshBuilder.CreateBox(`bridge-${unit.id}`, { width: 0.32, height: 0.2, depth: 0.43 }, scene)
  bridge.position.set(0, 0.14, -0.1)
  bridge.parent = group
  bridge.material = hullMaterial
  const ring = MeshBuilder.CreateTorus(`ring-${unit.id}`, { diameter: 1.1, thickness: 0.018, tessellation: 48 }, scene)
  ring.position.y = 0.025
  ring.parent = group
  const ringMaterial = new StandardMaterial(`ring-mat-${unit.id}`, scene)
  ringMaterial.emissiveColor = selected ? new Color3(0.28, 0.9, 0.71) : new Color3(0.08, 0.32, 0.29)
  ringMaterial.alpha = selected ? 0.92 : 0.48
  ring.material = ringMaterial
  group.metadata = { unitId: unit.id }
  return group
}

function contactMarker(scene: Scene, contact: Contact, selected: boolean) {
  const marker = MeshBuilder.CreateCylinder(`contact-${contact.id}`, { diameter: 0.58, height: 0.025, tessellation: 4 }, scene)
  marker.position.set(contact.x, 0.08, contact.z)
  marker.rotation.y = Math.PI / 4
  marker.metadata = { contactId: contact.id }
  const material = new StandardMaterial(`contact-mat-${contact.id}`, scene)
  const color = contact.domain === 'air' ? new Color3(0.94, 0.69, 0.34) : contact.domain === 'subsurface' ? new Color3(0.63, 0.56, 0.91) : new Color3(0.98, 0.39, 0.31)
  material.diffuseColor = color
  material.emissiveColor = selected ? color.scale(0.7) : color.scale(0.23)
  material.alpha = 0.9
  marker.material = material
  return marker
}

export function TacticalScene({ units, contacts, engagements, elapsedSeconds, selectedId, onSelect }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const selectRef = useRef(onSelect)
  const sceneRef = useRef<Scene | null>(null)
  selectRef.current = onSelect

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return
    const engine = new Engine(canvas, true, { preserveDrawingBuffer: true, stencil: true, antialias: true })
    const scene = new Scene(engine)
    scene.clearColor = new Color4(0.025, 0.065, 0.075, 1)
    const camera = new ArcRotateCamera('tactical-camera', -Math.PI / 2.3, 1.05, 22, new Vector3(0, 0, 0), scene)
    const zoom = (event: Event) => {
      const factor = (event as CustomEvent<number>).detail
      camera.radius = Math.max(camera.lowerRadiusLimit ?? 9, Math.min(camera.upperRadiusLimit ?? 42, camera.radius * factor))
    }
    const center = () => { camera.target = Vector3.Zero(); camera.radius = 22 }
    window.addEventListener('harpoon:zoom', zoom)
    window.addEventListener('harpoon:center', center)
    camera.lowerRadiusLimit = 9
    camera.upperRadiusLimit = 42
    camera.wheelPrecision = 35
    camera.panningSensibility = 95
    camera.attachControl(canvas, true)
    new HemisphericLight('ambient', new Vector3(0, 1, 0), scene).intensity = 0.75
    const sun = new DirectionalLight('sun', new Vector3(-0.5, -1, 0.3), scene)
    sun.intensity = 0.5

    const water = MeshBuilder.CreateGround('water', { width: 80, height: 80, subdivisions: 2 }, scene)
    const waterMaterial = new StandardMaterial('water-material', scene)
    waterMaterial.diffuseColor = new Color3(0.035, 0.105, 0.12)
    waterMaterial.specularColor = new Color3(0.12, 0.26, 0.28)
    water.material = waterMaterial

    for (let i = -8; i <= 8; i += 1) {
      const horizontal = MeshBuilder.CreateLines(`grid-h-${i}`, { points: [new Vector3(-18, 0.018, i * 2), new Vector3(18, 0.018, i * 2)] }, scene)
      horizontal.color = new Color3(0.13, 0.33, 0.33)
      horizontal.alpha = 0.25
      const vertical = MeshBuilder.CreateLines(`grid-v-${i}`, { points: [new Vector3(i * 2, 0.018, -18), new Vector3(i * 2, 0.018, 18)] }, scene)
      vertical.color = new Color3(0.13, 0.33, 0.33)
      vertical.alpha = 0.25
    }

    sceneRef.current = scene
    scene.onPointerObservable.add((pointer) => {
      if (pointer.type !== 1) return
      const picked = scene.pick(scene.pointerX, scene.pointerY, (mesh) => Boolean(mesh.metadata?.unitId || mesh.metadata?.contactId || (mesh.parent as Mesh | null)?.metadata?.unitId))
      const id = picked?.pickedMesh?.metadata?.unitId ?? picked?.pickedMesh?.metadata?.contactId ?? (picked?.pickedMesh?.parent as Mesh | null)?.metadata?.unitId
      if (typeof id === 'string') selectRef.current(id)
    })
    engine.runRenderLoop(() => scene.render())
    const resize = () => engine.resize()
    window.addEventListener('resize', resize)
    return () => {
      window.removeEventListener('resize', resize)
      window.removeEventListener('harpoon:zoom', zoom)
      window.removeEventListener('harpoon:center', center)
      sceneRef.current = null
      scene.dispose()
      engine.dispose()
    }
  }, [])

  useEffect(() => {
    const scene = sceneRef.current
    if (!scene) return
    for (const unit of units) {
      const root = scene.getMeshByName(`unit-${unit.id}`)
      if (!root) {
        ship(scene, unit, unit.id === selectedId)
        continue
      }
      root.position.set(unit.x, 0.08, unit.z)
      root.rotation.y = (unit.heading - 90) * Math.PI / 180
      const selected = unit.id === selectedId
      root.getChildMeshes().forEach((child) => {
        const material = child.material as StandardMaterial | null
        if (material && child.name.startsWith('ring-')) {
          material.emissiveColor = selected ? new Color3(0.28, 0.9, 0.71) : new Color3(0.08, 0.32, 0.29)
          material.alpha = selected ? 0.92 : 0.48
        } else if (material) {
          material.diffuseColor = selected ? new Color3(0.55, 0.9, 0.81) : new Color3(0.42, 0.62, 0.62)
          material.emissiveColor = selected ? new Color3(0.04, 0.19, 0.15) : new Color3(0.015, 0.045, 0.05)
        }
      })
    }
    for (const contact of contacts) {
      const marker = scene.getMeshByName(`contact-${contact.id}`)
      if (!marker) {
        contactMarker(scene, contact, contact.id === selectedId)
        continue
      }
      const color = contact.domain === 'air' ? new Color3(0.94, 0.69, 0.34) : contact.domain === 'subsurface' ? new Color3(0.63, 0.56, 0.91) : new Color3(0.98, 0.39, 0.31)
      const material = marker.material as StandardMaterial | null
      if (material) material.emissiveColor = contact.id === selectedId ? color.scale(0.7) : color.scale(0.23)
    }
    for (const mesh of scene.meshes.filter((item) => item.name.startsWith('engagement-track-') || item.name.startsWith('engagement-projectile-') || item.name.startsWith('engagement-result-'))) {
      mesh.dispose(false, true)
    }
    for (const engagement of engagements) {
      if (engagement.status !== 'in_flight') continue
      const source = units.find((unit) => unit.id === engagement.unitId)
      const target = contacts.find((contact) => contact.id === engagement.contactId)
      if (!source || !target) continue
      const flightDuration = Math.max(1, engagement.impactAtSeconds - engagement.elapsedSeconds)
      const progress = Math.max(0, Math.min(1, (elapsedSeconds - engagement.elapsedSeconds) / flightDuration))
      const start = new Vector3(source.x, 0.2, source.z)
      const destination = new Vector3(target.x, 0.2, target.z)
      const position = Vector3.Lerp(start, destination, progress)
      const isCyber = engagement.weaponKind === 'cyber'
      const trail = MeshBuilder.CreateLines(`engagement-track-${engagement.id}`, { points: [start, position] }, scene)
      trail.color = isCyber ? new Color3(0.76, 0.55, 0.88) : new Color3(0.3, 0.95, 0.78)
      trail.alpha = 0.9
      trail.isPickable = false
      const projectile = MeshBuilder.CreateSphere(`engagement-projectile-${engagement.id}`, { diameter: 0.25, segments: 8 }, scene)
      projectile.position.copyFrom(position)
      projectile.isPickable = false
      const projectileMaterial = new StandardMaterial(`engagement-projectile-material-${engagement.id}`, scene)
      projectileMaterial.diffuseColor = isCyber ? new Color3(0.71, 0.49, 0.86) : new Color3(0.28, 0.94, 0.76)
      projectileMaterial.emissiveColor = isCyber ? new Color3(0.55, 0.34, 0.72) : new Color3(0.16, 0.72, 0.56)
      projectile.material = projectileMaterial
    }
    const recentOutcomes = engagements.filter((engagement) => engagement.status !== 'in_flight').slice(-4)
    for (const engagement of recentOutcomes) {
      const target = contacts.find((contact) => contact.id === engagement.contactId)
      if (!target) continue
      const palette = {
        hit: new Color3(0.98, 0.31, 0.25),
        damaged: new Color3(0.98, 0.68, 0.28),
        disrupted: new Color3(0.76, 0.55, 0.88),
        intercepted: new Color3(0.35, 0.78, 0.94),
        miss: new Color3(0.58, 0.69, 0.69),
        aborted: new Color3(0.42, 0.5, 0.51),
      } as const
      const color = palette[engagement.status as keyof typeof palette]
      if (!color) continue
      const position = new Vector3(target.x, 0.16, target.z)
      if (engagement.status === 'miss' || engagement.status === 'aborted') {
        const angle = engagement.id * 2.4
        position.x += Math.cos(angle) * 0.9
        position.z += Math.sin(angle) * 0.9
      }
      const scale = engagement.status === 'damaged' ? 1 + engagement.damageAppliedPercent / 200 : 1
      const ring = MeshBuilder.CreateTorus(`engagement-result-ring-${engagement.id}`, { diameter: 1.25 * scale, thickness: 0.045, tessellation: 32 }, scene)
      ring.position.copyFrom(position)
      ring.isPickable = false
      const ringMaterial = new StandardMaterial(`engagement-result-ring-material-${engagement.id}`, scene)
      ringMaterial.diffuseColor = color
      ringMaterial.emissiveColor = color.scale(0.72)
      ring.material = ringMaterial

      if (engagement.status === 'hit') {
        const core = MeshBuilder.CreateSphere(`engagement-result-core-${engagement.id}`, { diameter: 0.28, segments: 8 }, scene)
        core.position.copyFrom(position)
        core.position.y = 0.24
        core.isPickable = false
        const coreMaterial = new StandardMaterial(`engagement-result-core-material-${engagement.id}`, scene)
        coreMaterial.diffuseColor = color
        coreMaterial.emissiveColor = color
        core.material = coreMaterial
      } else if (engagement.status === 'intercepted' || engagement.status === 'miss' || engagement.status === 'aborted') {
        const size = engagement.status === 'intercepted' ? 0.25 : 0.18
        for (const [index, direction] of [[-1, 1], [1, 1]] as const) {
          const line = MeshBuilder.CreateLines(`engagement-result-mark-${engagement.id}-${index}`, {
            points: [
              new Vector3(position.x - size, 0.25, position.z - size * direction),
              new Vector3(position.x + size, 0.25, position.z + size * direction),
            ],
          }, scene)
          line.color = color
          line.isPickable = false
        }
      }
    }
  }, [units, contacts, engagements, elapsedSeconds, selectedId])

  return <canvas ref={canvasRef} className="tactical-canvas" aria-label="Interactive 3D tactical map" />
}
