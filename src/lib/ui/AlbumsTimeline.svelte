<script lang="ts">
    import { onMount } from "svelte";
    import tippy from "tippy.js";
    import { debounce } from "lodash-es";
    import { fade, fly } from "svelte/transition";

    const pickn = (a, n) => {
        var p = Math.floor(a.length / n);
        return a.slice(0, p * n).filter(function (_, i) {
            return 0 == i % p;
        });
    };
    export let albumsByYear;

    let albums = [];
    let labelYears = [];
    let maxYear = 0;
    let minYear = 0;
    let yearSpan = 0;
    $: {
        if (albumsByYear && lineWidth) {
            albums = [];
            labelYears = [];
            maxYear = 0;
            minYear = 0;
            yearSpan = 0;

            albums = Object.entries(albumsByYear).filter(
                (a) => a[0] && Number(a[0]) > 1800
            );
            if (albums.length) {
                maxYear = Number(albums[albums.length - 1][0]);
                minYear = Number(albums[0][0]);
                yearSpan = maxYear - minYear;

                let looking = true;
                for (let i = minYear; i <= maxYear; i++) {
                    for (let u = minYear; looking; u--) {
                        if (u % 10 === 0) {
                            labelYears.push(u);
                            looking = false;
                            break;
                        }
                    }
                    if (i % 10 === 0) {
                        labelYears.push(i);
                    }
                    labelYears = labelYears;
                }
                minYear = labelYears[0];
                maxYear = labelYears[labelYears.length - 1];
                ready = true;
            }
        }
    }

    let line: HTMLDivElement;
    let lineWidth = 0;
    let isMounted = false;
    let ready = false;

    function getXPos(year) {
        lineWidth = line?.clientWidth;

        const pxPerYear = lineWidth / yearSpan;

        return (year - minYear) * pxPerYear;
    }

    onMount(() => {
        isMounted = true;
        lineWidth = line?.clientWidth;
    });

    function onResize() {
        lineWidth = line?.clientWidth;
    }
</script>

<svelte:window on:resize={debounce(onResize, 30)} />
<container>
    {#if !ready}
        <h2>Generating timeline...</h2>
    {:else}
        <h2>Your library spans <span>{yearSpan} years</span></h2>
    {/if}

    <timeline-container>
        <timeline>
            <div id="line" bind:this={line}></div>
        </timeline>
        <!-- <dots>
        {#if isMounted}
            {#each albums as album, idx (album[0])}
                <div
                    class="dot"
                    style="left: {getXPos(Number(album[0]))}px"
                    use:tippy={{
                        content: `${album[0]} - ${album[1].data[0].title}`,
                        placement: "bottom"
                    }}
                ></div>
            {/each}
        {/if}
    </dots> -->
        <albums>
            {#if isMounted}
                {#each albums as album, idx (album[0])}
                    {#if ready}
                        <div
                            in:fade={{ delay: idx * 8 }}
                            class="album-stack"
                            style="left: {getXPos(Number(album[0]))}px"
                        >
                            {#each album[1].data.slice(0, 5) as a, idx (a.id)}
                                {#if a.artwork}
                                    <img
                                        alt="artwork"
                                        src={a.artwork?.src || ""}
                                        class="album"
                                        use:tippy={{
                                            content: `${album[0]} - ${a.title}`,
                                            placement: "bottom"
                                        }}
                                    />
                                {:else}
                                    <div
                                        class="dot"
                                        use:tippy={{
                                            content: `${album[0]} - ${a.title}`,
                                            placement: "bottom"
                                        }}
                                    />
                                {/if}
                            {/each}
                        </div>
                    {/if}
                {/each}
            {/if}
        </albums>
        <labels>
            {#if isMounted && ready}
                {#each labelYears as year}
                    <div
                        in:fly={{
                            opacity: 0,
                            delay: 100,
                            y: -30,
                            duration: 600
                        }}
                        class="item"
                        style="left: {getXPos(year)}px"
                    >
                        <div class="vertical-line" />
                        <p>{year}</p>
                    </div>
                {/each}
            {/if}
        </labels>
    </timeline-container>
</container>

<style lang="scss">
    $border_color: rgba(236, 229, 229, 0.61);
    container {
        position: relative;
        width: 100%;
        height: 220px;

        h2 {
            margin: 0;
            position: absolute;
            top: 0.8em;
            left: 0;
            right: 0;
            color: $border_color;
            font-weight: 400;
            span {
                color: white;
                font-weight: 600;
            }
        }
    }
    timeline-container {
        position: absolute;
        width: 80%;
        left: 0;
        right: 0;
        margin: 0 auto;
        height: 150px;
        top: 110px;
        bottom: 0;
    }

    timeline {
        width: 100%;
        height: 30px;
        position: absolute;
        left: 0;
        right: 0;
        pointer-events: none;

        #line {
            width: 100%;
            height: 2px;
            position: absolute;
            margin: auto;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            border: 1px solid rgba(236, 229, 229, 0.31);
            border-radius: 2px;
            /* background-color: $border_color; */
        }
    }

    labels {
        width: 100%;
        position: absolute;
        margin-top: 25px;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;

        .item {
            position: absolute;
            width: fit-content;
            top: 0;
            bottom: 0;
            margin-top: 5px;

            &:not(:first-child, :last-child) {
                opacity: 0.5;
            }

            .vertical-line {
                height: 20px;
                width: 1px;
                border: 1px solid rgba(236, 229, 229, 0.31);
                border-radius: 2px;
            }

            p {
                rotate: 40deg;
                color: $border_color;
                margin: 0.5em 0;
                width: fit-content;
            }
        }
    }

    dots {
        width: 100%;
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
    }

    albums {
        width: 100%;
        position: absolute;
        top: 22px;
        left: 0;
        right: 0;
        bottom: 0;
        height: fit-content;
        .album-stack {
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: flex-end;
            position: absolute;
            top: 0;
            bottom: 0;
            gap: 2px;

            * {
                user-select: none;
            }

            .album {
                width: 14px;
                height: 14px;
                min-height: 14px;
                border-radius: 10px;
                border: 1px solid rgba(128, 128, 128, 0.422);
                background-color: rgb(30, 30, 30);
                z-index: 10;
                &:hover {
                    transform: scale(2);
                    z-index: 1000000 !important;
                }
            }

            .dot {
                width: 10px;
                height: 14px;
                min-height: 14px;
                margin: 0 auto;
                border-radius: 10px;
                border: 1.5px solid rgba(128, 128, 128, 0.594);
                background-color: rgb(65, 65, 65);
            }
        }
    }
</style>
