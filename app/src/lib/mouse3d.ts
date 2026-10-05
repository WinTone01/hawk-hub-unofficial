// HM220'nin prosedürel 3D modeli (three.js). Ölçüler: 122 × 65,5 × 38 mm (kitapçık);
// üstten gövde profili ürün fotoğrafından ölçüldü (600 × 1115 px, merkez x = 300).
// Birim: 1 = 100 mm. Fare boyu Z ekseninde, ön (tuşlar) +Z yönünde; Y yukarı.
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { ParametricGeometry } from "three/examples/jsm/geometries/ParametricGeometry.js";
import { RoomEnvironment } from "three/examples/jsm/environments/RoomEnvironment.js";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
import { DRACOLoader } from "three/examples/jsm/loaders/DRACOLoader.js";
import { ledFrame } from "./ledClock";
import type { ButtonId } from "./types";

/** Hawk'ın GLB modelindeki parça adları → fare tuşu. */
const PART_BUTTONS: { match: RegExp; button: ButtonId | "side" | "key" }[] = [
  { match: /WHEEL/i, button: "middle" },
  { match: /SIDEKEY/i, button: "side" },
  { match: /^6_KEY0/i, button: "key" }, // sol/sağ ana tuş: x konumuna göre ayrılır
];

let gltfLoader: GLTFLoader | null = null;
const gltfCache = new Map<string, Promise<THREE.Group>>();
function loadGltf(url: string): Promise<THREE.Group> {
  if (!gltfLoader) {
    const draco = new DRACOLoader();
    draco.setDecoderPath(`${import.meta.env.BASE_URL}draco/`);
    gltfLoader = new GLTFLoader();
    gltfLoader.setDRACOLoader(draco);
  }
  let p = gltfCache.get(url);
  if (!p) {
    p = gltfLoader.loadAsync(url).then((g) => g.scene);
    gltfCache.set(url, p);
  }
  return p.then((s) => s.clone(true));
}

const L = 1.22;
const PX = 0.655 / 600; // fotoğraf pikseli → birim

// Önden arkaya (v: 0 → 1) yarı genişlik, fotoğraf pikseli. Yan tuş çıkıntıları hariç.
const HALF_WIDTH: [number, number][] = [
  [0, 0], [0.018, 144], [0.054, 221], [0.09, 254], [0.18, 279], [0.3, 282], [0.42, 283], [0.55, 288],
  [0.65, 296], [0.72, 300], [0.8, 293], [0.88, 260], [0.94, 205], [0.98, 130], [1, 0],
];
// Sırt yüksekliği (birim): önde alçak, avuç kısmında (v ≈ 0.66) en yüksek 0.38.
const RIDGE: [number, number][] = [
  [0, 0.1], [0.05, 0.2], [0.15, 0.25], [0.3, 0.29], [0.45, 0.33], [0.6, 0.37], [0.68, 0.38], [0.8, 0.35],
  [0.9, 0.27], [0.97, 0.17], [1, 0.1],
];
const SIDE_H = 0.09; // dikey yan duvar yüksekliği
const SEAM_V = 487 / 1115; // tuşların arka sınırı
const SLOT_HALF = 38 * PX; // tekerlek yuvası yarı genişliği
const SLOT_V: [number, number] = [0.012, 0.262];
const WHEEL_V = 0.176;

/**
 * Monoton kübik Hermite spline (Fritsch–Carlson): ölçüm noktalarından geçer, aralarda taşma ve
 * dalgalanma yapmaz. Uçlarda eğim korunur; yüzey kesintisiz (C1) olur.
 */
