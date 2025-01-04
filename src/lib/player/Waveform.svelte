<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import type WaveSurfer from "wavesurfer.js";
    import ZoomPlugin from "wavesurfer.js/dist/plugins/zoom.esm.js";

    import {
        current,
        isCmdOrCtrlPressed,
        os,
        playerTime,
        seekTime,
        waveformPeaks
    } from "../../data/store";

    import type { Event } from "@tauri-apps/api/event";
    import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
    import { fade } from "svelte/transition";
    import type { Marker, Waveform } from "../../App";
    import RegionsPlugin from "./RegionsPlugin";
    import Hover from "wavesurfer.js/dist/plugins/hover.esm.js";
    import hotkeys from "hotkeys-js";
    import { db } from "../../data/db";
    import { currentThemeObject } from "../../theming/store";

    const appWindow = getCurrentWebviewWindow()

    let container;
    let isMounted = false;
    let wavesurfer: WaveSurfer;
    let wsRegions: RegionsPlugin;
    let song = null;

    let pxPerSec = 0;
    let isZoomed = false;

    // Loop
    let loopStartPos = null;
    let loopEndPos = null;

    const emptyAudio =
        "data:audio/mp3;base64,SUQzBAAAAAAAI1RTU0UAAAAPAAADTGF2ZjU2LjM2LjEwMAAAAAAAAAAAAAAA//OEAAAAAAAAAAAAAAAAAAAAAAAASW5mbwAAAA8AAAAEAAABIADAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDV1dXV1dXV1dXV1dXV1dXV1dXV1dXV1dXV6urq6urq6urq6urq6urq6urq6urq6urq6v////////////////////////////////8AAAAATGF2YzU2LjQxAAAAAAAAAAAAAAAAJAAAAAAAAAAAASDs90hvAAAAAAAAAAAAAAAAAAAA//MUZAAAAAGkAAAAAAAAA0gAAAAATEFN//MUZAMAAAGkAAAAAAAAA0gAAAAARTMu//MUZAYAAAGkAAAAAAAAA0gAAAAAOTku//MUZAkAAAGkAAAAAAAAA0gAAAAANVVV";
    onMount(async () => {
        const { default: WaveSurfer } = await import("wavesurfer.js");

        wavesurfer = WaveSurfer.create({
            container,
            waveColor: $currentThemeObject["waveform-wave"],
            progressColor: $currentThemeObject["waveform-progress"],
            cursorWidth: 1,
            cursorColor: $currentThemeObject["waveform-cursor"],
            // backend: 'MediaElement',
            barWidth: 1.5,
            responsive: true,
            barHeight: 2,
            height: "auto",
            barRadius: 2,
            autoScroll: false,
            hideScrollbar: true,
        });
        wavesurfer.registerPlugin(
            Hover.create({
                lineColor: $currentThemeObject["waveform-hover-line"],
                lineWidth: 1,
                labelBackground: $currentThemeObject["waveform-hover-label-bg"],
                labelColor: $currentThemeObject["waveform-hover-label-text"],
                labelSize: "11px"
            })
        );
        wavesurfer.registerPlugin(
            ZoomPlugin.create({
                // the amount of zoom per wheel step, e.g. 0.5 means a 50% magnification per scroll
                scale: 0.1,
                // Optionally, specify the maximum pixels-per-second factor while zooming
                maxZoom: 40
            })
        );
        wsRegions = wavesurfer.registerPlugin(RegionsPlugin.create());

        // Patch this function
        wavesurfer.seekTo = function (progress) {
            if (!hotkeys.isPressed("cmd") && !hotkeys.isPressed("ctrl")) {
                const time = wavesurfer.getDuration() * progress;
                wavesurfer.setTime(time);
            }
        };

        wsRegions.on("region-created", (region) => {
            console.log("region", region);
            if (region.start !== region.end) {
                wsRegions
                    .getRegions()
                    .filter((r) => r.start !== r.end)
                    .forEach((r) => {
                        if (r.id !== region.id) {
                            r.remove();
                        }
                    });

                // Set loop parameters
                if (
                    region.start !== $waveformPeaks.loopStartPos &&
                    region.end !== $waveformPeaks.loopEndPos
                ) {
                    console.log("NEW REGION", region);
                    invoke("loop_region", {
                        event: {
                            enabled: true,
                            start_pos: region.start,
                            end_pos: region.end
                        }
                    });
                }

                $waveformPeaks.loopEnabled = true;
                $waveformPeaks.loopStartPos = region.start;
                $waveformPeaks.loopEndPos = region.end;
            }
        });

        wsRegions.on("region-clicked", (region, ev) => {
            ev.stopPropagation();
            console.log("region-clicked");
            region.remove();
            $current.song.markers?.splice(
                $current.song.markers.findIndex((m) => m.pos === region.start),
                1
            );
            db.songs.put($current.song);
            if (region.start !== region.end) {
                $waveformPeaks.loopStartPos = null;
                $waveformPeaks.loopEndPos = null;
                invoke("loop_region", {
                    event: {
                        enabled: false,
                        start_pos: null,
                        end_pos: null
                    }
                });
                $waveformPeaks.loopEnabled = false;
            }
        });

        wsRegions.on("region-updated", (region) => {
            if (region.start !== region.end) {
                console.log("updated Loop", region);
                invoke("loop_region", {
                    event: {
                        enabled: true,
                        start_pos: region.start,
                        end_pos: region.end
                    }
                });

                $waveformPeaks.loopEnabled = true;
                $waveformPeaks.loopStartPos = region.start;
                $waveformPeaks.loopEndPos = region.end;
            }
        });

        wsRegions.on("region-double-clicked", (region, ev) => {
            ev.stopPropagation();
            console.log("region-dblclicked");
        });

        wsRegions.enableDragSelection({
            color: $currentThemeObject["waveform-region-loop"]
        });

        wavesurfer.on("click", (pos) => {
            console.log("interaction", pos);
            let posSeconds = $current.song.fileInfo.duration * pos;
            if (hotkeys.isPressed("cmd") || hotkeys.isPressed("ctrl")) {
                if ($current.song) {
                    const marker: Marker = {
                        pos: posSeconds,
                        title: "👂"
                    };
                    if ($current.song.markers) {
                        $current.song.markers.push(marker);
                    } else {
                        $current.song.markers = [marker];
                    }
                    db.songs.put($current.song);
                    wsRegions.addRegion({
                        start: posSeconds,
                        content: marker.title,
                        color: $currentThemeObject["waveform-region-current"]
                    });
                }
            } else {
                seekTime.set($current.song.fileInfo.duration * pos);
            }
        });

        wavesurfer.on("zoom", (px) => {
            isZoomed = px > pxPerSec;
        });

        isMounted = true;

        appWindow.listen("waveform", async (event: Event<Waveform>) => {
            await wavesurfer.load(
                null,
                event.payload.data,
                $current.song.fileInfo.duration
            );
            pxPerSec = wavesurfer.options.minPxPerSec;
            if (!$waveformPeaks) {
                $waveformPeaks = {
                    ...$waveformPeaks,
                    songId: $current.song.id,
                    data: event.payload.data
                };
            } else {
                $waveformPeaks.data = event.payload.data;
            }
        });
    });

    playerTime.subscribe((playerTime) => {
        if (wavesurfer && $current.song) {
            wavesurfer.seekTo(
                Math.min(playerTime / $current.song.fileInfo.duration)
            );
        }
    });

    async function restoreState() {
        await wavesurfer.load(
            null,
            $waveformPeaks.data,
            $current.song.fileInfo.duration
        );
        pxPerSec = wavesurfer.options.minPxPerSec;

        if ($current.song) {
            wavesurfer.seekTo($playerTime / $current.song.fileInfo.duration);
        }

        // Restore loop point
        if ($waveformPeaks.loopEnabled) {
            console.log("STATE", $waveformPeaks);
            wsRegions.addRegion({
                start: $waveformPeaks.loopStartPos,
                end: $waveformPeaks.loopEndPos,
                color: $currentThemeObject["waveform-region-loop"]
            });
        }

        $current.song.markers?.forEach((m) => {
            wsRegions.addRegion({
                start: m.pos,
                content: m.title,
                color: $currentThemeObject["waveform-region-current"]
            });
        });
    }

    async function getWaveform() {
        if (
            $waveformPeaks?.data &&
            $current.song?.id === $waveformPeaks.songId
        ) {
            restoreState();
        } else if ($current.song?.id !== $waveformPeaks?.songId) {
            // Changing songs, reset and get new waveform
            wsRegions.clearRegions();
            const result = await invoke("get_waveform", {
                event: {
                    path: $current.song.path
                }
            });
            $waveformPeaks.songId = $current.song.id;

            $current.song.markers?.forEach((m) => {
                wsRegions.addRegion({
                    start: m.pos,
                    content: m.title,
                    color: $currentThemeObject["waveform-region-current"]
                });
            });
        }
        song = $current.song;
        // console.log("result", result);
    }

    $: if (isMounted && $current.song?.path !== song?.path && wavesurfer) {
        getWaveform();
        console.log(
            "$current.song.fileInfo.duration",
            $current.song.fileInfo.duration
        );
        wavesurfer.setOptions({ duration: $current.song.fileInfo.duration });

        if (wavesurfer.getDecodedData()) {
            wavesurfer.zoom(false);
        }

        console.log("duration", wavesurfer.getDuration());
    }

    let hoverPos = 0;
    let hoverTime = "";
    let showHoverhead = false;
</script>

<div class="container">
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div bind:this={container} class:zoomed={isZoomed} class="waveform" />
    {#if showHoverhead}
        <div
            class="hoverhead"
            style="left: {hoverPos}px;"
            transition:fade={{ duration: 100 }}
        >
            <p>{hoverTime}</p>
            <div class="line" />
        </div>
    {/if}
</div>

<style lang="scss">
    .container {
        margin: auto 0;
        height: 100%;
        position: relative;
        padding: 8px 0;
        .waveform {
            margin: auto;
            max-height: 40px;

            &.zoomed {
                mask-image: linear-gradient(
                    to right,
                    transparent 0%,
                    white 5%,
                    white 95%,
                    transparent 100%
                );
            }
        }
        .hoverhead {
            position: absolute;
            z-index: 2;
            height: 100%;
            top: 0;
            bottom: 0;
            pointer-events: none;
            p {
                position: absolute;
                bottom: 0px;
                left: 5px;
                height: fit-content;
                margin: 0;
                font-size: 12px;
            }
            .line {
                height: 100%;
                width: 1px;
                position: absolute;
                top: 0;
                bottom: 0;
                background-color: var(--waveform-hoverhead-line-bg);
            }
        }
    }
</style>
