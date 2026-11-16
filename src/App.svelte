<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import {
    inTauri, listModules, ipc, islandSnapshot, islandDropFiles, islandClearDroppedFiles, islandStartTimer,
    islandMediaToggle, islandMediaNext, islandMediaPrev, islandSetMediaVolume,
    type ModuleDto, type IslandSnapshot, type DownloadTask, type SystemStats,
  } from "./lib/ipc";
  import IslandWindow from "./IslandWindow.svelte";

  // ---------- navigation ----------
  type Tab = "dashboard" | "modules" | "island" | "permissions" | "capabilities" | "settings";
  let tab: Tab = $state("dashboard");

  const NAV: { id: Tab; label: string; icon: string }[] = [
    { id: "dashboard", label: "Pano", icon: "◉" },
    { id: "modules", label: "Modüller", icon: "▦" },
    { id: "island", label: "Orbit Island", icon: "⬭" },
    { id: "permissions", label: "İzinler", icon: "⚿" },
    { id: "capabilities", label: "Yetenekler", icon: "⚙" },
    { id: "settings", label: "Ayarlar", icon: "✦" },
  ];

  // ---------- theme / density / accent ----------
  type ThemeMode = "dark" | "light" | "system";
  type Density = "compact" | "comfortable";
  let theme: ThemeMode = $state("dark");
  let density: Density = $state("comfortable");
  let accent = $state("#6c8cff");
  let reduceMotion = $state(false);
  let islandMode: "inApp" | "overlay" | "auto" = $state("auto");
  let systemStatsEnabled = $state(true);
  const ACCENTS = ["#6c8cff", "#8a5cf6", "#3ddc84", "#ffc857", "#ff8fa3", "#38bdf8"];

  function resolvedTheme(): "dark" | "light" {
    if (theme === "system") {
      return window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
    }
    return theme;
  }

  function applyTheme() {
    const root = document.documentElement;
    root.dataset.theme = resolvedTheme();
    root.dataset.density = density;
    root.style.setProperty("--accent", accent);
    root.style.setProperty("--accent-contrast", "#0b0d14");
    localStorage.setItem("orbit.theme", theme);
    localStorage.setItem("orbit.density", density);
    localStorage.setItem("orbit.accent", accent);
  }

  $effect(applyTheme);

  // ---------- host data ----------
  let liveHost = $state(false);
  let modules: ModuleDto[] = $state([]);
  let modulesLoading = $state(true);
  let hostNote = $state("");

  type PermDto = { id: string; granted: boolean; label: string; description: string };
  let permissions: PermDto[] = $state([]);
  let caps: { platform: string; capabilities: { capability: string; state: string; note: string | null; usable: boolean }[] } | null = $state(null);

  // Preview-only fallbacks (Tauri host absent). Clearly marked; nothing is faked as live.
  const PREVIEW_PERMS: PermDto[] = [
    { id: "clipboard_read", granted: false, label: "Panoyu okuma", description: "Panoda saklanan metinleri ve görselleri okur." },
    { id: "clipboard_monitor", granted: false, label: "Panoyu izleme", description: "Pano değişikliklerini izler." },
    { id: "screen_capture", granted: false, label: "Ekran görüntüsü", description: "Ekranınızdan görüntü yakalar." },
    { id: "keyboard_listen", granted: false, label: "Klavyeyi dinleme", description: "Tuş kimliklerini dinler (yazılan metin değil)." },
    { id: "process_metrics", granted: false, label: "Süreç metrikleri", description: "CPU/RAM süreç istatistiklerini okur." },
    { id: "window_management", granted: false, label: "Pencere yönetimi", description: "Pencere konumu ve boyutunu değiştirir." },
    { id: "file_system_watch", granted: false, label: "Dosya izleme", description: "İzin verilen dizinleri izler." },
    { id: "network", granted: false, label: "Ağ", description: "Ağ erişimi gerektiren özellikler için." },
    { id: "autostart", granted: false, label: "Otomatik başlatma", description: "Oturumla birlikte başlar." },
    { id: "theme_control", granted: false, label: "Tema kontrolü", description: "Sistem temasını değiştirir." },
  ];
  const PREVIEW_CAPS = [
    { capability: "IslandOverlayWindow", state: "limited", note: "KDE/X11: best-effort; Wayland/GNOME: garantili değil — uygulama içi panel kullanılır.", usable: true },
    { capability: "GlobalHotkeys", state: "limited", note: "KDE ve çoğu compositor'da çalışır; garanti yok.", usable: true },
    { capability: "ScreenshotCapture", state: "limited", note: "Wayland'de portal (izinli) yolu; yakalama portal üzerinden.", usable: true },
    { capability: "WindowManagement", state: "unsupported", note: "Wayland'de pencere yerleşimi compositor'a aittir.", usable: false },
    { capability: "SystemMetrics", state: "supported", note: "/proc ve sistem API'leri üzerinden.", usable: true },
  ];

  type AppSettings = {
    themeMode: string; accent: string; density: string; reduceMotion: boolean;
    island: { enabled: boolean; mode: "inApp" | "overlay" | "auto"; system_stats?: boolean };
    disabledModules: string[];
  };
  let hostSettings: AppSettings | null = $state(null);

  function loadLocal() {
    theme = (localStorage.getItem("orbit.theme") as ThemeMode) || "dark";
    density = (localStorage.getItem("orbit.density") as Density) || "comfortable";
    accent = localStorage.getItem("orbit.accent") || "#6c8cff";
    systemStatsEnabled = localStorage.getItem("orbit.system_stats") !== "false";
  }

  async function hydrateFromHost() {
    const s = await ipc<AppSettings>("get_settings");
    if (s) {
      hostSettings = s;
      theme = s.themeMode as ThemeMode;
      density = s.density as Density;
      accent = s.accent;
      reduceMotion = s.reduceMotion;
      islandMode = s.island.mode;
      systemStatsEnabled = s.island.system_stats ?? true;
      return true;
    }
    return false;
  }

  async function persistSettings() {
    const merged: AppSettings = {
      ...(hostSettings ?? {
        themeMode: "dark", accent: "#6c8cff", density: "comfortable", reduceMotion: false,
        island: { enabled: true, mode: "auto", system_stats: systemStatsEnabled }, disabledModules: [],
      }),
      themeMode: theme,
      density,
      accent,
      reduceMotion,
      island: { ...(hostSettings?.island ?? { enabled: true, mode: "auto" }), mode: islandMode, system_stats: systemStatsEnabled },
    };
    localStorage.setItem("orbit.system_stats", String(systemStatsEnabled));
    hostSettings = merged;
    if (inTauri) {
      const saved = await ipc<AppSettings>("save_settings", { settings: merged });
      if (saved) hostSettings = saved;
    }
    applyTheme();
  }

  async function toggleModule(id: string) {
    const disabled = new Set(hostSettings?.disabledModules ?? modules.filter(m => !m.enabled).map(m => m.id));
    if (disabled.has(id)) disabled.delete(id); else disabled.add(id);
    hostSettings = { ...(hostSettings ?? { themeMode: "dark", accent, density, reduceMotion, island: { enabled: true, mode: islandMode }, disabledModules: [] }), disabledModules: [...disabled] };
    if (inTauri) {
      const merged = { ...hostSettings };
      const saved = await ipc<AppSettings>("save_settings", { settings: merged });
      if (saved) hostSettings = saved;
      const res = await listModules();
      modules = res.modules;
    } else {
      modules = modules.map(m => (m.id === id ? { ...m, enabled: !m.enabled } : m));
    }
  }

  async function togglePerm(id: string, granted: boolean) {
    if (inTauri) {
      const updated = await ipc<PermDto>("set_permission", { id, granted: !granted });
      if (updated) permissions = permissions.map(p => (p.id === id ? updated : p));
    } else {
      permissions = permissions.map(p => (p.id === id ? { ...p, granted: !p.granted } : p));
    }
  }

  let enabledModules = $derived(modules.filter(m => m.enabled).length);
  let implementedModules = $derived(modules.filter(m => m.implemented).length);
  let grantedPerms = $derived(permissions.filter(p => p.granted).length);

  // True when this document is being shown inside the island overlay window.
  let isIslandWindow = $state(false);

  onMount(async () => {
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      if (getCurrentWindow().label === "island") { isIslandWindow = true; return; }
    } catch {
      // plain browser — fall through to the dashboard
    }
    loadLocal();
    const hosted = await hydrateFromHost();
    liveHost = hosted && inTauri;
    if (hosted) {
      hostNote = "Tauri kabuğu algılandı — listelenen veriler kabuğun gerçek kayıtlarıdır.";
    } else {
      hostNote = "Tarayıcı önizlemesi — veriler örnek/statiktir; canlı sistem durumu gösterilmez. Gerçek durum için Tauri uygulamasını çalıştırın.";
    }
    const res = await listModules();
    modules = res.modules;
    modulesLoading = false;

    clockHandle = setInterval(() => (now = new Date()), 1000);
    window.addEventListener("keydown", onKeydown);
    if (inTauri) {
      const first = await islandSnapshot();
      if (first) snapshot = first;
      pollHandle = setInterval(async () => { const s = await islandSnapshot(); if (s) snapshot = s; }, 2000);
      try {
        const { listen } = await import("@tauri-apps/api/event");
        unlistenChanged = await listen<IslandSnapshot>("orbit://island-changed", e => { snapshot = e.payload; });
      } catch (err) { console.error("[orbit] island-changed listen failed:", err); }
      try {
        const { getCurrentWebview } = await import("@tauri-apps/api/webview");
        unlistenDrop = await getCurrentWebview().onDragDropEvent(async event => {
          if (event.payload.type === "drop") {
            const res = await islandDropFiles(event.payload.paths);
            if (res) snapshot = res;
          }
        });
      } catch (err) { console.error("[orbit] drag-drop listener failed:", err); }
    }
    if (hosted) {
      permissions = (await ipc<PermDto[]>("list_permissions")) ?? PREVIEW_PERMS;
      caps = await ipc("get_capabilities");
    } else {
      permissions = PREVIEW_PERMS;
      caps = { platform: "Tarayıcı (bilinmiyor)", capabilities: PREVIEW_CAPS };
    }
  });

  // ---------- island local demo ----------
  let islandExpanded = $state(false);
  let islandMsg = $state("Orbit hazır");
  let timerRemaining = $state(0);
  let timerLabel = $state("");
  let timerHandle: ReturnType<typeof setInterval> | null = null;

  function demoToast(text: string) {
    islandMsg = text;
    islandExpanded = true;
  }

  function startDemoTimer(secs: number, label: string) {
    if (timerHandle) clearInterval(timerHandle);
    timerRemaining = secs;
    timerLabel = label;
    islandExpanded = true;
    islandMsg = `${label} — ${secs} sn`;
    timerHandle = setInterval(() => {
      timerRemaining -= 1;
      islandMsg = `${timerLabel} — ${timerRemaining} sn`;
      if (timerRemaining <= 0 && timerHandle) {
        clearInterval(timerHandle);
        timerHandle = null;
        islandMsg = `${timerLabel} bitti ✓`;
      }
    }, 1000);
  }

  // ---------- island ambient (host-pushed snapshot; never invented) ----------
  let now = $state(new Date());
  let snapshot = $state<IslandSnapshot | null>(null);
  let droppedPreview: { path: string; bytes: number }[] = $state([]);
  let dropDragOver = $state(false);
  let clockHandle: ReturnType<typeof setInterval> | null = null;
  let pollHandle: ReturnType<typeof setInterval> | null = null;
  let unlistenChanged: (() => void) | null = null;
  let unlistenDrop: (() => void) | null = null;

  // Static sample, browser preview only — clearly labeled "önizleme" in the UI.
  const PREVIEW_NOW_PLAYING = { player: "preview", title: "Blinding Lights", artist: "The Weeknd", position_secs: 74, length_secs: 200, playing: true, volume: 0.62 };
  const PREVIEW_SYSTEM: SystemStats = { cpu_percent: 12, ram_percent: 34, ram_used_mb: 5400, ram_total_mb: 16000 };
  const PREVIEW_DOWNLOADS: DownloadTask[] = [
    { id: "preview-1", label: "proje-sunumu.pdf", received_bytes: 4_600_000, total_bytes: 8_400_000, done: false },
    { id: "preview-2", label: "orbit-logo.svg", received_bytes: 92_000, total_bytes: 92_000, done: true },
  ];

  let nowPlaying = $derived(liveHost ? (snapshot?.now_playing ?? null) : PREVIEW_NOW_PLAYING);
  let downloads = $derived(liveHost ? (snapshot?.downloads ?? []) : PREVIEW_DOWNLOADS);
  let droppedFiles = $derived(liveHost ? (snapshot?.dropped_files ?? []) : droppedPreview);
  let systemStats = $derived(liveHost ? (snapshot?.system ?? null) : (systemStatsEnabled ? PREVIEW_SYSTEM : null));
  // Volume display: the player's value unless the user just dragged the slider;
  // the host pushes MPRIS Volume on its 2 s poll anyway.
  let volumeOverride = $state<number | null>(null);
  let displayedVolume = $derived(volumeOverride ?? nowPlaying?.volume ?? null);

  function fmtBytes(b: number): string {
    if (b < 1024) return `${b} B`;
    if (b < 1024 * 1024) return `${(b / 1024).toFixed(1)} KB`;
    if (b < 1024 ** 3) return `${(b / 1024 / 1024).toFixed(1)} MB`;
    return `${(b / 1024 ** 3).toFixed(2)} GB`;
  }

  function fmtDuration(secs: number): string {
    const m = Math.floor(secs / 60), s = Math.floor(secs % 60);
    return `${m}:${s.toString().padStart(2, "0")}`;
  }

  async function hostTimer(mins: number) {
    if (liveHost) {
      const res = await islandStartTimer(`Odak ${mins}`, mins * 60);
      if (res) { snapshot = res; islandExpanded = true; return; }
    }
    startDemoTimer(mins * 60, `Odak ${mins}`);
  }

  function onDropZoneDrop(e: DragEvent) {
    e.preventDefault();
    dropDragOver = false;
    if (!e.dataTransfer) return;
    const files = [...e.dataTransfer.files];
    droppedPreview = [...droppedPreview, ...files.map(f => ({ path: f.name, bytes: f.size }))].slice(0, 16);
    islandMsg = `${files.length} dosya (önizleme — hiçbir şey kopyalanmaz)`;
    islandExpanded = true;
  }

  async function clearDropped() {
    if (liveHost) {
      const res = await islandClearDroppedFiles();
      if (res) snapshot = res;
    }
    droppedPreview = [];
  }

  function onKeydown(e: KeyboardEvent) {
    const t = e.target as HTMLElement | null;
    if (t && (t.tagName === "INPUT" || t.tagName === "TEXTAREA" || t.tagName === "SELECT" || t.isContentEditable)) return;
    if (e.ctrlKey && e.code === "Space") {
      e.preventDefault();
      if (tab !== "island") tab = "island"; else islandExpanded = !islandExpanded;
      return;
    }
    if (tab !== "island") return;
    if (e.key === "Escape") islandExpanded = false;
    else if (e.key === "1") hostTimer(25);
    else if (e.key === "2") hostTimer(10);
    else if (e.key === "3") hostTimer(5);
    else if (e.key === "4") hostTimer(60);
    else if (e.key === "5") hostTimer(120);
  }

  onDestroy(() => {
    if (timerHandle) clearInterval(timerHandle);
    if (clockHandle) clearInterval(clockHandle);
    if (pollHandle) clearInterval(pollHandle);
    window.removeEventListener("keydown", onKeydown);
    unlistenChanged?.();
    unlistenDrop?.();
  });