function spline(table: [number, number][]): (v: number) => number {
  const n = table.length;
  const xs = table.map((p) => p[0]);
  const ys = table.map((p) => p[1]);
  const d = xs.slice(0, -1).map((x, i) => (ys[i + 1] - ys[i]) / (xs[i + 1] - x));
  const m = ys.map((_, i) => (i === 0 ? d[0] : i === n - 1 ? d[n - 2] : d[i - 1] * d[i] <= 0 ? 0 : (d[i - 1] + d[i]) / 2));
  for (let i = 0; i < n - 1; i++) {
    if (d[i] === 0) {
      m[i] = m[i + 1] = 0;
      continue;
    }
    const a = m[i] / d[i];
    const b = m[i + 1] / d[i];
    const h = a * a + b * b;
    if (h > 9) {
      const t = 3 / Math.sqrt(h);
      m[i] = t * a * d[i];
      m[i + 1] = t * b * d[i];
    }
  }
  return (v) => {
    v = Math.min(xs[n - 1], Math.max(xs[0], v));
    let i = 0;
    while (i < n - 2 && xs[i + 1] < v) i++;
    const h = xs[i + 1] - xs[i];
    const t = (v - xs[i]) / h;
    const t2 = t * t;
    const t3 = t2 * t;
    return (
      (2 * t3 - 3 * t2 + 1) * ys[i] + (t3 - 2 * t2 + t) * h * m[i] + (-2 * t3 + 3 * t2) * ys[i + 1] + (t3 - t2) * h * m[i + 1]
    );
  };
}

const widthSpline = spline(HALF_WIDTH);
const ridgeSpline = spline(RIDGE);
// Uçlarda (v → 0, 1) gövdenin yuvarlak kapanması için genişliği çember profiline yaklaştır.
export const halfWidth = (v: number) => {
  const tip = Math.min(1, Math.sqrt(Math.min(v, 1 - v) / 0.06));
  return Math.max(0, widthSpline(v)) * PX * (0.35 + 0.65 * tip);
};
const ridge = (v: number) => ridgeSpline(v);
const along = (v: number) => (0.5 - v) * L;

/** Üst kabuk yüksekliği: kenarda SIDE_H, ortada sırt yüksekliği. */
function shellHeight(u: number, v: number): number {
  const k = Math.pow(Math.max(0, 1 - u * u), 0.42);
  return SIDE_H + (ridge(v) - SIDE_H) * k;
}

function shellPoint(u: number, v: number, lift = 0, target = new THREE.Vector3()) {
  return target.set(u * halfWidth(v), shellHeight(u, v) + lift, along(v));
}

export interface MouseLook {
  body: string; // gövde rengi
  ledMode: "off" | "static" | "breathing" | "neon";
  ledColor: string; // ekranda görünen renk
  brightness: number; // 1–8
  speed: number; // 0 (hızlı) – 5 (yavaş)
}

export class Mouse3D {
  private renderer: THREE.WebGLRenderer;
  private scene = new THREE.Scene();
  private camera = new THREE.PerspectiveCamera(28, 1, 0.1, 50);
  private controls: OrbitControls;
  private bodyMat: THREE.MeshPhysicalMaterial;
  private slotMat: THREE.MeshStandardMaterial;
  private glow: THREE.Sprite;
  private led: THREE.PointLight;
  private floorGlow: THREE.Mesh;
  private frame = 0;
  private look: MouseLook;
  private resize: ResizeObserver;
  private visible = true;
  private baseDistance = 1;
  private procedural!: THREE.Group;
  private model: THREE.Object3D | null = null;
  private modelUrl: string | null = null;
  /** Modelde tekerleğin altına yerleştirilen ışıklı yuva malzemesi. */
  private modelLed: THREE.MeshStandardMaterial | null = null;
  /** Tıklanabilir parçalar: mesh → tuş ("side": yan tuşlar, z konumuna göre ileri/geri). */
  private parts = new Map<THREE.Mesh, { button: ButtonId | "side"; center: THREE.Vector3 }>();
  private selected: ButtonId | null = null;
  private hovered: ButtonId | null = null;
  private onSelect: ((b: ButtonId) => void) | null = null;
  private onHover: ((b: ButtonId | null) => void) | null = null;
  private raycaster = new THREE.Raycaster();
  private downAt: [number, number] | null = null;

