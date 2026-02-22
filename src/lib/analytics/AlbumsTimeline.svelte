<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { debounce } from "lodash-es";
    import audioPlayer from "../player/AudioPlayer";
    import { current } from "../../data/store";
    import { currentThemeObject } from "../../theming/store";

    export let albumsByYear: Record<
        string,
        { data: { id: string; artist: string; displayTitle: string }[] }
    >;

    // ── Types ─────────────────────────────────────────────────────────────────
    type AlbumData = { id: string; artist: string; displayTitle: string };
    type AlbumEntry = [string, { data: AlbumData[] }];

    // ── Derived data ──────────────────────────────────────────────────────────
    let albums: AlbumEntry[] = [];
    let labelYears: number[] = [];
    let minYear = 0;
    let maxYear = 0;
    let yearSpan = 0;

    $: processData(albumsByYear);

    function processData(byYear: typeof albumsByYear) {
        if (!byYear) return;
        albums = Object.entries(byYear).filter(
            ([y]) => y && Number(y) > 1800,
        ) as AlbumEntry[];
        if (!albums.length) return;

        const snapMin = Math.floor(Number(albums[0][0]) / 10) * 10;
        const snapMax =
            Math.ceil(Number(albums[albums.length - 1][0]) / 10) * 10;
        minYear = snapMin;
        maxYear = snapMax;
        yearSpan = snapMax - snapMin;

        labelYears = [];
        for (let y = snapMin; y <= snapMax; y += 10) labelYears.push(y);

        drawCanvas();
    }

    // ── Layout constants ──────────────────────────────────────────────────────
    const PADDING_X = 20;
    const DOT_R = 4;
    const DOT_GAP = 2;
    const LABEL_H = 28; // space below the track for year labels
    const TOP_PAD = 8; // space above tallest dot so it isn't clipped

    // Colors — approximated from original CSS variables
    const C_TRACK = "rgba(255,255,255,0.12)";
    const C_TICK = "rgba(255,255,255,0.2)";
    const C_LABEL = "rgba(255,255,255,0.4)";
    const C_LABEL_EDGE = "rgba(255,255,255,0.85)";

    const C_DOT_FILL = $currentThemeObject["accent-secondary"]; // --accent-secondary
    const C_DOT_STROKE = $currentThemeObject["panel-separator"];
    const C_ACTIVE_FILL = $currentThemeObject["accent"]; // --accent
    const C_ACTIVE_STROKE = $currentThemeObject["accent"];

    // ── Canvas refs ───────────────────────────────────────────────────────────
    let canvas: HTMLCanvasElement;
    let wrapper: HTMLDivElement;

    function xFor(year: number, w: number) {
        return PADDING_X + ((year - minYear) / yearSpan) * (w - PADDING_X * 2);
    }

    function getTrackY(h: number) {
        return h - LABEL_H;
    }

    function drawCanvas() {
        if (!canvas || !albums.length) return;
        const dpr = window.devicePixelRatio || 1;
        const w = canvas.clientWidth;
        const h = canvas.clientHeight;
        if (!w || !h) return;

        canvas.width = w * dpr;
        canvas.height = h * dpr;
        const ctx = canvas.getContext("2d")!;
        ctx.scale(dpr, dpr);
        ctx.clearRect(0, 0, w, h);

        const ty = getTrackY(h);
        const activeId = $current?.song?.albumId;

        // Track line
        ctx.beginPath();
        ctx.moveTo(PADDING_X, ty);
        ctx.lineTo(w - PADDING_X, ty);
        ctx.strokeStyle = C_TRACK;
        ctx.lineWidth = 1.5;
        ctx.stroke();

        // Decade ticks + labels
        ctx.textAlign = "center";
        ctx.textBaseline = "top";
        ctx.font = "12px system-ui, sans-serif";
        for (const year of labelYears) {
            const x = xFor(year, w);
            const isEdge = year === minYear || year === maxYear;

            ctx.beginPath();
            ctx.moveTo(x, ty - 5);
            ctx.lineTo(x, ty + 5);
            ctx.strokeStyle = isEdge ? C_LABEL_EDGE : C_TICK;
            ctx.lineWidth = 1;
            ctx.stroke();

            ctx.fillStyle = isEdge ? C_LABEL_EDGE : C_LABEL;
            ctx.fillText(String(year), x, ty + 8);
        }

        // Album dots — stacked upward from track
        for (const [yearStr, group] of albums) {
            const x = xFor(Number(yearStr), w);
            group.data.forEach((album, i) => {
                const isActive = album.id === activeId;
                const isHovered = hoveredKey === `${yearStr}-${album.id}`;
                const cy = ty - DOT_R - i * (DOT_R * 2 + DOT_GAP);
                const r = isHovered && !isActive ? DOT_R + 2 : DOT_R;

                ctx.beginPath();
                ctx.arc(x, cy, r, 0, Math.PI * 2);
                ctx.fillStyle = isActive
                    ? C_ACTIVE_FILL
                    : isHovered
                      ? "rgba(160,150,230,0.95)"
                      : C_DOT_FILL;
                if (isActive || isHovered) {
                    ctx.shadowColor = isActive
                        ? C_ACTIVE_FILL
                        : "rgba(160,150,230,0.8)";
                    ctx.shadowBlur = isActive ? 7 : 5;
                }
                ctx.fill();
                ctx.shadowBlur = 0;

                ctx.beginPath();
                ctx.arc(x, cy, r, 0, Math.PI * 2);
                ctx.strokeStyle = isActive
                    ? C_ACTIVE_STROKE
                    : isHovered
                      ? "rgba(190,180,255,0.7)"
                      : C_DOT_STROKE;
                ctx.lineWidth = 1.5;
                ctx.stroke();
            });
        }
    }

    // ── Hover state ────────────────────────────────────────────────────────────
    let hoveredKey: string | null = null;

    // ── Custom tooltip ────────────────────────────────────────────────────────
    let tooltip = { visible: false, x: 0, y: 0, text: "" };
    let tooltipEl: HTMLDivElement;

    // Clamp x so the tooltip never overflows the wrapper's edges.
    // tooltipEl is always mounted (visibility:hidden when not shown), so offsetWidth is always valid.
    function clampTooltipX(idealX: number): number {
        if (!tooltipEl || !wrapper) return idealX;
        const half = tooltipEl.offsetWidth / 2;
        const max = wrapper.clientWidth;
        return Math.min(Math.max(idealX, half), max - half);
    }

    function hitTest(mx: number, my: number, w: number, h: number) {
        if (!canvas || !albums.length) return null;
        const ty = getTrackY(h);

        for (const [yearStr, group] of albums) {
            const x = xFor(Number(yearStr), w);
            for (let i = 0; i < group.data.length; i++) {
                const cy = ty - DOT_R - i * (DOT_R * 2 + DOT_GAP);
                if (Math.hypot(mx - x, my - cy) <= DOT_R + 2) {
                    return { album: group.data[i], year: yearStr, cx: x, cy };
                }
            }
        }
        return null;
    }

    function onMouseMove(e: MouseEvent) {
        const rect = canvas.getBoundingClientRect();
        const hit = hitTest(
            e.clientX - rect.left,
            e.clientY - rect.top,
            rect.width,
            rect.height,
        );
        const key = hit ? `${hit.year}-${hit.album.id}` : null;

        if (key !== hoveredKey) {
            hoveredKey = key;
            drawCanvas(); // redraw with hover highlight
        }

        if (hit) {
            canvas.style.cursor = "pointer";
            // Pre-position at idealX so the element has its final text before we measure width
            const idealX = hit.cx;
            tooltip = {
                visible: true,
                x: idealX,
                y: hit.cy + DOT_R + 8,
                text: `${hit.year} · ${hit.album.artist} — ${hit.album.displayTitle}`,
            };
            // Clamp after Svelte has rendered the new text (next microtask)
            Promise.resolve().then(() => {
                tooltip = { ...tooltip, x: clampTooltipX(idealX) };
            });
        } else {
            canvas.style.cursor = "default";
            tooltip = { ...tooltip, visible: false };
        }
    }

    function onMouseLeave() {
        hoveredKey = null;
        drawCanvas();
        tooltip = { ...tooltip, visible: false };
    }

    function onClick(e: MouseEvent) {
        const rect = canvas.getBoundingClientRect();
        const hit = hitTest(
            e.clientX - rect.left,
            e.clientY - rect.top,
            rect.width,
            rect.height,
        );
        if (hit) audioPlayer.playAlbum(hit.album.id);
    }

    // ── Dynamic canvas height ─────────────────────────────────────────────────
    // Tall enough for the deepest stack, never clips dots at the top.
    $: canvasHeight = albums.length
        ? TOP_PAD +
          Math.max(...albums.map(([, g]) => g.data.length)) *
              (DOT_R * 2 + DOT_GAP) +
          LABEL_H +
          DOT_R
        : 100;

    // Redraw whenever active track changes
    $: if ($current) drawCanvas();

    const onResize = debounce(drawCanvas, 20);
    let ro: ResizeObserver;

    onMount(() => {
        ro = new ResizeObserver(onResize);
        ro.observe(wrapper);
        drawCanvas();
    });

    onDestroy(() => ro?.disconnect());
