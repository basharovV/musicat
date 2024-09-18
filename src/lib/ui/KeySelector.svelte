<script lang="ts">
    import { autoWidth } from "../../utils/AutoWidth";
    import Menu from "../menu/Menu.svelte";
    import isDarkColor from "is-dark-color";
    import type { MenuItem, MenuSection } from "../../App";

    let matches: { name: string; color: string }[] = [];
    let selectedItem: MenuItem = null;
    export let value = "";
    export let onKeyUpdated;

    $: firstMatch = (matches.length && matches[0]) ?? null;
    let textInputColor;

    $: correctedTextInputColor =
        textInputColor !== null && isDarkColor(textInputColor)
            ? "white"
            : textInputColor ?? "#FFFFFF";
    let options: { name: string; color: string }[] = [
        // Major#
        { name: "C", color: "#FFFFFF" },
        { name: "C#", color: "#E5C874" },
        { name: "D", color: "#FFD600" },
        { name: "D#", color: "#48BF8D" },
        { name: "E", color: "#70EE3D" },
        { name: "F", color: "#ED7513" },
        { name: "F#", color: "#ED4A8F" },
        { name: "G", color: "#FF3400" },
        { name: "G#", color: "#E600FF" },
        { name: "A", color: "#3BBDDF" },
        { name: "A#", color: "#936339" },
        { name: "B", color: "#FF4F00" },
        // Minor#
        { name: "Cm", color: "#156E6E" },
        { name: "C#m", color: "#1B0909" },
        { name: "Dm", color: "#06176A" },
        { name: "D#m", color: "#6A065E" },
        { name: "Em", color: "#3A3997" },
        { name: "Fm", color: "#04431C" },
        { name: "F#m", color: "#740E0E" },
        { name: "Gm", color: "#2A2612" },
        { name: "G#m", color: "#1C4128" },
        { name: "Am", color: "#3C1C41" },
        { name: "A#m", color: "#422210" },
        { name: "Bm", color: "#723DEE" }
    ];

    $: sections = matches.reduce(
        (sections: MenuSection[], currentKey) => {
            if (!currentKey.name.endsWith("m")) {
                sections
                    .find((s) => s.title === "Major")
                    .items.push({
                        text: currentKey.name,
                        color: currentKey.color
                    });
            } else {
                sections
                    .find((s) => s.title === "Minor")
                    .items.push({
                        text: currentKey.name,
                        color: currentKey.color
                    });
            }
            return sections;
        },
        [
            {
                title: "Major",
                items: []
            },
            {
                title: "Minor",
                items: []
            }
        ]
    );

    $: {
        if (value) {
            textInputColor =
                options.find((o) => o.name === value)?.color ?? "#FFFFFF";
        } else {
            textInputColor = "#FFFFFF";
        }
    }

    function onFocus() {
        console.log("onfocus");
        setTimeout(() => {
            matches = options;
            updateQueryPartsAutocompletePos();

            if (value && value.trim().length === 0) {
                matches = matches;
                console.log("matching parts", matches);
            }
        }, 150);
    }

    function onLostFocus() {
        matches = [];
    }

    async function onInput(evt) {
        let matched = [];
        value = evt.target.value;
        if (value && value.trim().length > 0) {
            updateQueryPartsAutocompletePos();
            matches = [];
            options.forEach((a) => {
                if (a.name.toLowerCase().includes(value.toLowerCase())) {
                    matched.push(a);
                }
            });

            if (value.trim() in options.map((o) => o.name)) {
                onKeyUpdated(value);
            }

            matches = matched;
        } else {
            matches = options;
        }
    }

    function onSelectItem(item: MenuItem) {
        console.log("onSelectItem", item);
        value = item.text;
        onKeyUpdated(value);
        matches = [];
    }

    function closeAutoComplete() {
        matches = [];
    }

    let artistInput: HTMLInputElement;
    let inputX: number;
    let inputY: number;

    function updateQueryPartsAutocompletePos() {
        inputX = artistInput.offsetLeft;
        inputY = artistInput.offsetTop + 40;
        console.log("X", inputX, "Y", inputY);
    }
</script>

<input
    use:autoWidth
    bind:this={artistInput}
    {value}
    maxlength="3"
    on:input={onInput}
    on:focus={onFocus}
    type="text"
    class="artist add{options?.length ? ' alt' : ''}"
    placeholder="key"
    style="color: {correctedTextInputColor}"
    autocomplete="off"
    spellcheck="false"
/>

{#if matches.length > 0}
    <Menu
        x={inputX}
        y={inputY}
        onClickOutside={closeAutoComplete}
        {sections}
        padding={2}
        position="manual"
        onItemSelected={onSelectItem}
    />
{/if}

<style lang="scss">
    input {
        min-width: 60px;
        max-width: 40px;
        width: 100%;
        align-items: center;
        padding: 0.2em 0.5em;
        font-size: 14px;
        outline: none;
        background: none;
        border: none;

        &::placeholder {
            color: rgb(105, 105, 105);
        }
    }
</style>
