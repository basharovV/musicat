<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { setQueue } from "../../data/storeHelper";
    import type { Song } from "../../App";
    import { current } from "../../data/store";
    import { currentFont, currentThemeObject } from "../../theming/store";

    // ── Types ──────────────────────────────────────────────────────────────────

    interface Bubble {
        name: string;
        count: number;
        x: number;
        y: number;
        radius: number;
        color: string;
        vx: number;
        vy: number;
    }

    // ── Props ──────────────────────────────────────────────────────────────────

    export let songs: Song[] = [];

    // ── Canvas state — all plain vars, zero Svelte reactivity ─────────────────

    let canvas: HTMLCanvasElement;
    let container: HTMLDivElement;
    let ctx: CanvasRenderingContext2D | null = null;

    let width = 600;
    let height = 400;
    let dpr = 1;
    let settled = false;

    let bubbles: Bubble[] = [];
    let hoveredIndex = -1;
    let resizeObserver: ResizeObserver;

    // ── Tooltip — direct DOM, never touches Svelte reactivity ─────────────────

    let tooltipEl: HTMLDivElement | null = null;

    function tooltipMount(node: HTMLDivElement): { destroy(): void } {
        tooltipEl = node;
        return {
            destroy() {
                tooltipEl = null;
            },
        };
    }

    function showTooltip(text: string, x: number, y: number): void {
        if (!tooltipEl) return;
        tooltipEl.textContent = text;
        tooltipEl.style.left = `${x}px`;
        tooltipEl.style.top = `${y}px`;
        tooltipEl.style.display = "block";
    }

    function hideTooltip(): void {
        if (!tooltipEl) return;
        tooltipEl.style.display = "none";
    }

    // ── Portal — mounts node as direct <body> child ────────────────────────────

    function portal(node: HTMLElement): { destroy(): void } {
        document.body.appendChild(node);
        return {
            destroy() {
                node.parentNode?.removeChild(node);
            },
        };
    }

    // ── Seeded PRNG (mulberry32 + FNV-1a hash) ────────────────────────────────

    function hashStr(str: string): number {
        let h = 2166136261;
        for (let i = 0; i < str.length; i++) {
            h ^= str.charCodeAt(i);
            h = (h * 16777619) >>> 0;
        }
        return h;
    }

    function mulberry32(seed: number): () => number {
        let s = seed >>> 0;
        return () => {
            s += 0x6d2b79f5;
            let z = s;
            z = Math.imul(z ^ (z >>> 15), z | 1);
            z ^= z + Math.imul(z ^ (z >>> 7), z | 61);
            return ((z ^ (z >>> 14)) >>> 0) / 0xffffffff;
        };
    }

    // ── Palette ────────────────────────────────────────────────────────────────

    const PALETTE: string[] = [
        $currentThemeObject["mapview-scale-1"], // Purple
        $currentThemeObject["mapview-scale-2"], // Purple
    ];

    // ── Similarity (Jaccard on tokens) ────────────────────────────────────────

    function tokenize(s: string): Set<string> {
        return new Set(
            s
                .toLowerCase()
                .replace(/[^a-z0-9 ]/g, "")
                .split(/\s+/)
                .filter(Boolean),
        );
    }

    function similarity(a: string, b: string): number {
        const ta = tokenize(a);
        const tb = tokenize(b);
        if (!ta.size && !tb.size) return 1;
        let inter = 0;
        ta.forEach((t) => {
            if (tb.has(t)) inter++;
        });
        const union = ta.size + tb.size - inter;
        return union === 0 ? 0 : inter / union;
    }

    // ── Build ──────────────────────────────────────────────────────────────────

    function buildBubbles(): void {
        if (!ctx) return;

        const counts = new Map<string, number>();
        songs.forEach((song) => {
            (song.genre ?? []).forEach((g) => {
                const key = g.trim();
                if (key) counts.set(key, (counts.get(key) ?? 0) + 1);
            });
        });

        const entries = [...counts.entries()].sort((a, b) => b[1] - a[1]);
        if (!entries.length) {
            bubbles = [];
            draw();
            return;
        }

        const maxCount = entries[0][1];
        const names = entries.map((e) => e[0]);
        const n = names.length;

        // Scale radii so total area ≈ 58% of canvas
        const canvasArea = width * height * 0.58;
        const rawR = entries.map(([, c]) => (c / maxCount) ** 0.5);
        const rawAreaSum = rawR.reduce((sum, r) => sum + r * r * Math.PI, 0);
        const scale = Math.sqrt(canvasArea / rawAreaSum);
        const minR = 14;
        const maxR = Math.min(width, height) * 0.21;

        // Greedy angular ordering — similar genres placed at adjacent angles
        const order = [0];
        const used = new Set([0]);
        while (order.length < n) {
            const last = order[order.length - 1];
            let best = -1,
                bestSim = -1;
            for (let i = 0; i < n; i++) {
                if (!used.has(i)) {
                    const s = similarity(names[last], names[i]);
                    if (s > bestSim) {
                        bestSim = s;
                        best = i;
                    }
                }
            }
            order.push(best);
            used.add(best);
        }

        const angles = new Array<number>(n);
        order.forEach((idx, pos) => {
            angles[idx] = (pos / n) * Math.PI * 2;
        });

        // Deterministic jitter — same genre set always produces same layout
        const seed = hashStr(names.slice().sort().join("|"));
        const rand = mulberry32(seed);

        const newBubbles: Bubble[] = entries.map(([name, count], i) => {
            const r = Math.min(maxR, Math.max(minR, rawR[i] * scale));
            const jitter = 0.35 + rand() * 0.55;
            return {
                name,
                count,
                x: width / 2 + Math.cos(angles[i]) * width * 0.36 * jitter,
                y: height / 2 + Math.sin(angles[i]) * height * 0.28 * jitter,
                radius: r,
                color: PALETTE[i % PALETTE.length],
                vx: 0,
                vy: 0,
            };
        });

        simulate(newBubbles, 500);
        bubbles = newBubbles;
        hoveredIndex = -1;
        settled = true;
        draw();
    }

    // ── Simulation ────────────────────────────────────────────────────────────

    function simulate(bs: Bubble[], steps: number): void {
        const cx = width / 2;
        const cy = height / 2;
        const pad = 3;

        for (let step = 0; step < steps; step++) {
            const alpha = Math.pow(1 - step / steps, 1.4);

            bs.forEach((b) => {
                b.vx += (cx - b.x) * 0.005 * alpha;
                b.vy += (cy - b.y) * 0.005 * alpha;
            });

            for (let i = 0; i < bs.length; i++) {
                for (let j = i + 1; j < bs.length; j++) {
                    const a = bs[i],
                        b = bs[j];
                    const dx = b.x - a.x;
                    const dy = b.y - a.y;
                    const dist = Math.sqrt(dx * dx + dy * dy) || 0.001;
                    const minDist = a.radius + b.radius + pad;
                    if (dist < minDist) {
                        const push = ((minDist - dist) / dist) * 0.52;
                        const fx = dx * push,
                            fy = dy * push;
                        a.vx -= fx;
                        a.vy -= fy;
                        b.vx += fx;
                        b.vy += fy;
                    }
                }
            }

            bs.forEach((b) => {
                b.vx *= 0.8;
                b.vy *= 0.8;
                b.x += b.vx;
                b.y += b.vy;
                b.x = Math.max(
                    b.radius + 1,
                    Math.min(width - b.radius - 1, b.x),
                );
                b.y = Math.max(
                    b.radius + 1,
                    Math.min(height - b.radius - 1, b.y),
                );
            });
        }
    }

    // ── Draw ──────────────────────────────────────────────────────────────────

    function draw(): void {
        if (!ctx) return;
        ctx.clearRect(0, 0, width, height);

        // Similarity edges
        ctx.lineWidth = 1;
        for (let i = 0; i < bubbles.length; i++) {
            for (let j = i + 1; j < bubbles.length; j++) {
                const s = similarity(bubbles[i].name, bubbles[j].name);
                if (s > 0.2) {
                    ctx.beginPath();
                    ctx.moveTo(bubbles[i].x, bubbles[i].y);
                    ctx.lineTo(bubbles[j].x, bubbles[j].y);
                    ctx.strokeStyle = `rgba(255,255,255,${s * 0.08})`;
                    ctx.stroke();
                }
            }
        }

        // Non-hovered first, hovered on top
        bubbles.forEach((b, i) => {
            if (i !== hoveredIndex)
                renderBubble(b, false, $current.song?.genre.includes(b.name));
        });
        if (hoveredIndex >= 0)
            renderBubble(
                bubbles[hoveredIndex],
                true,
                $current.song?.genre.includes(bubbles[hoveredIndex].name),
            );
    }

    function renderBubble(
        b: Bubble,
        isHovered: boolean,
        isPlaying: boolean = false,
    ): void {
        if (!ctx) return;

        // Fill
        ctx.beginPath();
        ctx.arc(b.x, b.y, b.radius, 0, Math.PI * 2);
        ctx.fillStyle = isPlaying
            ? $currentThemeObject["accent"]
            : isHovered
              ? lighten(b.color, 55)
              : b.color;
        ctx.fill();

        // Ring
        ctx.beginPath();
        ctx.arc(b.x, b.y, b.radius - 1, 0, Math.PI * 2);
        ctx.strokeStyle = isHovered
            ? "rgba(255,255,255,0.9)"
            : "rgba(255,255,255,0.14)";
        ctx.lineWidth = isHovered ? 3 : 1;
        ctx.stroke();

        // Text label
        const fontSize = Math.max(9, Math.min(b.radius * 0.37, 15));
        if (b.radius > 22) {
            ctx.textAlign = "center";
            ctx.textBaseline = "middle";
            ctx.fillStyle = "rgba(255,255,255,0.95)";
            ctx.font = `600 ${fontSize}px 'DM Sans', system-ui, sans-serif`;

            const words = b.name.split(" ");
            const maxW = b.radius * 1.55;
            const lines: string[] = [];
            let cur = "";
            words.forEach((w) => {
                const test = cur ? `${cur} ${w}` : w;
                if (ctx!.measureText(test).width > maxW && cur) {
                    lines.push(cur);
                    cur = w;
                } else cur = test;
            });
            if (cur) lines.push(cur);

            const lineH = fontSize * 1.25;
            const showCount = b.radius > 34;
            const blockH =
                lines.length * lineH + (showCount ? fontSize * 1.1 : 0);
            const startY = b.y - blockH / 2 + lineH / 2;

            lines.forEach((line, li) =>
                ctx!.fillText(line, b.x, startY + li * lineH),
            );

            if (showCount) {
                ctx.font = `400 ${Math.max(8, fontSize * 0.72)}px 'DM Sans', system-ui, sans-serif`;
                ctx.fillStyle = "rgba(255,255,255,0.5)";
                ctx.fillText(`${b.count}`, b.x, startY + lines.length * lineH);
            }
        }
    }

    function lighten(hex: string, amount: number): string {
        const r = Math.min(255, parseInt(hex.slice(1, 3), 16) + amount);
        const g = Math.min(255, parseInt(hex.slice(3, 5), 16) + amount);
        const b = Math.min(255, parseInt(hex.slice(5, 7), 16) + amount);
        return `rgb(${r},${g},${b})`;
    }

    // ── Genre playback ────────────────────────────────────────────────────────

    function playGenre(genre: string): void {
        const genreSongs = songs.filter((s) => s.genre?.includes(genre));
        if (genreSongs.length) setQueue(genreSongs, 0);
    }

    function hitTest(clientX: number, clientY: number): number {
        const rect = canvas.getBoundingClientRect();
        const mx = (clientX - rect.left) * (width / rect.width);
        const my = (clientY - rect.top) * (height / rect.height);
        for (let i = bubbles.length - 1; i >= 0; i--) {
            const { x, y, radius } = bubbles[i];
            const dx = mx - x,
                dy = my - y;
            if (dx * dx + dy * dy <= radius * radius) return i;
        }
        return -1;
    }

    // ── Mouse handlers ────────────────────────────────────────────────────────

    function handleMouseMove(e: MouseEvent): void {
        if (!settled) return;
        const foundIndex = hitTest(e.clientX, e.clientY);

        if (foundIndex !== hoveredIndex) {
            hoveredIndex = foundIndex;
            canvas.style.cursor = foundIndex >= 0 ? "pointer" : "default";
            draw();
        }

        const found = foundIndex >= 0 ? bubbles[foundIndex] : null;
        if (found && found.radius <= 22) {
            showTooltip(`${found.name} · ${found.count}`, e.clientX, e.clientY);
        } else {
            hideTooltip();
        }
    }

    function handleMouseLeave(): void {
        hoveredIndex = -1;
        canvas.style.cursor = "default";
        hideTooltip();
        draw();
    }

    function handleClick(e: MouseEvent): void {
        if (!settled) return;
        const foundIndex = hitTest(e.clientX, e.clientY);
        if (foundIndex >= 0) playGenre(bubbles[foundIndex].name);
    }

    // ── Resize ────────────────────────────────────────────────────────────────

    function resize(): void {
        if (!container || !canvas || !ctx) return;
        const rect = container.getBoundingClientRect();
        width = Math.max(rect.width, 200);
        height = Math.max(rect.height, 200);
        dpr = Math.min(window.devicePixelRatio || 1, 2);

        canvas.width = width * dpr;
        canvas.height = height * dpr;
        canvas.style.width = `${width}px`;
        canvas.style.height = `${height}px`;

        // Set DPR scale once — all coords stay in CSS pixels
        ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

        if (songs.length) buildBubbles();
        else draw();
    }

    // ── Lifecycle ─────────────────────────────────────────────────────────────

    onMount(() => {
        ctx = canvas.getContext("2d");
        resizeObserver = new ResizeObserver(resize);
        resizeObserver.observe(container);
        resize();
    });

    onDestroy(() => resizeObserver?.disconnect());

    // Only reactive statement — watches `songs` prop, nothing internal
    $: songs, current && buildBubbles();