  private autoRotate: boolean;
  /** Dikey kutularda kameranın ne kadar geri çekileceği (üstten görünümde fare zaten dikey durur). */
  private narrowFactor: number;

  /** `view: "top"`: Hawk Hub'daki ürün görseli gibi üstten, tekerlek yukarıda. */
  constructor(
    private host: HTMLElement,
    look: MouseLook,
    opts: { autoRotate?: boolean; distance?: number; view?: "orbit" | "top" } = {},
  ) {
    this.look = look;
    this.renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true, powerPreference: "high-performance" });
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    this.renderer.outputColorSpace = THREE.SRGBColorSpace;
    this.renderer.toneMapping = THREE.ACESFilmicToneMapping;
    this.renderer.toneMappingExposure = 1.05;
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    host.appendChild(this.renderer.domElement);

    const pmrem = new THREE.PMREMGenerator(this.renderer);
    this.scene.environment = pmrem.fromScene(new RoomEnvironment(), 0.04).texture;
    this.scene.environmentIntensity = 0.75;

    const d = opts.distance ?? 2.6;
    const top = opts.view === "top";
    if (top) this.camera.position.set(0, d * 0.96, -d * 0.3);
    else this.camera.position.set(0.95 * d * 0.55, 0.9 * d * 0.55, 1.15 * d * 0.55);
    this.narrowFactor = top ? 0.62 : 1.15;
    this.autoRotate = opts.autoRotate ?? !top;
    this.controls = new OrbitControls(this.camera, this.renderer.domElement);
    this.controls.target.set(0, 0.14, 0.02);
    this.controls.enableZoom = false;
    this.controls.enablePan = false;
    this.controls.enableDamping = true;
    this.controls.dampingFactor = 0.08;
    this.controls.minPolarAngle = top ? 0 : 0.25;
    this.controls.maxPolarAngle = 1.35;
    this.controls.autoRotate = this.autoRotate;
    this.controls.autoRotateSpeed = 0.7;
    this.controls.addEventListener("start", () => (this.controls.autoRotate = false));
    this.baseDistance = this.camera.position.distanceTo(this.controls.target);

    // Işıklar: anahtar (gölge), dolgu, arka kenar.
    const key = new THREE.DirectionalLight(0xffffff, 1.25);
    key.position.set(1.5, 3, 1.2);
    key.castShadow = true;
    key.shadow.mapSize.set(1024, 1024);
    key.shadow.camera.left = key.shadow.camera.bottom = -1;
    key.shadow.camera.right = key.shadow.camera.top = 1;
    key.shadow.radius = 6;
    this.scene.add(key);
    const rim = new THREE.DirectionalLight(0x9db4ff, 0.9);
    rim.position.set(-2, 1.2, -2.5);
    this.scene.add(rim);
    this.scene.add(new THREE.AmbientLight(0xffffff, 0.15));

    // Malzemeler
    // HM220 mat kaplamalı: düşük clearcoat, yüksek pürüzlülük.
    this.bodyMat = new THREE.MeshPhysicalMaterial({ roughness: 0.62, metalness: 0, clearcoat: 0.12, clearcoatRoughness: 0.7, sheen: 0.3, sheenRoughness: 0.8 });
    const dark = new THREE.MeshStandardMaterial({ color: 0x0b0c0f, roughness: 0.8 });
    this.slotMat = new THREE.MeshStandardMaterial({ color: 0x050506, roughness: 0.6, emissive: 0x000000 });
    const rubber = new THREE.MeshStandardMaterial({ color: 0x15161a, roughness: 0.95 });
    const steel = new THREE.MeshStandardMaterial({ color: 0xb8bcc6, roughness: 0.25, metalness: 1 });

    const mouse = new THREE.Group();

    // Üst kabuk
    const shell = new ParametricGeometry((u, v, t) => shellPoint(u * 2 - 1, v, 0, t), 96, 160);
    shell.computeVertexNormals();
    const shellMesh = new THREE.Mesh(shell, this.bodyMat);
    shellMesh.castShadow = true;
    mouse.add(shellMesh);

    // Yan duvar: çevre boyunca 0 → SIDE_H
    const side = new ParametricGeometry(
      (t, s, target) => {
        const right = t < 0.5;
        const v = right ? t * 2 : 2 - t * 2;
        const u = right ? 1 : -1;
        const inset = 1 - 0.035 * (1 - s); // alta doğru hafif içe çekik
        return target.set(u * halfWidth(v) * inset, s * SIDE_H, along(v));
      },
      240,
      6,
    );
    side.computeVertexNormals();
    const sideMesh = new THREE.Mesh(side, this.bodyMat);
    sideMesh.castShadow = true;
    mouse.add(sideMesh);

    // Alt plaka
    const outline = new THREE.Shape();
    for (let i = 0; i <= 120; i++) {
      const v = i / 120;
      const p = [halfWidth(v), along(v)] as const;
      i === 0 ? outline.moveTo(p[0], p[1]) : outline.lineTo(p[0], p[1]);
    }
    for (let i = 120; i >= 0; i--) outline.lineTo(-halfWidth(i / 120), along(i / 120));
    const bottom = new THREE.Mesh(new THREE.ShapeGeometry(outline), dark);
    bottom.rotation.x = Math.PI / 2;
    bottom.position.y = 0.002;
    mouse.add(bottom);

    // Tekerlek yuvası (LED bu yuvadan parlar)
    const slot = new ParametricGeometry(
      (a, b, t) => {
        const v = SLOT_V[0] + (SLOT_V[1] - SLOT_V[0]) * b;
        const u = ((a * 2 - 1) * SLOT_HALF) / halfWidth(v);
        return shellPoint(u, v, 0.0025, t);
      },
      8,
      40,
    );
    slot.computeVertexNormals();
    mouse.add(new THREE.Mesh(slot, this.slotMat));

    // Tekerlek: kauçuk silindir + çelik yanaklar
    const wheelR = 0.075;
    const wheel = new THREE.Group();
    const tire = new THREE.Mesh(new THREE.CylinderGeometry(wheelR, wheelR, 0.05, 40, 1), rubber);
    tire.rotation.z = Math.PI / 2;
    wheel.add(tire);
    for (const sx of [-0.028, 0.028]) {
      const cap = new THREE.Mesh(new THREE.CylinderGeometry(wheelR * 0.97, wheelR * 0.97, 0.006, 40), steel);
      cap.rotation.z = Math.PI / 2;
      cap.position.x = sx;
      wheel.add(cap);
    }
    // Lastik dişleri
    for (let i = 0; i < 24; i++) {
      const tread = new THREE.Mesh(new THREE.BoxGeometry(0.046, 0.006, 0.012), rubber);
      const a = (i / 24) * Math.PI * 2;
      tread.position.set(0, Math.cos(a) * wheelR, Math.sin(a) * wheelR);
      tread.rotation.x = -a;
      wheel.add(tread);
    }
    wheel.position.set(0, shellHeight(0, WHEEL_V) - wheelR + 0.028, along(WHEEL_V));
    wheel.castShadow = true;
    mouse.add(wheel);

    // Ek yerleri: tuşlar arası orta çizgi ve tuşların arka sınırı
    const seamMat = new THREE.LineBasicMaterial({ color: 0x000000, transparent: true, opacity: 0.85 });
    const seamPts = (pts: THREE.Vector3[]) => mouse.add(new THREE.Line(new THREE.BufferGeometry().setFromPoints(pts), seamMat));
    seamPts(Array.from({ length: 30 }, (_, i) => shellPoint(0, SLOT_V[1] + ((SEAM_V - SLOT_V[1]) * i) / 29, 0.002)));
    seamPts(Array.from({ length: 60 }, (_, i) => shellPoint(-1 + (2 * i) / 59, SEAM_V, 0.002)));
    // Kabuk kenarına ince bir çizgi (tuşlar gövdeden ayrı parça)
    seamPts(Array.from({ length: 50 }, (_, i) => shellPoint(-0.985, (SEAM_V * i) / 49, 0.002)));
    seamPts(Array.from({ length: 50 }, (_, i) => shellPoint(0.985, (SEAM_V * i) / 49, 0.002)));

    // Yan tuşlar (sol tarafta)
    for (const [v0, v1] of [[0.3, 0.428], [0.44, 0.585]]) {
      const vm = (v0 + v1) / 2;
      const len = (v1 - v0) * L;
      const btn = new THREE.Mesh(new THREE.CapsuleGeometry(0.014, len - 0.03, 4, 12), this.bodyMat);
      btn.rotation.x = Math.PI / 2;
      btn.position.set(-halfWidth(vm) - 0.006, SIDE_H + 0.05, along(vm));
      btn.castShadow = true;
      mouse.add(btn);
    }

    this.scene.add(mouse);
    this.procedural = mouse;

    // LED: yuvanın üstünde parıltı + nokta ışık + zemine yansıyan halka
    this.glow = new THREE.Sprite(new THREE.SpriteMaterial({ map: radialTexture(), blending: THREE.AdditiveBlending, depthWrite: false, transparent: true }));
    this.glow.scale.set(0.2, 0.36, 1);
    this.glow.position.set(0, shellHeight(0, WHEEL_V) + 0.02, along(WHEEL_V));
    this.scene.add(this.glow);
    this.led = new THREE.PointLight(0xff0000, 0, 0.8, 2);
    this.led.position.set(0, shellHeight(0, WHEEL_V) + 0.06, along(WHEEL_V));
    this.scene.add(this.led);

    const floor = new THREE.Mesh(new THREE.PlaneGeometry(4, 4), new THREE.ShadowMaterial({ opacity: 0.45 }));
    floor.rotation.x = -Math.PI / 2;
    floor.receiveShadow = true;
    // Üstten görünümde zemin gölgesi fareyi dikdörtgen bir leke içinde gösteriyor; gerek yok.
    floor.visible = !top;
    this.scene.add(floor);
    this.floorGlow = new THREE.Mesh(
      new THREE.PlaneGeometry(1.6, 2.2),
      new THREE.MeshBasicMaterial({ map: radialTexture(), blending: THREE.AdditiveBlending, transparent: true, depthWrite: false, opacity: 0.0 }),
    );
    this.floorGlow.rotation.x = -Math.PI / 2;
    this.floorGlow.position.y = 0.001;
    this.floorGlow.visible = !top;
    this.scene.add(this.floorGlow);

    this.setLook(look);
    this.resize = new ResizeObserver(() => this.fit());
    this.resize.observe(host);
    this.fit();
    document.addEventListener("visibilitychange", this.onVisibility);
    this.loop();
  }

  private onVisibility = () => {
    this.visible = !document.hidden;
    if (this.visible) this.loop();
  };

  private fit() {
    const { clientWidth: w, clientHeight: h } = this.host;
    if (!w || !h) return;
    this.renderer.setSize(w, h, false);
    this.camera.aspect = w / h;
    this.camera.updateProjectionMatrix();
    // Dar (dikey) kutularda yatay görüş daralır: fare kırpılmasın diye kamera geri çekilir.
    const dir = this.camera.position.clone().sub(this.controls.target);
    dir.setLength(this.baseDistance * Math.max(1, this.narrowFactor / this.camera.aspect));
    this.camera.position.copy(this.controls.target).add(dir);
  }

  setLook(look: MouseLook) {
    this.look = look;
    const body = new THREE.Color(look.body);
    this.bodyMat.color.copy(body);
    // Açık renkli gövdeler daha mat görünür.
    // Açık renkli gövdeler (beyaz, pembe) ışıkta patlamasın: pozlamayı ve yansımayı düşür.
    const light = body.getHSL({ h: 0, s: 0, l: 0 }).l > 0.6;
    this.bodyMat.roughness = light ? 0.58 : 0.62;
    this.renderer.toneMappingExposure = light ? 0.78 : 1.05;
    this.bodyMat.color.multiplyScalar(light ? 0.88 : 1);
  }

  private loop = () => {
    if (!this.visible || this.frame < 0) return;
    this.frame = requestAnimationFrame(this.loop);
    // Ortam ışığıyla aynı saat (ledClock) → ikisi eş zamanlı nefes alır.
    const f = ledFrame(this.look);
    const intensity = f.intensity;
    const color = new THREE.Color(f.color);

    this.slotMat.emissive.copy(color).multiplyScalar(intensity * 0.9);
    if (this.modelLed) this.modelLed.emissive.copy(color).multiplyScalar(intensity * 2.2);
    (this.glow.material as THREE.SpriteMaterial).color.copy(color);
    this.glow.material.opacity = intensity * 0.55;
    this.led.color.copy(color);
    this.led.intensity = intensity * 0.9;
    (this.floorGlow.material as THREE.MeshBasicMaterial).color.copy(color);
    (this.floorGlow.material as THREE.MeshBasicMaterial).opacity = intensity * 0.18;

    this.controls.update();
    this.renderer.render(this.scene, this.camera);
  };

  /**
   * Hawk'ın resmî GLB modelini yükler (varyant başına ayrı dosya). Yön ve ölçek otomatik:
   * en uzun yatay eksen boy kabul edilir, tekerleğin olduğu uç öne (+Z) çevrilir, boy 1.22 birime ölçeklenir.
   */
  async loadModel(url: string | null) {
    if (url === this.modelUrl) return;
    this.modelUrl = url;
    if (!url) return this.useModel(null);
    try {
      const scene = await loadGltf(url);
      if (this.modelUrl !== url || this.frame < 0) return;
      this.useModel(scene);
    } catch (e) {
      console.warn("3D model yüklenemedi, prosedürel model kullanılıyor:", e);
      this.useModel(null);
    }
  }

  private useModel(scene: THREE.Object3D | null) {
    if (this.model) {
      this.scene.remove(this.model);
      this.model = null;
    }
    this.parts.clear();
    this.modelLed = null;
    this.procedural.visible = !scene;
    if (!scene) {
      this.placeLed(new THREE.Vector3(0, shellHeight(0, WHEEL_V), along(WHEEL_V)));
      return;
    }

    // Yön: model dosyada eğik (≈45°) duruyor. Gövde merkezinden tekerleğe giden doğrultu "ön" kabul
    // edilir ve +Z'ye çevrilir; sonra boy 1.22 birime ölçeklenir, taban y=0'a, merkez orijine alınır.
    const root = new THREE.Group();
    const pivot = new THREE.Group();
    pivot.add(scene);
    root.add(pivot);
    let wheel = null as THREE.Object3D | null;
    scene.traverse((o) => {
      if (!wheel && /WHEEL/i.test(o.name)) wheel = o;
    });
    root.updateMatrixWorld(true);
    const center0 = new THREE.Box3().setFromObject(scene).getCenter(new THREE.Vector3());
    if (wheel) {
      const w0 = new THREE.Box3().setFromObject(wheel).getCenter(new THREE.Vector3());
      pivot.rotation.y = -Math.atan2(w0.x - center0.x, w0.z - center0.z);
    }
    root.updateMatrixWorld(true);
    let box = new THREE.Box3().setFromObject(pivot);
    pivot.scale.setScalar(L / box.getSize(new THREE.Vector3()).z);
    root.updateMatrixWorld(true);
    box = new THREE.Box3().setFromObject(pivot);
    pivot.position.set(-(box.min.x + box.max.x) / 2, -box.min.y, -(box.min.z + box.max.z) / 2);
    root.updateMatrixWorld(true);
    const wheelBox = wheel ? new THREE.Box3().setFromObject(wheel) : null;

    // Gölge, malzeme kopyaları (vurgu için) ve tıklanabilir parçalar
    let sideX = 0; // yan tuşların bulunduğu taraf (en dıştaki yan tuş parçasının x'i)
    const keys: THREE.Mesh[] = [];
    scene.traverse((o) => {
      const m = o as THREE.Mesh;
      if (!m.isMesh) return;
      m.castShadow = true;
      m.material = Array.isArray(m.material) ? m.material.map((x) => x.clone()) : m.material.clone();
      const name = (m.name || m.parent?.name || "") + " " + (m.parent?.name || "");
      const part = PART_BUTTONS.find((p) => p.match.test(m.name) || p.match.test(m.parent?.name ?? "") || p.match.test(name));
      if (!part) return;
      const center = new THREE.Box3().setFromObject(m).getCenter(new THREE.Vector3());
      if (part.button === "key") keys.push(m);
      else this.parts.set(m, { button: part.button, center });
      if (part.button === "side" && Math.abs(center.x) > Math.abs(sideX)) sideX = center.x;
    });
    // Ana tuşlar: yan tuşlarla aynı taraftaki sol tuştur.
    for (const k of keys) {
      const center = new THREE.Box3().setFromObject(k).getCenter(new THREE.Vector3());
      this.parts.set(k, { button: Math.sign(center.x) === Math.sign(sideX || 1) ? "left" : "right", center });
    }

    // LED: tekerlek yuvasında ışıyan ince bir şerit (modelde ayrı LED parçası yok)
    const wb = wheelBox ? new THREE.Box3().setFromObject(wheel!) : null;
    if (wb) {
      const w = wb.getSize(new THREE.Vector3());
      const c = wb.getCenter(new THREE.Vector3());
      this.modelLed = new THREE.MeshStandardMaterial({ color: 0x000000, emissive: 0x000000, transparent: true, opacity: 0.95 });
      // Tekerleğin altında, yuvanın içinde kalan ince bir şerit; ışık tekerleğin kenarlarından sızar.
      const strip = new THREE.Mesh(new THREE.BoxGeometry(w.x * 1.06, w.y * 0.22, w.z * 0.82), this.modelLed);
      strip.position.set(c.x, c.y - w.y * 0.3, c.z);
      root.add(strip);
      this.placeLed(new THREE.Vector3(c.x, wb.max.y, c.z));
    }

    this.model = root;
    if (import.meta.env.DEV) (window as unknown as { __hm220?: Mouse3D }).__hm220 = this;
    this.scene.add(root);
    this.applyHighlight();
  }

  private placeLed(p: THREE.Vector3) {
    this.glow.position.set(p.x, p.y + 0.02, p.z);
    this.led.position.set(p.x, p.y + 0.06, p.z);
    this.floorGlow.position.set(p.x, 0.001, p.z - 0.2);
  }

  /** Tuş seçimini etkinleştirir (yalnızca resmî modelde). */
  setPicking(onSelect: ((b: ButtonId) => void) | null, onHover: ((b: ButtonId | null) => void) | null) {
    this.onSelect = onSelect;
    this.onHover = onHover;
    const el = this.renderer.domElement;
    el.onpointerdown = onSelect ? (e) => (this.downAt = [e.clientX, e.clientY]) : null;
    el.onpointermove = onSelect ? (e) => this.hoverAt(e) : null;
    el.onpointerleave = onSelect ? () => this.setHover(null) : null;
    el.onpointerup = onSelect
      ? (e) => {
          const d = this.downAt;
          this.downAt = null;
          if (d && Math.hypot(e.clientX - d[0], e.clientY - d[1]) < 5) {
            const b = this.pick(e);
            if (b) this.onSelect?.(b);
          }
        }
      : null;
  }

  get pickable() {
    return this.parts.size > 0;
  }

  private pick(e: PointerEvent): ButtonId | null {
    if (!this.parts.size) return null;
    const r = this.renderer.domElement.getBoundingClientRect();
    const ndc = new THREE.Vector2(((e.clientX - r.left) / r.width) * 2 - 1, -((e.clientY - r.top) / r.height) * 2 + 1);
    this.raycaster.setFromCamera(ndc, this.camera);
    const hit = this.raycaster.intersectObjects(this.model ? [this.model] : [], true)[0];
    if (!hit) return null;
    const part = this.parts.get(hit.object as THREE.Mesh);
    if (!part) return null;
    if (part.button === "side") return hit.point.z > part.center.z ? "forward" : "back";
    return part.button;
  }

  private hoverAt(e: PointerEvent) {
    if (this.downAt) return; // sürüklerken vurgulama yok
    this.setHover(this.pick(e));
  }

  private setHover(b: ButtonId | null) {
    if (b === this.hovered) return;
    this.hovered = b;
    this.renderer.domElement.style.cursor = b ? "pointer" : "";
    this.onHover?.(b);
    this.applyHighlight();
  }

  setSelection(selected: ButtonId | null, hovered: ButtonId | null = this.hovered) {
    this.selected = selected;
    this.hovered = hovered;
    this.applyHighlight();
  }

  private applyHighlight() {
    const accent = new THREE.Color("#ff3347");
    for (const [mesh, part] of this.parts) {
      const is = (b: ButtonId | null) =>
        b != null && (part.button === b || (part.button === "side" && (b === "forward" || b === "back")));
      const k = is(this.selected) ? 0.55 : is(this.hovered) ? 0.25 : 0;
      for (const mat of Array.isArray(mesh.material) ? mesh.material : [mesh.material]) {
        const m = mat as THREE.MeshStandardMaterial;
        if (m.emissive) m.emissive.copy(accent).multiplyScalar(k);
      }
    }
  }

  /** Kamerayı başlangıç açısına döndürür. */
  resetView() {
    this.controls.reset();
    this.controls.autoRotate = this.autoRotate;
    this.fit();
  }

  dispose() {
    cancelAnimationFrame(this.frame);
    this.frame = -1;
    document.removeEventListener("visibilitychange", this.onVisibility);
    this.resize.disconnect();
    this.controls.dispose();
    this.scene.traverse((o) => {
      const m = o as THREE.Mesh;
      m.geometry?.dispose();
      const mat = m.material as THREE.Material | THREE.Material[] | undefined;
      (Array.isArray(mat) ? mat : mat ? [mat] : []).forEach((x) => x.dispose());
    });
    this.renderer.dispose();
    this.renderer.domElement.remove();
  }
}

let radial: THREE.CanvasTexture | null = null;
function radialTexture(): THREE.CanvasTexture {
  if (radial) return radial;
  const c = document.createElement("canvas");
  c.width = c.height = 128;
  const g = c.getContext("2d")!;
  const grad = g.createRadialGradient(64, 64, 0, 64, 64, 64);
  grad.addColorStop(0, "rgba(255,255,255,1)");
  grad.addColorStop(0.35, "rgba(255,255,255,0.45)");
  grad.addColorStop(1, "rgba(255,255,255,0)");
  g.fillStyle = grad;
  g.fillRect(0, 0, 128, 128);
  radial = new THREE.CanvasTexture(c);
  radial.colorSpace = THREE.SRGBColorSpace;
  return radial;
}

export function webglAvailable(): boolean {
  try {
    const c = document.createElement("canvas");
    return !!(c.getContext("webgl2") || c.getContext("webgl"));
  } catch {
    return false;
  }
}
