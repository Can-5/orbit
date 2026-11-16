<script lang="ts">
  // Floating island window: transparent viewport, black pill, always on top.
  // Renders nothing but the pill. The main window keeps the full dashboard.
  import { onMount, onDestroy } from "svelte";
  import {
    islandSnapshot, islandDropFiles, islandClearDroppedFiles, islandStartTimer,
    islandMediaToggle, islandMediaNext, islandMediaPrev, islandSetMediaVolume,
    type IslandSnapshot, type DownloadTask, type SystemStats,
  } from "./lib/ipc";

  let now = $state(new Date());
  let snapshot = $state<IslandSnapshot | null>(null);
  let expanded = $state(false);
  let volumeOverride = $state<number | null>(null);
  let dropDragOver = $state(false);

  let clockHandle: ReturnType<typeof setInterval> | null = null;
  let pollHandle: ReturnType<typeof setInterval> | null = null;
  let unlistenChanged: (() => void) | null = null;
  let unlistenDrop: (() => void) | null = null;

  let nowPlaying = $derived(snapshot?.now_playing ?? null);
  let downloads = $derived<DownloadTask[]>(snapshot?.downloads ?? []);
  let droppedFiles = $derived(snapshot?.dropped_files ?? []);
  let systemStats = $derived<SystemStats | null>(snapshot?.system ?? null);
  let timerLabel = $derived(snapshot?.timer?.label ?? null);
  let timerRemaining = $derived(snapshot?.timer_remaining_secs ?? null);
  let volume = $derived(volumeOverride ?? nowPlaying?.volume ?? null);

  const fmtBytes = (n: number) => {
    if (n < 1024) return `${n} B`;
    const units = ["KB", "MB", "GB"];
    let v = n / 1024, i = 0;
    while (v >= 1024 && i < units.length - 1) { v /= 1024; i++; }
    return `${v.toFixed(v >= 10 ? 0 : 1)} ${units[i]}`;
  };
  const fmtDuration = (s: number) => `${Math.floor(s / 60)}:${String(Math.floor(s % 60)).padStart(2, "0")}`;

  async function applySize() {
    try {
      const { getCurrentWindow, LogicalSize } = await import("@tauri-apps/api/window");
      await getCurrentWindow().setSize(new LogicalSize(460, expanded ? 430 : 48));
    } catch (err) {
      console.error("[island] setSize failed:", err);
    }
  }

  function toggleExpanded() {
    expanded = !expanded;
    void applySize();
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") { expanded = false; void applySize(); }
  }

  onMount(async () => {
    document.documentElement.classList.add("island-window");
    document.body.classList.add("island-window");
    clockHandle = setInterval(() => (now = new Date()), 1000);
    window.addEventListener("keydown", onKeydown);
    const first = await islandSnapshot();
    if (first) snapshot = first;
    pollHandle = setInterval(async () => {
      const s = await islandSnapshot();
      if (s) snapshot = s;
    }, 2000);
    try {
      const { listen } = await import("@tauri-apps/api/event");
      unlistenChanged = await listen<IslandSnapshot>("orbit://island-changed", e => { snapshot = e.payload; });
    } catch (err) { console.error("[island] listen failed:", err); }
    try {
      const { getCurrentWebview } = await import("@tauri-apps/api/webview");
      unlistenDrop = await getCurrentWebview().onDragDropEvent(async event => {
        if (event.payload.type === "drop") {
          const res = await islandDropFiles(event.payload.paths);
          if (res) snapshot = res;
          expanded = true;
          void applySize();
        }
      });
    } catch (err) { console.error("[island] drag-drop listener failed:", err); }
    await applySize();
  });

  onDestroy(() => {
    if (clockHandle) clearInterval(clockHandle);
    if (pollHandle) clearInterval(pollHandle);
    window.removeEventListener("keydown", onKeydown);
    unlistenChanged?.();
    unlistenDrop?.();
    document.documentElement.classList.remove("island-window");
    document.body.classList.remove("island-window");
  });
</script>