</script>

<div class="genre-map" bind:this={container}>
    <canvas
        bind:this={canvas}
        on:mousemove={handleMouseMove}
        on:mouseleave={handleMouseLeave}
        on:click={handleClick}
        aria-label="Genre distribution bubble chart"
        role="img"
    />
    {#if !songs.length || bubbles.length === 0}
        <p class="empty">No genre data</p>
    {/if}
</div>

<!-- Always mounted, shown/hidden via display. Zero Svelte reactivity. -->
<div
    class="genre-tooltip"
    use:portal
    use:tooltipMount
    style="display:none"
></div>

<svelte:head>
    <style>
        .genre-tooltip {
            position: fixed;
            z-index: 9999;
            pointer-events: none;
            transform: translate(-50%, calc(-100% - 10px));
            background: rgba(15, 15, 20, 0.92);
            border: 1px solid rgba(255, 255, 255, 0.13);
            color: #fff;
            font-family: "DM Sans", system-ui, sans-serif;
            font-size: 12px;
            font-weight: 500;
            padding: 4px 10px;
            border-radius: 5px;
            white-space: nowrap;
            letter-spacing: 0.02em;
            backdrop-filter: blur(6px);
        }
        .genre-tooltip::after {
            content: "";
            position: absolute;
            bottom: -4px;
            left: 50%;
            transform: translateX(-50%);
            border: 4px solid transparent;
            border-top-color: rgba(255, 255, 255, 0.13);
            border-bottom: none;
        }
    </style>
</svelte:head>

<style lang="scss">
    .genre-map {
        position: relative;
        width: 100%;
        height: 100%;
    }

    canvas {
        display: block;
    }

    .empty {
        position: absolute;
        inset: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        margin: 0;
        font-family: "DM Sans", system-ui, sans-serif;
        font-size: 13px;
        color: rgba(255, 255, 255, 0.25);
        letter-spacing: 0.05em;
    }
</style>