</script>

<div class="tl-root" bind:this={wrapper}>
    {#if yearSpan > 0}
        <h2>Your library spans <span>{yearSpan} years</span></h2>
    {:else}
        <h2>Generating timeline…</h2>
    {/if}

    <div class="canvas-wrap">
        <canvas
            style="height: {canvasHeight}px"
            bind:this={canvas}
            on:mousemove={onMouseMove}
            on:mouseleave={onMouseLeave}
            on:click={onClick}
        />

        <div
            class="tooltip"
            bind:this={tooltipEl}
            style="left: {tooltip.x}px; top: {tooltip.y}px; visibility: {tooltip.visible
                ? 'visible'
                : 'hidden'};"
        >
            {tooltip.text}
        </div>
    </div>
</div>

<style lang="scss">
    .tl-root {
        position: relative;
        width: 100%;
    }

    h2 {
        margin: 0 0 0.5em;
        font-weight: 400;
        color: var(--text);
        font-size: 1rem;

        span {
            color: white;
            font-weight: 600;
        }
    }

    .canvas-wrap {
        position: relative;
        width: 100%;
    }

    canvas {
        display: block;
        width: 100%;
        /* height is set dynamically via inline style */
    }

    .tooltip {
        position: absolute;
        transform: translateX(-50%);
        pointer-events: none;
        white-space: nowrap;
        z-index: 100;

        background: rgba(18, 18, 24, 0.92);
        backdrop-filter: blur(8px);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 5px;
        padding: 4px 10px;
        font-size: 11px;
        color: rgba(255, 255, 255, 0.85);
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);

        /* prevent overflow at edges */
        max-width: 90%;
        overflow: hidden;
        text-overflow: ellipsis;
    }
</style>