<div class="pill-stage" role="region" aria-label="Orbit Island">
  <div class="pill {expanded ? "expanded" : ""}" role="status" aria-live="polite">
    <button class="pill-head" onclick={toggleExpanded} aria-expanded={expanded} aria-label="Orbit Island — genişlet/daralt">
      <span class="dot"></span>
      <span class="pill-clock mono">{now.toLocaleTimeString("tr-TR", { hour: "2-digit", minute: "2-digit" })}</span>
      {#if nowPlaying}
        <span class="pill-track">{nowPlaying.playing ? "♪" : "❚❚"} {nowPlaying.title}{nowPlaying.artist ? ` — ${nowPlaying.artist}` : ""}</span>
      {:else if timerRemaining !== null}
        <span class="pill-track">{timerLabel ?? "Zamanlayıcı"} · {fmtDuration(timerRemaining)}</span>
      {:else}
        <span class="pill-track pill-track-dim">Orbit</span>
      {/if}
      <span class="pill-toggle" aria-hidden="true">{expanded ? "▾" : "▴"}</span>
    </button>

    {#if expanded}
      <div class="pill-body">
        {#if nowPlaying}
          <div class="ambient-section">
            <div class="ambient-label">Çalan</div>
            <div class="np-title">{nowPlaying.title || "(başlıksız parça)"}</div>
            <div class="np-meta">{nowPlaying.artist}{nowPlaying.player ? ` · ${nowPlaying.player}` : ""} · {nowPlaying.playing ? "oynuyor" : "duraklatıldı"}</div>
            {#if nowPlaying.length_secs}
              <div class="progress"><div class="progress-fill" style="width:{Math.min(100, (nowPlaying.position_secs / nowPlaying.length_secs) * 100)}%"></div></div>
              <div class="np-meta">{fmtDuration(nowPlaying.position_secs)} / {fmtDuration(nowPlaying.length_secs)}</div>
            {/if}
            <div class="media-row">
              <button class="media-btn" onclick={() => islandMediaPrev()} aria-label="Önceki parça">⏮</button>
              <button class="media-btn media-toggle" onclick={() => islandMediaToggle()} aria-label={nowPlaying.playing ? "Durdur" : "Oynat"}>{nowPlaying.playing ? "⏸" : "▶"}</button>
              <button class="media-btn" onclick={() => islandMediaNext()} aria-label="Sonraki parça">⏭</button>
            </div>
            {#if volume !== null}
              <label class="vol-row">
                <span class="np-meta">Ses</span>
                <input type="range" min="0" max="100" step="1" value={Math.round(volume * 100)}
                  oninput={(e) => { const v = Number((e.currentTarget as HTMLInputElement).value) / 100; volumeOverride = v; islandSetMediaVolume(v); }}
                  aria-label="Medya sesi" />
                <span class="mono np-meta">{Math.round(volume * 100)}</span>
              </label>
            {/if}
          </div>
        {/if}

        {#if systemStats}
          <div class="ambient-section">
            <div class="ambient-label">Sistem</div>
            <div class="sys-row"><span class="sys-name">CPU</span><div class="progress"><div class="progress-fill" style="width:{Math.min(100, systemStats.cpu_percent)}%"></div></div><span class="mono np-meta">{systemStats.cpu_percent.toFixed(0)}%</span></div>
            <div class="sys-row"><span class="sys-name">RAM</span><div class="progress"><div class="progress-fill" style="width:{Math.min(100, systemStats.ram_percent)}%"></div></div><span class="mono np-meta">{systemStats.ram_percent.toFixed(0)}%</span></div>
            <div class="np-meta">{systemStats.ram_used_mb.toLocaleString("tr-TR")} MB / {systemStats.ram_total_mb.toLocaleString("tr-TR")} MB · yalnızca bu makine</div>
          </div>
        {/if}

        {#if downloads.length > 0}
          <div class="ambient-section">
            <div class="ambient-label">İndirmeler · {snapshot?.downloads_active ?? 0} sürüyor</div>
            {#each downloads as dl (dl.id)}
              <div class="dl-row">
                <span class="dl-icon" aria-hidden="true">{dl.done ? "✓" : "⬇"}</span>
                <div class="dl-body">
                  <div class="dl-label">{dl.label}</div>
                  {#if dl.total_bytes}
                    <div class="progress"><div class="progress-fill" style="width:{Math.min(100, (dl.received_bytes / dl.total_bytes) * 100)}%"></div></div>
                    <div class="np-meta">{fmtBytes(dl.received_bytes)} / {fmtBytes(dl.total_bytes)}</div>
                  {:else}
                    <div class="progress progress-indeterminate"><div class="progress-fill"></div></div>
                    <div class="np-meta">{fmtBytes(dl.received_bytes)} alındı</div>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        {/if}

        {#if droppedFiles.length > 0}
          <div class="ambient-section">
            <div class="ambient-label">Bırakılan dosyalar ({droppedFiles.length})</div>
            {#each droppedFiles as f}
              <div class="dropped-row">
                <span class="mono dropped-path" title={f.path}>{f.path.split("/").pop()}</span>
                <span class="np-meta">{fmtBytes(f.bytes)}</span>
              </div>
            {/each}
            <button class="btn" style="margin-top:.35rem" onclick={() => islandClearDroppedFiles()}>Listeyi temizle</button>
          </div>
        {/if}

        <div class="ambient-section pill-timers">
          <button class="btn" onclick={() => islandStartTimer("Odak", 25 * 60)}>25 dk</button>
          <button class="btn" onclick={() => islandStartTimer("Mola", 10 * 60)}>10 dk</button>
          <button class="btn" onclick={() => islandStartTimer("Hızlı", 5 * 60)}>5 dk</button>
        </div>

        {#if !nowPlaying && downloads.length === 0 && droppedFiles.length === 0 && !systemStats}
          <div class="ambient-section"><div class="np-meta">Şu an içerik yok — çalan müzik yok, süren indirme yok.</div></div>
        {/if}
      </div>
    {/if}
  </div>
</div>
