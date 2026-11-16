<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { inTauri, listModules, ipc, type ModuleDto } from "./lib/ipc";

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
    island: { enabled: boolean; mode: "inApp" | "overlay" | "auto" };
    disabledModules: string[];
  };
  let hostSettings: AppSettings | null = $state(null);

  function loadLocal() {
    theme = (localStorage.getItem("orbit.theme") as ThemeMode) || "dark";
    density = (localStorage.getItem("orbit.density") as Density) || "comfortable";
    accent = localStorage.getItem("orbit.accent") || "#6c8cff";
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
      return true;
    }
    return false;
  }

  async function persistSettings() {
    const merged: AppSettings = {
      ...(hostSettings ?? {
        themeMode: "dark", accent: "#6c8cff", density: "comfortable", reduceMotion: false,
        island: { enabled: true, mode: "auto" }, disabledModules: [],
      }),
      themeMode: theme,
      density,
      accent,
      reduceMotion,
      island: { ...(hostSettings?.island ?? { enabled: true, mode: "auto" }), mode: islandMode },
    };
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

  onMount(async () => {
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

  onDestroy(() => { if (timerHandle) clearInterval(timerHandle); });
</script>

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
      <div class="note">Görsel önizleme — yerel demo durumuyla sürülür, sistem olayları değil. Gerçek olaylar (pano, ekran görüntüsü, zamanlayıcı) Tauri ana bilgisayarından gelir.</div>
      <div class="island-stage">
        <div class="island {islandExpanded ? "expanded" : ""}" role="status" aria-live="polite">
          <div style="display:flex;align-items:center;gap:.5rem">
            <span class="dot"></span>
            <span>{islandMsg}</span>
          </div>
          <button class="btn" onclick={() => (islandExpanded = !islandExpanded)} aria-expanded={islandExpanded}>
            {islandExpanded ? "Daralt" : "Genişlet"}
          </button>
        </div>
      </div>
      <div class="island-actions">
        <button class="btn" onclick={() => demoToast("Ayarlar kaydedildi")}>Toast göster</button>
        <button class="btn" onclick={() => demoToast("Pano kaydedildi (önizleme)")}>Pano bildirimi</button>
        <button class="btn" onclick={() => startDemoTimer(25 * 60, "Odak 25")}>25 dk odak</button>
        <button class="btn" onclick={() => startDemoTimer(10, "Kısa mola (demo 10 sn)")}>10 sn mola</button>
        <button class="btn" onclick={() => { islandMsg = "Orbit hazır"; if (timerHandle) { clearInterval(timerHandle); timerHandle = null; } }}>Sıfırla</button>
      </div>
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
        <label for="motion">Hareketi azalt</label>
        <button class="toggle" id="motion" aria-pressed={reduceMotion} aria-label="Hareketi azalt" onclick={() => { reduceMotion = !reduceMotion; persistSettings(); }}></button>
        <div class="hint">Açıkken geçişler devre dışı kalır.</div>
      </div>
    {/if}
  </main>
</div>