</script>

{#if isIslandWindow}
  <IslandWindow />
{:else}
<div class="shell">
  <aside class="sidebar">
    <div class="brand">
      <svg width="34" height="34" viewBox="0 0 64 64" aria-hidden="true">
        <defs>
          <linearGradient id="orbitmark" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0" stop-color="#6c8cff"/><stop offset="1" stop-color="#8a5cf6"/>
          </linearGradient>
        </defs>
        <rect x="4" y="4" width="56" height="56" rx="14" fill="url(#orbitmark)"/>
        <ellipse cx="32" cy="32" rx="19" ry="7.5" fill="none" stroke="#fff" stroke-width="4" stroke-opacity="0.95" transform="rotate(-24 32 32)"/>
        <circle cx="46.5" cy="20.5" r="5" fill="#fff"/>
        <circle cx="32" cy="32" r="3.4" fill="#fff"/>
      </svg>
      <div>
        <div class="name">Orbit<span class="tagline">Your desktop, organized around you.</span></div>
      </div>
    </div>
    <nav class="nav" aria-label="Orbit">
      {#each NAV as item}
        <button aria-current={tab === item.id ? "page" : undefined} onclick={() => (tab = item.id)}>
          <span class="icon" aria-hidden="true">{item.icon}</span>{item.label}
        </button>
      {/each}
    </nav>
    <div class="sidebar-foot">
      <div class="mono">v0.1.0 · {liveHost ? "tauri-host" : "önizleme"}</div>
      <div>Telemetri yok · hesap yok · yerel veri</div>
    </div>
  </aside>

  <main class="main">
    {#if tab === "dashboard"}
      <h1 class="page-title">Pano</h1>
      <p class="page-sub">Orbit ekosistemine genel bakış — dürüst durum, sahte veri yok.</p>
      <div class="note">{hostNote}</div>
      <div class="grid">
        <div class="card">
          <h3>Modüller</h3>
          <div class="mono" style="font-size:1.6rem">{modules.length}</div>
          <p>{implementedModules} canlı (Orbit Island) · geri kalanı Phase 2+ — hiçbiri sahte değil.</p>
        </div>
        <div class="card">
          <h3>Etkin</h3>
          <div class="mono" style="font-size:1.6rem">{enabledModules}</div>
          <p>Yan kapatılabilen modüller ayarlarda saklanır.</p>
        </div>
        <div class="card">
          <h3>İzinler</h3>
          <div class="mono" style="font-size:1.6rem">{grantedPerms} / {permissions.length}</div>
          <p>Hepsi varsayılan kapalı; açık onay olmadan hiçbir özellik çalışmaz.</p>
        </div>
        <div class="card">
          <h3>Orbit Island</h3>
          <div class="mono" style="font-size:1.6rem">{islandExpanded ? "açık" : "kapalı"}</div>
          <p>İmza özellik — Ayarlar’dan modu (inApp/overlay/auto) seçilir.</p>
        </div>
      </div>
    {:else if tab === "modules"}
      <h1 class="page-title">Modüller</h1>
      <p class="page-sub">12 bağımsız repo — Orbit hiçbirini zorunlu kılmaz.</p>
      <div class="note">{hostNote}</div>
      {#if modulesLoading}
        <p>Yükleniyor…</p>
      {:else}
        <div class="grid">
          {#each modules as m}
            <div class="card">
              <h3 style="display:flex;justify-content:space-between;align-items:center;gap:.5rem">
                {m.name}
                <span class="badge {m.implemented ? "live" : "soon"}">{m.implemented ? "live" : "phase " + m.phase}</span>
              </h3>
              <p>{m.description}</p>
              <div style="display:flex;justify-content:space-between;align-items:center;margin-top:.6rem">
                <span class="mono">{m.id}</span>
                <button class="toggle" aria-pressed={m.enabled} aria-label={`${m.name} etkinliği`} onclick={() => toggleModule(m.id)}></button>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    {:else if tab === "island"}
      <h1 class="page-title">Orbit Island</h1>
      <p class="page-sub">Dynamic Island tarzı etkileşimli hap — Apple markası yok, sizin kabuğunuzda.</p>
      <div class="note">{liveHost ? "Kabuğa bağlı — saat, çalınan müzik, indirmeler ve bıraktığınız dosyalar gerçek durumdur (MPRIS / İndirilenler klasörü izlenir)." : "Tarayıcı önizlemesi — saat gerçektir; müzik/indirme satırları örnek ve statiktir, etiketlidir. Gerçek olaylar Tauri kabuğundan gelir."}</div>

      <div class="clock-line mono">{now.toLocaleTimeString("tr-TR", { hour: "2-digit", minute: "2-digit", second: "2-digit" })}</div>
      <div class="island-date">{now.toLocaleDateString("tr-TR", { weekday: "long", day: "numeric", month: "long" })}</div>

      <div class="island-stage">
        <div class="island ambient {islandExpanded ? "expanded" : ""}" role="status" aria-live="polite">
          <div class="island-head">
            <span class="dot"></span>
            <span class="island-msg">{islandMsg}</span>
            <button class="btn" onclick={() => (islandExpanded = !islandExpanded)} aria-expanded={islandExpanded}>
              {islandExpanded ? "Daralt" : "Genişlet"}
            </button>
          </div>

          {#if islandExpanded}
            <div class="ambient-body">
              {#if nowPlaying}
                <div class="ambient-section">
                  <div class="ambient-label">Çalan {liveHost ? "" : "(önizleme)"}</div>
                  <div class="np-title">{nowPlaying.title || "(başlıksız parça)"}</div>
                  <div class="np-meta">{nowPlaying.artist}{nowPlaying.player ? ` · ${nowPlaying.player}` : ""} · {nowPlaying.playing ? "oynuyor" : "duraklatıldı"}</div>
                  {#if nowPlaying.length_secs}
                    <div class="progress"><div class="progress-fill" style="width:{Math.min(100, (nowPlaying.position_secs / nowPlaying.length_secs) * 100)}%"></div></div>
                    <div class="np-meta">{fmtDuration(nowPlaying.position_secs)} / {fmtDuration(nowPlaying.length_secs)}</div>
                  {:else}
                    <div class="np-meta">{fmtDuration(nowPlaying.position_secs)}</div>
                  {/if}
                  <div class="media-row">
                    <button class="media-btn" onclick={() => islandMediaPrev()} disabled={!liveHost} aria-label="Önceki parça" title={liveHost ? "Önceki" : "Medya tuşları yalnızca Tauri kabuğunda çalışır"}>⏮</button>
                    <button class="media-btn media-toggle" onclick={() => islandMediaToggle()} disabled={!liveHost} aria-label={nowPlaying.playing ? "Durdur" : "Oynat"} title={liveHost ? (nowPlaying.playing ? "Durdur/Duraklat" : "Oynat") : "Medya tuşları yalnızca Tauri kabuğunda çalışır"}>{nowPlaying.playing ? "⏸" : "▶"}</button>
                    <button class="media-btn" onclick={() => islandMediaNext()} disabled={!liveHost} aria-label="Sonraki parça" title={liveHost ? "Sonraki" : "Medya tuşları yalnızca Tauri kabuğunda çalışır"}>⏭</button>
                  </div>
                  {#if displayedVolume !== null}
                    <label class="vol-row">
                      <span class="np-meta">Ses</span>
                      <input type="range" min="0" max="100" step="1" value={Math.round((displayedVolume ?? 0) * 100)} disabled={!liveHost}
                        oninput={(e) => { const v = Number((e.currentTarget as HTMLInputElement).value) / 100; volumeOverride = v; islandSetMediaVolume(v); }}
                        aria-label="Medya sesi" />
                      <span class="mono np-meta">{Math.round((displayedVolume ?? 0) * 100)}</span>
                    </label>
                  {:else if !liveHost}
                    <div class="np-meta vol-off">Ses: oyuncu MPRIS üzerinde ses bildirmiyor (önizlemede sabit örnek gösterilmez).</div>
                  {/if}
                </div>
              {/if}

              {#if systemStats}
                <div class="ambient-section">
                  <div class="ambient-label">Sistem {liveHost ? "" : "(önizleme)"}</div>
                  <div class="sys-row"><span class="sys-name">CPU</span><div class="progress"><div class="progress-fill" style="width:{Math.min(100, systemStats.cpu_percent)}%"></div></div><span class="mono np-meta">{systemStats.cpu_percent.toFixed(0)}%</span></div>
                  <div class="sys-row"><span class="sys-name">RAM</span><div class="progress"><div class="progress-fill" style="width:{Math.min(100, systemStats.ram_percent)}%"></div></div><span class="mono np-meta">{systemStats.ram_percent.toFixed(0)}%</span></div>
                  <div class="np-meta">{systemStats.ram_used_mb.toLocaleString("tr-TR")} MB / {systemStats.ram_total_mb.toLocaleString("tr-TR")} MB · yalnızca bu makine, hiçbir yere gönderilmez</div>
                </div>
              {/if}

              {#if downloads.length > 0}
                <div class="ambient-section">
                  <div class="ambient-label">İndirmeler {liveHost ? `· ${snapshot?.downloads_active ?? 0} sürüyor` : "(önizleme)"}</div>
                  {#each downloads as dl (dl.id)}
                    <div class="dl-row">
                      <span class="dl-icon" aria-hidden="true">{dl.done ? "✓" : "⬇"}</span>
                      <div class="dl-body">
                        <div class="dl-label">{dl.label}</div>
                        {#if dl.total_bytes}
                          <div class="progress"><div class="progress-fill" style="width:{Math.min(100, (dl.received_bytes / dl.total_bytes) * 100)}%"></div></div>
                          <div class="np-meta">{fmtBytes(dl.received_bytes)} / {fmtBytes(dl.total_bytes)}{dl.done ? " · tamamlandı" : ""}</div>
                        {:else}
                          <div class="progress progress-indeterminate"><div class="progress-fill"></div></div>
                          <div class="np-meta">{fmtBytes(dl.received_bytes)} alındı · toplam bilinmiyor{dl.done ? " · tamamlandı" : ""}</div>
                        {/if}
                      </div>
                    </div>
                  {/each}
                </div>
              {/if}

              {#if droppedFiles.length > 0}
                <div class="ambient-section">
                  <div class="ambient-label">Bırakılan dosyalar ({droppedFiles.length}) · hiçbir şey taşınmaz</div>
                  {#each droppedFiles as f}
                    <div class="dropped-row">
                      <span class="mono dropped-path" title={f.path}>{f.path.split("/").pop()}</span>
                      <span class="np-meta">{fmtBytes(f.bytes)}</span>
                    </div>
                  {/each}
                  <button class="btn" style="margin-top:.4rem" onclick={clearDropped}>Listeyi temizle</button>
                </div>
              {/if}

              {#if !nowPlaying && downloads.length === 0 && droppedFiles.length === 0 && !systemStats}
                <div class="ambient-section"><div class="np-meta">Şu an içerik yok — çalan müzik yok, süren indirme yok.</div></div>
              {/if}
            </div>
          {/if}
        </div>
      </div>

      <div class="island-actions">
        <button class="btn primary" onclick={() => hostTimer(25)}>25 dk odak <kbd>1</kbd></button>
        <button class="btn" onclick={() => hostTimer(10)}>10 dk <kbd>2</kbd></button>
        <button class="btn" onclick={() => hostTimer(5)}>5 dk <kbd>3</kbd></button>
        <button class="btn" onclick={() => hostTimer(60)}>1 sa <kbd>4</kbd></button>
        <button class="btn" onclick={() => hostTimer(120)}>2 sa <kbd>5</kbd></button>
        <button class="btn" onclick={() => demoToast("Ayarlar kaydedildi")}>Toast göster</button>
        <button class="btn" onclick={() => { islandMsg = "Orbit hazır"; if (timerHandle) { clearInterval(timerHandle); timerHandle = null; } }}>Sıfırla</button>
      </div>

      <div
        class="dropzone {dropDragOver ? "dragover" : ""}"
        role="region"
        aria-label="Dosya bırakma alanı"
        ondragover={e => { e.preventDefault(); dropDragOver = true; }}
        ondragleave={() => (dropDragOver = false)}
        ondrop={onDropZoneDrop}
      >
        <div class="dropzone-inner">
          <div class="dropzone-title">Dosyaları buraya bırakın</div>
          <div class="np-meta">{inTauri ? "Kabuk yolu alır — dosyalar listelenir, asla taşınmaz/kopyalanmaz." : "Önizleme — yalnızca dosya adı ve boyut gösterilir."}</div>
        </div>
      </div>

      <p class="island-hint">Kısayollar: <kbd>Ctrl</kbd>+<kbd>Space</kbd> island'a geçer / daralt-genişlet · <kbd>1</kbd>–<kbd>5</kbd> zamanlayıcı · <kbd>Esc</kbd> daralt</p>
    {:else if tab === "permissions"}
      <h1 class="page-title">İzinler</h1>
      <p class="page-sub">Her izin açıkça sizin açmanız içindir — varsayılan kapalıdır.</p>
      {#if !liveHost}
        <div class="note">Tarayıcı önizlemesi — anahtarlar yalnızca arayüzde değişir; kalıcı kayıt Tauri kabuğuna aittir.</div>
      {/if}
      {#each permissions as p}
        <div class="perm-row">
          <div>
            <strong>{p.label}</strong>
            <small>{p.description}</small>
            <small class="mono">{p.id}</small>
          </div>
          <button class="toggle" aria-pressed={p.granted} aria-label={`${p.label} izni`} onclick={() => togglePerm(p.id, p.granted)}></button>
        </div>
      {/each}
    {:else if tab === "capabilities"}
      <h1 class="page-title">Yetenekler</h1>
      <p class="page-sub">Bu platformda neler dürüstçe çalışır — istek karşılanamazsa “unavailable” denir, sahtesi yok.</p>
      {#if caps}
        <div class="note">Platfom: <strong>{caps.platform}</strong>{!liveHost ? " (tarayıcı tahmini — kabuğun raporu değildir)" : ""}</div>
        <table class="caps">
          <thead><tr><th>Yetenek</th><th>Durum</th><th>Not</th><th>Kullanılabilir</th></tr></thead>
          <tbody>
            {#each caps.capabilities as c}
              <tr>
                <td class="mono">{c.capability}</td>
                <td><span class="badge {c.state === "supported" ? "live" : c.state === "limited" ? "soon" : ""}">{c.state}</span></td>
                <td>{c.note ?? "—"}</td>
                <td>{c.usable ? "evet" : "hayır"}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {:else}
        <p>Yükleniyor…</p>
      {/if}
    {:else if tab === "settings"}
      <h1 class="page-title">Ayarlar</h1>
      <p class="page-sub">Tema, vurgu rengi, yoğunluk ve Island modu.{liveHost ? " Kabukla senkron saklanır." : " Yerel olarak saklanır."}</p>
      <div class="field">
        <label for="theme">Tema</label>
        <div class="segmented" id="theme">
          {#each [["dark", "Karanlık"], ["light", "Aydınlık"], ["system", "Sistem"]] as [val, label]}
            <button aria-pressed={theme === val} onclick={() => { theme = val as ThemeMode; persistSettings(); }}>{label}</button>
          {/each}
        </div>
      </div>
      <div class="field">
        <label for="accent">Vurgu rengi</label>
        <div class="swatches" id="accent">
          {#each ACCENTS as a}
            <button class="swatch" style="background:{a}" aria-pressed={accent === a} aria-label={a} onclick={() => { accent = a; persistSettings(); }}></button>
          {/each}
        </div>
        <div class="hint">Seçili: <span class="mono">{accent}</span></div>
      </div>
      <div class="field">
        <label for="density">Yoğunluk</label>
        <div class="segmented" id="density">
          {#each [["compact", "Sıkışık"], ["comfortable", "Rahat"]] as [val, label]}
            <button aria-pressed={density === val} onclick={() => { density = val as Density; persistSettings(); }}>{label}</button>
          {/each}
        </div>
      </div>
      <div class="field">
        <label for="islandmode">Orbit Island modu</label>
        <select id="islandmode" bind:value={islandMode} onchange={persistSettings}>
          <option value="auto">Otomatik (Windows/macOS overlay, Linux uygulama içi)</option>
          <option value="inApp">Uygulama içi panel</option>
          <option value="overlay">Ayrı overlay (best-effort)</option>
        </select>
        <div class="hint">Wayland/GNOME’da overlay garantili değildir — bu yüzden “Otomatik” uygulama içi panele düşer.</div>
      </div>
      <div class="field">
        <label for="sysstats">CPU/RAM satırı (ada içinde)</label>
        <button class="toggle" id="sysstats" aria-pressed={systemStatsEnabled} aria-label="CPU/RAM satırı" onclick={() => { systemStatsEnabled = !systemStatsEnabled; persistSettings(); }}></button>
        <div class="hint">Adada bu makinenin CPU/RAM örneğini gösterir (/proc üzerinden, cihazdan hiç çıkmaz). Kapandığında kabuk yeniden başlatılınca örnek toplanmayı durdurur.</div>
      </div>
      <div class="field">
        <label for="motion">Hareketi azalt</label>
        <button class="toggle" id="motion" aria-pressed={reduceMotion} aria-label="Hareketi azalt" onclick={() => { reduceMotion = !reduceMotion; persistSettings(); }}></button>
        <div class="hint">Açıkken geçişler devre dışı kalır.</div>
      </div>
    {/if}
  </main>
</div>
{/if}
