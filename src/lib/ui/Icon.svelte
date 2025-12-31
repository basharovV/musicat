<script>
    import { optionalTippy } from "./TippyAction";

    export let icon;
    export let size = 20;
    export let sizePx = `${size}px`;
    export let width = sizePx;
    export let height = sizePx;
    export let color = "currentColor";
    export let stroke = color;
    export let disabled = false;
    let fill = color;
    export let onClick = null;
    export let boxed = false;
    export let tooltip = null;
    export let cursor = "inherit";

    let className = "";
    export { className as class };
    let icons = {
        "mdi:music-clef-treble": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><rect x="0" y="0" width="24" height="24" fill="none" stroke="none" /><path fill="currentColor" d="M13 11V7.5l2.2-2.21c.8-.79.95-2.05.39-3.03A2.467 2.467 0 0 0 13.45 1c-.21 0-.45.03-.64.09C11.73 1.38 11 2.38 11 3.5v3.24L7.86 9.91a5.946 5.946 0 0 0-1.25 6.43c.77 1.9 2.45 3.21 4.39 3.55v.61c0 .26-.23.5-.5.5H9v2h1.5c1.35 0 2.5-1.11 2.5-2.5V20c2.03 0 4.16-1.92 4.16-4.75c0-2.3-1.92-4.25-4.16-4.25m0-7.5c0-.23.11-.41.32-.47c.22-.06.45.03.56.23a.5.5 0 0 1-.08.61l-.8.86zm-2 8c-.97.64-1.7 1.74-1.96 2.76l1.96.52v3.05a3.86 3.86 0 0 1-2.57-2.26c-.59-1.46-.27-3.12.83-4.24L11 9.5zm2 6.5v-5.06c1.17 0 2.18 1.1 2.18 2.31C15.18 17 13.91 18 13 18"/></svg>`,
        },
        "mingcute:close-circle-fill": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><rect x="0" y="0" width="24" height="24" fill="none" stroke="none" /><g fill="none"><path d="M24 0v24H0V0zM12.593 23.258l-.011.002l-.071.035l-.02.004l-.014-.004l-.071-.035c-.01-.004-.019-.001-.024.005l-.004.01l-.017.428l.005.02l.01.013l.104.074l.015.004l.012-.004l.104-.074l.012-.016l.004-.017l-.017-.427c-.002-.01-.009-.017-.017-.018m.265-.113l-.013.002l-.185.093l-.01.01l-.003.011l.018.43l.005.012l.008.007l.201.093c.012.004.023 0 .029-.008l.004-.014l-.034-.614c-.003-.012-.01-.02-.02-.022m-.715.002a.023.023 0 0 0-.027.006l-.006.014l-.034.614c0 .012.007.02.017.024l.015-.002l.201-.093l.01-.008l.004-.011l.017-.43l-.003-.012l-.01-.01z"/><path fill="currentColor" d="M12 2c5.523 0 10 4.477 10 10s-4.477 10-10 10S2 17.523 2 12S6.477 2 12 2M9.879 8.464a1 1 0 0 0-1.498 1.32l.084.095l2.12 2.12l-2.12 2.122a1 1 0 0 0 1.32 1.498l.094-.083L12 13.414l2.121 2.122a1 1 0 0 0 1.498-1.32l-.083-.095L13.414 12l2.122-2.121a1 1 0 0 0-1.32-1.498l-.095.083L12 10.586z"/></g></svg>`,
        },
        "line-md:loading-loop": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-dasharray="15" stroke-dashoffset="15" stroke-linecap="round" stroke-width="2" d="M12 3C16.9706 3 21 7.02944 21 12"><animate fill="freeze" attributeName="stroke-dashoffset" dur="0.3s" values="15;0"/><animateTransform attributeName="transform" dur="1.5s" repeatCount="indefinite" type="rotate" values="0 12 12;360 12 12"/></path></svg>`,
        },
        "ion:search": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 512 512"><path fill="currentColor" d="M456.69 421.39L362.6 327.3a173.81 173.81 0 0 0 34.84-104.58C397.44 126.38 319.06 48 222.72 48S48 126.38 48 222.72s78.38 174.72 174.72 174.72A173.81 173.81 0 0 0 327.3 362.6l94.09 94.09a25 25 0 0 0 35.3-35.3M97.92 222.72a124.8 124.8 0 1 1 124.8 124.8a124.95 124.95 0 0 1-124.8-124.8"/></svg>`,
        },
        "ic:round-album": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10s10-4.48 10-10S17.52 2 12 2m0 14.5c-2.49 0-4.5-2.01-4.5-4.5S9.51 7.5 12 7.5s4.5 2.01 4.5 4.5s-2.01 4.5-4.5 4.5m0-5.5c-.55 0-1 .45-1 1s.45 1 1 1s1-.45 1-1s-.45-1-1-1"/></svg>`,
        },
        "mdi:playlist-music": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M15 6H3v2h12zm0 4H3v2h12zM3 16h8v-2H3zM17 6v8.18c-.31-.11-.65-.18-1-.18a3 3 0 0 0-3 3a3 3 0 0 0 3 3a3 3 0 0 0 3-3V8h3V6z"/></svg>`,
        },
        "mdi:search-web": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="m15.5 14l5 5l-1.5 1.5l-5-5v-.79l-.27-.28A6.47 6.47 0 0 1 9.5 16A6.5 6.5 0 0 1 3 9.5A6.5 6.5 0 0 1 9.5 3A6.5 6.5 0 0 1 16 9.5c0 1.61-.59 3.09-1.57 4.23l.28.27zm-6-9.5l-.55.03c-.24.52-.61 1.4-.88 2.47h2.86c-.27-1.07-.64-1.95-.88-2.47c-.18-.03-.36-.03-.55-.03M13.83 7a4.94 4.94 0 0 0-2.68-2.22c.24.53.55 1.3.78 2.22zM5.17 7h1.9c.23-.92.54-1.69.78-2.22A4.94 4.94 0 0 0 5.17 7M4.5 9.5c0 .5.08 1.03.23 1.5h2.14l-.12-1.5l.12-1.5H4.73c-.15.47-.23 1-.23 1.5m9.77 1.5c.15-.47.23-1 .23-1.5s-.08-1.03-.23-1.5h-2.14a9.5 9.5 0 0 1 0 3zm-6.4-3l-.12 1.5l.12 1.5h3.26a9.5 9.5 0 0 0 0-3zm1.63 6.5c.18 0 .36 0 .53-.03c.25-.52.63-1.4.9-2.47H8.07c.27 1.07.65 1.95.9 2.47zm4.33-2.5h-1.9c-.23.92-.54 1.69-.78 2.22A4.94 4.94 0 0 0 13.83 12m-8.66 0a4.94 4.94 0 0 0 2.68 2.22c-.24-.53-.55-1.3-.78-2.22z"/></svg>`,
        },
        "fluent:library-20-filled": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 20 20"><path fill="currentColor" d="M3.5 2C2.67 2 2 2.67 2 3.498v12.981c0 .828.671 1.498 1.5 1.498h1c.827 0 1.499-.67 1.499-1.498V3.498C5.999 2.67 5.327 2 4.499 2zm4.998 0c-.828 0-1.5.67-1.5 1.498v12.981c0 .828.672 1.498 1.5 1.498h1c.828 0 1.499-.67 1.499-1.498V3.498c0-.827-.671-1.498-1.5-1.498zm7.22 4.157a1.5 1.5 0 0 0-1.87-1.106l-.745.21a1.498 1.498 0 0 0-1.06 1.742l2.003 9.799a1.5 1.5 0 0 0 1.839 1.151l.985-.25c.79-.2 1.274-.994 1.092-1.787z"/></svg>`,
        },
        "fluent:search-20-filled": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 20 20"><path fill="currentColor" d="M8.5 3a5.5 5.5 0 0 1 4.383 8.823l4.147 4.147a.75.75 0 0 1-.976 1.133l-.084-.073l-4.147-4.147A5.5 5.5 0 1 1 8.5 3m0 1.5a4 4 0 1 0 0 8a4 4 0 0 0 0-8"/></svg>`,
        },
        "charm:menu-kebab": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 16 16"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"><circle cx="8" cy="2.5" r=".75"/><circle cx="8" cy="8" r=".75"/><circle cx="8" cy="13.5" r=".75"/></g></svg>`,
        },
        "mdi:map": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="m15 19l-6-2.11V5l6 2.11M20.5 3h-.16L15 5.1L9 3L3.36 4.9c-.21.07-.36.25-.36.48V20.5a.5.5 0 0 0 .5.5c.05 0 .11 0 .16-.03L9 18.9l6 2.1l5.64-1.9c.21-.1.36-.25.36-.48V3.5a.5.5 0 0 0-.5-.5"/></svg>`,
        },
        "gridicons:line-graph": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M3 19h18v2H3zm3-3c1.1 0 2-.9 2-2c0-.5-.2-1-.5-1.3L8.8 10H9c.5 0 1-.2 1.3-.5l2.7 1.4v.1c0 1.1.9 2 2 2s2-.9 2-2c0-.5-.2-.9-.5-1.3L17.8 7h.2c1.1 0 2-.9 2-2s-.9-2-2-2s-2 .9-2 2c0 .5.2 1 .5 1.3L15.2 9H15c-.5 0-1 .2-1.3.5L11 8.2V8c0-1.1-.9-2-2-2s-2 .9-2 2c0 .5.2 1 .5 1.3L6.2 12H6c-1.1 0-2 .9-2 2s.9 2 2 2"/></svg>`,
        },
        "ph:shuffle-bold": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 256 256"><path fill="currentColor" d="M240.49 175.51a12 12 0 0 1 0 17l-24 24a12 12 0 0 1-17-17L203 196h-2.09a76.17 76.17 0 0 1-61.85-31.83l-41.68-58.39A52.1 52.1 0 0 0 55.06 84H32a12 12 0 0 1 0-24h23.06a76.17 76.17 0 0 1 61.85 31.83l41.71 58.39A52.1 52.1 0 0 0 200.94 172H203l-3.52-3.51a12 12 0 0 1 17-17Zm-95.62-72.62a12 12 0 0 0 16.93-1.13A52 52 0 0 1 200.94 84H203l-3.52 3.51a12 12 0 0 0 17 17l24-24a12 12 0 0 0 0-17l-24-24a12 12 0 0 0-17 17L203 60h-2.09a76 76 0 0 0-57.2 26a12 12 0 0 0 1.16 16.89m-33.74 50.22a12 12 0 0 0-16.93 1.13A52 52 0 0 1 55.06 172H32a12 12 0 0 0 0 24h23.06a76 76 0 0 0 57.2-26a12 12 0 0 0-1.13-16.89"/></svg>`,
        },
        "ph:repeat-bold": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256"><path fill="currentColor" d="M24 128a72.08 72.08 0 0 1 72-72h108.69l-10.35-10.34a8 8 0 0 1 11.32-11.32l24 24a8 8 0 0 1 0 11.32l-24 24a8 8 0 0 1-11.32-11.32L204.69 72H96a56.06 56.06 0 0 0-56 56a8 8 0 0 1-16 0m200-8a8 8 0 0 0-8 8a56.06 56.06 0 0 1-56 56H51.31l10.35-10.34a8 8 0 0 0-11.32-11.32l-24 24a8 8 0 0 0 0 11.32l24 24a8 8 0 0 0 11.32-11.32L51.31 200H160a72.08 72.08 0 0 0 72-72a8 8 0 0 0-8-8"/></svg>`,
        },
        "ph:repeat-once": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256"><path fill="currentColor" d="M24 128a72.08 72.08 0 0 1 72-72h108.69l-10.35-10.34a8 8 0 0 1 11.32-11.32l24 24a8 8 0 0 1 0 11.32l-24 24a8 8 0 0 1-11.32-11.32L204.69 72H96a56.06 56.06 0 0 0-56 56a8 8 0 0 1-16 0m200-8a8 8 0 0 0-8 8a56.06 56.06 0 0 1-56 56H51.31l10.35-10.34a8 8 0 0 0-11.32-11.32l-24 24a8 8 0 0 0 0 11.32l24 24a8 8 0 0 0 11.32-11.32L51.31 200H160a72.08 72.08 0 0 0 72-72a8 8 0 0 0-8-8m-88 40a8 8 0 0 0 8-8v-48a8 8 0 0 0-11.58-7.16l-16 8a8 8 0 1 0 7.16 14.31l4.42-2.21V152a8 8 0 0 0 8 8"/></svg>`,
        },
        "fe:backward": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" fill-rule="evenodd" d="M12 12c0-.107.023-.216.072-.316a.617.617 0 0 1 .221-.253l6.86-4.35c.276-.174.623-.06.775.254a.725.725 0 0 1 .072.316v8.698c0 .36-.255.651-.57.651a.516.516 0 0 1-.277-.082l-6.86-4.349A.671.671 0 0 1 12 12v4.35c0 .36-.255.651-.57.651a.516.516 0 0 1-.277-.082l-6.86-4.349c-.275-.174-.374-.57-.221-.885a.617.617 0 0 1 .221-.253l6.86-4.35c.276-.174.623-.06.775.254a.725.725 0 0 1 .072.316z"/></svg>`,
        },
        "fe:forward": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M12 12V7.65c0-.11.025-.22.072-.316c.152-.314.5-.428.775-.253l6.86 4.349c.093.059.17.147.221.253c.153.314.054.71-.221.885l-6.86 4.35a.516.516 0 0 1-.277.081c-.315 0-.57-.291-.57-.651zc0 .23-.106.451-.293.57l-6.86 4.35a.516.516 0 0 1-.277.08c-.315 0-.57-.291-.57-.651V7.651c0-.11.025-.22.072-.316c.152-.314.5-.428.775-.253l6.86 4.349c.093.059.17.147.221.253c.049.1.072.209.072.315"/></svg>`,
        },
        "fe:pause": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" fill-rule="evenodd" d="M10 18a1 1 0 0 1-1 1H8a1 1 0 0 1-1-1V6a1 1 0 0 1 1-1h1a1 1 0 0 1 1 1zm7 0a1 1 0 0 1-1 1h-1a1 1 0 0 1-1-1V6a1 1 0 0 1 1-1h1a1 1 0 0 1 1 1z"/></svg>`,
        },
        "fe:play": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M6 5.912c0-.155.037-.307.107-.443c.23-.44.75-.599 1.163-.354l10.29 6.088c.14.083.255.206.332.355c.23.44.08.995-.332 1.239L7.27 18.885a.814.814 0 0 1-.415.115C6.383 19 6 18.592 6 18.089z"/></svg>`,
        },
        "clarity:heart-solid": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 36 36"><path fill="currentColor" d="M33 7.64c-1.34-2.75-5.2-5-9.69-3.69A9.87 9.87 0 0 0 18 7.72a9.87 9.87 0 0 0-5.31-3.77C8.19 2.66 4.34 4.89 3 7.64c-1.88 3.85-1.1 8.18 2.32 12.87C8 24.18 11.83 27.9 17.39 32.22a1 1 0 0 0 1.23 0c5.55-4.31 9.39-8 12.07-11.71c3.41-4.69 4.19-9.02 2.31-12.87" class="clr-i-solid clr-i-solid-path-1"/><path fill="none" d="M0 0h36v36H0z"/></svg>`,
        },
        "clarity:heart-line": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 36 36"><path fill="currentColor" d="M18 32.43a1 1 0 0 1-.61-.21C11.83 27.9 8 24.18 5.32 20.51C1.9 15.82 1.12 11.49 3 7.64c1.34-2.75 5.19-5 9.69-3.69A9.87 9.87 0 0 1 18 7.72a9.87 9.87 0 0 1 5.31-3.77c4.49-1.29 8.35.94 9.69 3.69c1.88 3.85 1.1 8.18-2.32 12.87c-2.68 3.67-6.51 7.39-12.07 11.71a1 1 0 0 1-.61.21M10.13 5.58A5.9 5.9 0 0 0 4.8 8.51c-1.55 3.18-.85 6.72 2.14 10.81A57.13 57.13 0 0 0 18 30.16a57.13 57.13 0 0 0 11.06-10.83c3-4.1 3.69-7.64 2.14-10.81c-1-2-4-3.59-7.34-2.65a8 8 0 0 0-4.94 4.2a1 1 0 0 1-1.85 0a7.93 7.93 0 0 0-4.94-4.2a7.31 7.31 0 0 0-2-.29" class="clr-i-outline clr-i-outline-path-1"/><path fill="none" d="M0 0h36v36H0z"/></svg>`,
        },
        "mdi:information": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M13 9h-2V7h2m0 10h-2v-6h2m-1-9A10 10 0 0 0 2 12a10 10 0 0 0 10 10a10 10 0 0 0 10-10A10 10 0 0 0 12 2"/></svg>`,
        },
        "ph:wave-sine-duotone": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 256 256"><g fill="currentColor"><path d="M232 128c-52 110.85-78 55.43-104 0Zm-208 0h104c-26-55.43-52-110.85-104 0" opacity=".2"/><path d="M239.24 131.4c-22 46.8-41.4 68.6-61.2 68.6c-25.1 0-40.73-33.32-57.28-68.6C107.7 103.56 92.9 72 78 72c-16.4 0-36.31 37.21-46.72 59.4a8 8 0 0 1-14.48-6.8C38.71 77.8 58.16 56 78 56c25.1 0 40.73 33.32 57.28 68.6C148.3 152.44 163.1 184 178 184c16.4 0 36.31-37.21 46.72-59.4a8 8 0 0 1 14.48 6.8Z"/></g></svg>`,
        },
        "bi:file-earmark-play": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 16 16"><g fill="currentColor"><path d="M6 6.883v4.234a.5.5 0 0 0 .757.429l3.528-2.117a.5.5 0 0 0 0-.858L6.757 6.454a.5.5 0 0 0-.757.43z"/><path d="M14 14V4.5L9.5 0H4a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2M9.5 3A1.5 1.5 0 0 0 11 4.5h2V14a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V2a1 1 0 0 1 1-1h5.5z"/></g></svg>`,
        },
        "iconoir:atom": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"><path d="M4.404 13.61C3.515 13.145 3 12.592 3 12c0-1.657 4.03-3 9-3s9 1.343 9 3c0 .714-.75 1.37-2 1.886m-7-2.876l.01-.011"/><path d="M16.883 6c-.005-1.023-.263-1.747-.797-2.02c-1.477-.751-4.503 2.23-6.76 6.658c-2.256 4.429-2.889 8.629-1.412 9.381c.527.269 1.252.061 2.07-.519"/><path d="M9.6 4.252c-.66-.386-1.243-.497-1.686-.271c-1.477.752-.844 4.952 1.413 9.38c2.256 4.43 5.282 7.41 6.759 6.658c1.312-.669.958-4.061-.722-7.917"/></g></svg>`,
        },
        "ic:round-info": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10s10-4.48 10-10S17.52 2 12 2m0 15c-.55 0-1-.45-1-1v-4c0-.55.45-1 1-1s1 .45 1 1v4c0 .55-.45 1-1 1m1-8h-2V7h2z"/></svg>`,
        },
        "fe:music": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" fill-rule="evenodd" d="M20 4v13a3 3 0 1 1-2-2.83V6h-8v13a3 3 0 1 1-2-2.83V4a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2"/></svg>`,
        },
        "ant-design:warning-outlined": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 1024 1024"><path fill="currentColor" d="M464 720a48 48 0 1 0 96 0a48 48 0 1 0-96 0m16-304v184c0 4.4 3.6 8 8 8h48c4.4 0 8-3.6 8-8V416c0-4.4-3.6-8-8-8h-48c-4.4 0-8 3.6-8 8m475.7 440l-416-720c-6.2-10.7-16.9-16-27.7-16s-21.6 5.3-27.7 16l-416 720C56 877.4 71.4 904 96 904h832c24.6 0 40-26.6 27.7-48m-783.5-27.9L512 239.9l339.8 588.2z"/></svg>`,
        },
        "ri:tools-fill": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M5.33 3.272a3.5 3.5 0 0 1 4.472 4.473L20.647 18.59l-2.122 2.121L7.68 9.867a3.5 3.5 0 0 1-4.472-4.474L5.444 7.63a1.5 1.5 0 0 0 2.121-2.121zm10.367 1.883l3.182-1.768l1.414 1.415l-1.768 3.181l-1.768.354l-2.12 2.121l-1.415-1.414l2.121-2.121zm-7.071 7.778l2.121 2.122l-4.95 4.95A1.5 1.5 0 0 1 3.58 17.99l.097-.107z"/></svg>`,
        },
        "charm:tick": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 16 16"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="m2.75 8.75l3.5 3.5l7-7.5"/></svg>`,
        },
        "heroicons-solid:selector": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 20 20"><path fill="currentColor" fill-rule="evenodd" d="M10 3a1 1 0 0 1 .707.293l3 3a1 1 0 0 1-1.414 1.414L10 5.414L7.707 7.707a1 1 0 0 1-1.414-1.414l3-3A1 1 0 0 1 10 3m-3.707 9.293a1 1 0 0 1 1.414 0L10 14.586l2.293-2.293a1 1 0 0 1 1.414 1.414l-3 3a1 1 0 0 1-1.414 0l-3-3a1 1 0 0 1 0-1.414" clip-rule="evenodd"/></svg>`,
        },
        "mdi:thunder": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M11 15H6l7-14v8h5l-7 14z"/></svg>`,
        },
        "gg:arrows-expand-up-right": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><g fill="currentColor"><path d="M13 5V3h8v8h-2V6.414l-5.364 5.364a1 1 0 0 1-1.414-1.414L17.586 5z"/><path fill-rule="evenodd" d="M5 13a2 2 0 0 0-2 2v4a2 2 0 0 0 2 2h4a2 2 0 0 0 2-2v-4a2 2 0 0 0-2-2zm0 2v4h4v-4z" clip-rule="evenodd"/></g></svg>`,
        },
        "gg:arrows-expand-down-left": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><g fill="currentColor"><path fill-rule="evenodd" d="M13 5a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v4a2 2 0 0 1-2 2h-4a2 2 0 0 1-2-2zm2 0h4v4h-4z" clip-rule="evenodd"/><path d="M5 13H3v8h8v-2H6.414l5.364-5.364a1 1 0 0 0-1.414-1.414L5 17.586z"/></g></svg>`,
        },
        "tabler:refresh": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 11A8.1 8.1 0 0 0 4.5 9M4 5v4h4m-4 4a8.1 8.1 0 0 0 15.5 2m.5 4v-4h-4"/></svg>`,
        },
        "material-symbols:lyrics": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M6 14h4v-2H6zm13-2q-1.25 0-2.125-.875T16 9q0-1.25.875-2.125T19 6q.275 0 .513.05t.487.125V1h4v2h-2v6q0 1.25-.875 2.125T19 12M6 11h7V9H6zm0-3h7V6H6zm0 10l-4 4V4q0-.825.588-1.412T4 2h11q.825 0 1.413.588T17 4v.425q-1.375.6-2.187 1.838T14 9q0 1.5.813 2.738T17 13.575V16q0 .825-.587 1.413T15 18z"/></svg>`,
        },
        "lucide:chevron-down": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="m6 9l6 6l6-6"/></svg>`,
        },
        "lucide:chevron-right": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="m9 18l6-6l-6-6"/></svg>`,
        },
        "lucide:chevron-up": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="m18 15l-6-6l-6 6"/></svg>`,
        },
        "mdi:format-font-size-decrease": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M5.12 14L7.5 7.67L9.87 14M6.5 5L1 19h2.25l1.12-3h6.25l1.13 3H14L8.5 5zM18 17l5-5.07l-1.41-1.43L19 13.1V7h-2v6.1l-2.59-2.6L13 11.93z"/></svg>`,
        },
        "mdi:format-font-size-increase": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M5.12 14L7.5 7.67L9.87 14M6.5 5L1 19h2.25l1.12-3h6.25l1.13 3H14L8.5 5zM18 7l-5 5.07l1.41 1.43L17 10.9V17h2v-6.1l2.59 2.6L23 12.07z"/></svg>`,
        },
        "heroicons-solid:volume-up": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><g fill="currentColor"><path d="M13.5 4.06c0-1.336-1.616-2.005-2.56-1.06l-4.5 4.5H4.508c-1.141 0-2.318.664-2.66 1.905A9.76 9.76 0 0 0 1.5 12c0 .898.121 1.768.35 2.595c.341 1.24 1.518 1.905 2.659 1.905h1.93l4.5 4.5c.945.945 2.561.276 2.561-1.06zm5.084 1.046a.75.75 0 0 1 1.06 0c3.808 3.807 3.808 9.98 0 13.788a.75.75 0 0 1-1.06-1.06a8.25 8.25 0 0 0 0-11.668a.75.75 0 0 1 0-1.06"/><path d="M15.932 7.757a.75.75 0 0 1 1.061 0a6 6 0 0 1 0 8.486a.75.75 0 0 1-1.06-1.061a4.5 4.5 0 0 0 0-6.364a.75.75 0 0 1 0-1.06"/></g></svg>`,
        },
        "system-uicons:audio-wave": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 21 21"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" d="M6.5 8.5v4m2-6v9m2-6v2m2-4v6.814m2-9.814v12"/></svg>`,
        },
        "bxs:video": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="currentColor" d="M18 7c0-1.103-.897-2-2-2H4c-1.103 0-2 .897-2 2v10c0 1.103.897 2 2 2h12c1.103 0 2-.897 2-2v-3.333L22 17V7l-4 3.333z"/></svg>`,
        },
        "dashicons:editor-video": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 20 20"><path fill="currentColor" d="M16 2h-3v1H7V2H4v15h3v-1h6v1h3zM6 3v1H5V3zm9 0v1h-1V3zm-2 1v5H7V4zM6 5v1H5V5zm9 0v1h-1V5zM6 7v1H5V7zm9 0v1h-1V7zM6 9v1H5V9zm9 0v1h-1V9zm-2 1v5H7v-5zm-7 1v1H5v-1zm9 0v1h-1v-1zm-9 2v1H5v-1zm9 0v1h-1v-1zm-9 2v1H5v-1zm9 0v1h-1v-1z"/></svg>`,
        },
        "akar-icons:link-chain": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"><path d="M13.544 10.456a4.368 4.368 0 0 0-6.176 0l-3.089 3.088a4.367 4.367 0 1 0 6.177 6.177L12 18.177"/><path d="M10.456 13.544a4.368 4.368 0 0 0 6.176 0l3.089-3.088a4.367 4.367 0 1 0-6.177-6.177L12 5.823"/></g></svg>`,
        },
        "ant-design:delete-outlined": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 1024 1024"><path fill="currentColor" d="M360 184h-8c4.4 0 8-3.6 8-8zh304v-8c0 4.4 3.6 8 8 8h-8v72h72v-80c0-35.3-28.7-64-64-64H352c-35.3 0-64 28.7-64 64v80h72zm504 72H160c-17.7 0-32 14.3-32 32v32c0 4.4 3.6 8 8 8h60.4l24.7 523c1.6 34.1 29.8 61 63.9 61h454c34.2 0 62.3-26.8 63.9-61l24.7-523H888c4.4 0 8-3.6 8-8v-32c0-17.7-14.3-32-32-32M731.3 840H292.7l-24.2-512h487z"/></svg>`,
        },
        "icon-park-outline:full-screen-one": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 48 48"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="4" d="m6 6l10 9.9m-10 26L16 32m26 9.9L32.1 32m9.8-26L32 15.9M33 6h9v9m0 18v9h-9m-18 0H6v-9m0-18V6h9"/></svg>`,
        },
        "material-symbols:add": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M11 13H5v-2h6V5h2v6h6v2h-6v6h-2z"/></svg>`,
        },
        "material-symbols:edit-outline": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M5 19h1.425L16.2 9.225L14.775 7.8L5 17.575zm-2 2v-4.25L16.2 3.575q.3-.275.663-.425t.762-.15t.775.15t.65.45L20.425 5q.3.275.438.65T21 6.4q0 .4-.137.763t-.438.662L7.25 21zM19 6.4L17.6 5zm-3.525 2.125l-.7-.725L16.2 9.225z"/></svg>`,
        },
        "material-symbols:save-outline": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="currentColor" d="M21 7v12q0 .825-.587 1.413T19 21H5q-.825 0-1.412-.587T3 19V5q0-.825.588-1.412T5 3h12zm-2 .85L16.15 5H5v14h14zM12 18q1.25 0 2.125-.875T15 15t-.875-2.125T12 12t-2.125.875T9 15t.875 2.125T12 18m-6-8h9V6H6zM5 7.85V19V5z"/></svg>`,
        },
        "material-symbols:close": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="currentColor" d="M6.4 19L5 17.6l5.6-5.6L5 6.4L6.4 5l5.6 5.6L17.6 5L19 6.4L13.4 12l5.6 5.6l-1.4 1.4l-5.6-5.6z"/></svg>`,
        },
        "f7:speaker-2-fill": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 56 56"><path fill="currentColor" d="M26 49.451c1.581 0 2.721-1.163 2.721-2.721V9.637c0-1.558-1.14-2.86-2.767-2.86c-1.14 0-1.907.511-3.14 1.674l-10.256 9.698a.911.911 0 0 1-.604.209H5.046C1.768 18.358 0 20.148 0 23.637v9.023c0 3.489 1.767 5.28 5.047 5.28h6.907c.232 0 .441.07.604.209l10.256 9.79c1.116 1.047 2.046 1.512 3.186 1.512m22.489-4.744c.906.604 2.023.372 2.674-.558c3.07-4.28 4.837-9.977 4.837-15.93c0-5.977-1.744-11.675-4.837-15.954c-.674-.907-1.768-1.14-2.674-.535c-.884.605-1.024 1.744-.326 2.744c2.535 3.721 4.093 8.605 4.093 13.744c0 5.14-1.512 10.07-4.117 13.745c-.65 1-.534 2.14.35 2.744m-9.28-6.28c.791.559 1.93.373 2.605-.534c1.814-2.442 2.907-6.024 2.907-9.675c0-3.65-1.116-7.209-2.907-9.697c-.674-.907-1.79-1.093-2.605-.535c-1.023.674-1.14 1.86-.395 2.883c1.349 1.814 2.163 4.582 2.163 7.35c0 2.767-.86 5.534-2.186 7.371c-.698 1-.582 2.14.418 2.838"/></svg>`,
        },
        "hugeicons:download-05": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M12 15V5m0 10c-.7 0-2.008-1.994-2.5-2.5M12 15c.7 0 2.008-1.994 2.5-2.5M5 19h14" color="currentColor"/></svg>`,
        },
        "material-symbols:folder": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="currentColor" d="M4 20q-.825 0-1.412-.587T2 18V6q0-.825.588-1.412T4 4h6l2 2h8q.825 0 1.413.588T22 8v10q0 .825-.587 1.413T20 20z"/></svg>`,
        },
        "ic:round-star-outline": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="currentColor" d="m19.65 9.04l-4.84-.42l-1.89-4.45c-.34-.81-1.5-.81-1.84 0L9.19 8.63l-4.83.41c-.88.07-1.24 1.17-.57 1.75l3.67 3.18l-1.1 4.72c-.2.86.73 1.54 1.49 1.08l4.15-2.5l4.15 2.51c.76.46 1.69-.22 1.49-1.08l-1.1-4.73l3.67-3.18c.67-.58.32-1.68-.56-1.75M12 15.4l-3.76 2.27l1-4.28l-3.32-2.88l4.38-.38L12 6.1l1.71 4.04l4.38.38l-3.32 2.88l1 4.28z"/></svg>`,
        },
        "tabler:world-download": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"><path d="M21 12a9 9 0 1 0-9 9M3.6 9h16.8M3.6 15H12"/><path d="M11.578 3a17 17 0 0 0 0 18M12.5 3c1.719 2.755 2.5 5.876 2.5 9m3 2v7m-3-3l3 3l3-3"/></g></svg>`,
        },
        "ic:twotone-downloading": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="currentColor" d="M18.32 4.26A9.949 9.949 0 0 0 13 2.05v2.02c1.46.18 2.79.76 3.9 1.62zM19.93 11h2.02c-.2-2.01-1-3.84-2.21-5.32L18.31 7.1a7.941 7.941 0 0 1 1.62 3.9m-1.62 5.9l1.43 1.43a9.981 9.981 0 0 0 2.21-5.32h-2.02a7.945 7.945 0 0 1-1.62 3.89M13 19.93v2.02c2.01-.2 3.84-1 5.32-2.21l-1.43-1.43c-1.1.86-2.43 1.44-3.89 1.62M13 12V7h-2v5H7l5 5l5-5zm-2 7.93v2.02c-5.05-.5-9-4.76-9-9.95s3.95-9.45 9-9.95v2.02C7.05 4.56 4 7.92 4 12s3.05 7.44 7 7.93"/></svg>`,
        },
        "tabler:layout-sidebar-left-collapse": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"><path d="M4 6a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2zm5-2v16"/><path d="m15 10l-2 2l2 2"/></g></svg>`,
        },
        "tabler:layout-sidebar-left-expand": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"><path d="M4 6a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2zm5-2v16"/><path d="m14 10l2 2l-2 2"/></g></svg>`,
        },
        "fad:mono": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 256 256"><path fill="currentColor" fill-rule="evenodd" d="M127 210c-44.735 0-81-36.265-81-81s36.265-81 81-81s81 36.265 81 81s-36.265 81-81 81m1-21c34.794 0 63-27.087 63-60.5S162.794 68 128 68s-63 27.087-63 60.5S93.206 189 128 189"/></svg>`,
        },
        "fad:stereo": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 256 256"><path fill="currentColor" fill-rule="evenodd" d="M128.802 95.03c-9.229-9.369-22.39-15.228-37-15.228c-27.92 0-50.555 21.402-50.555 47.803s22.634 47.802 50.555 47.802c14.711 0 27.954-5.94 37.193-15.423c-12.232-16.88-14.177-19.888-14.177-32.38c0-12.016 5.924-18.458 14.19-31.142c6.753 13.293 13.629 19.445 13.629 31.538c0 12.802-6.03 20.525-13.402 32.614c9.206 9.115 22.185 14.793 36.567 14.793c27.922 0 50.556-21.401 50.556-47.802c0-26.4-22.634-47.803-50.556-47.803c-14.608 0-27.77 5.86-37 15.228M128 75.374C138.501 68.202 151.252 64 165 64c35.899 0 65 28.654 65 64s-29.101 64-65 64c-13.748 0-26.499-4.202-37-11.374C117.499 187.798 104.748 192 91 192c-35.899 0-65-28.654-65-64s29.101-64 65-64c13.748 0 26.499 4.202 37 11.374"/></svg>`,
        },
        "bi:file-earmark-text": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 16 16"><g fill="currentColor"><path d="M5.5 7a.5.5 0 0 0 0 1h5a.5.5 0 0 0 0-1zM5 9.5a.5.5 0 0 1 .5-.5h5a.5.5 0 0 1 0 1h-5a.5.5 0 0 1-.5-.5m0 2a.5.5 0 0 1 .5-.5h2a.5.5 0 0 1 0 1h-2a.5.5 0 0 1-.5-.5"/><path d="M9.5 0H4a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2V4.5zm0 1v2A1.5 1.5 0 0 0 11 4.5h2V14a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V2a1 1 0 0 1 1-1z"/></g></svg>`,
        },
        "ic:baseline-plus": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="currentColor" d="M19 12.998h-6v6h-2v-6H5v-2h6v-6h2v6h6z"/></svg>`,
        },
        "ic:baseline-edit": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="currentColor" d="M3 17.25V21h3.75L17.81 9.94l-3.75-3.75zM20.71 7.04a.996.996 0 0 0 0-1.41l-2.34-2.34a.996.996 0 0 0-1.41 0l-1.83 1.83l3.75 3.75z"/></svg>`,
        },
        "ph:plus-fill": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 256 256"><path fill="currentColor" d="M208 32H48a16 16 0 0 0-16 16v160a16 16 0 0 0 16 16h160a16 16 0 0 0 16-16V48a16 16 0 0 0-16-16m-24 104h-48v48a8 8 0 0 1-16 0v-48H72a8 8 0 0 1 0-16h48V72a8 8 0 0 1 16 0v48h48a8 8 0 0 1 0 16"/></svg>`,
        },
        "transpose-up": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" fill="currentColor" viewBox="0 0 12 12"> <path d="M7.582 1.001c-.4.094-2.787.684-3.435.84a.044.044 0 0 0-.034.043v6.189a.044.044 0 0 1-.029.041l-.895.324C2.532 8.673 2 8.855 2 9.738c0 .715.518 1.039.835 1.1.11.019.22.027.33.024a2.24 2.24 0 0 0 1.047-.313c.606-.391.606-.391.606-1.097V4.019a.044.044 0 0 1 .033-.042l2.751-.727a.044.044 0 0 0 .033-.043V1.044a.044.044 0 0 0-.053-.043Z"/> <g clip-path="url(#a)">   <path d="M10.796 6.626c0 .117-.039.22-.116.309l-.236.257a.375.375 0 0 1-.286.13.36.36 0 0 1-.283-.13l-.923-1.004v2.411a.38.38 0 0 1-.118.29.4.4 0 0 1-.285.111h-.402a.4.4 0 0 1-.284-.111.38.38 0 0 1-.118-.29V6.188L6.82 7.192a.36.36 0 0 1-.283.13.36.36 0 0 1-.283-.13l-.235-.257a.44.44 0 0 1-.12-.309c0-.12.04-.225.12-.311l2.045-2.23a.357.357 0 0 1 .283-.127c.113 0 .209.042.286.127l2.046 2.23a.46.46 0 0 1 .116.311Z"/> </g> <defs>   <clipPath id="a">     <path d="M5.827 3.958h5.042V9H5.827z"/>   </clipPath> </defs>/svg>`,
        },
        "transpose-down": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" fill="currentColor" viewBox="0 0 12 12">  <path d="M7.582 1.001c-.4.094-2.787.684-3.435.84a.044.044 0 0 0-.034.043v6.189a.044.044 0 0 1-.029.041l-.895.324C2.532 8.673 2 8.855 2 9.738c0 .715.518 1.039.835 1.1.11.019.22.027.33.024a2.24 2.24 0 0 0 1.047-.313c.606-.391.606-.391.606-1.097V4.019a.044.044 0 0 1 .033-.042l2.751-.727a.044.044 0 0 0 .033-.043V1.044a.044.044 0 0 0-.053-.043Z"/>  <g clip-path="url(#a)">    <path d="M10.628 6.342a.424.424 0 0 0-.108-.287l-.22-.24a.35.35 0 0 0-.266-.12.335.335 0 0 0-.263.12l-.86.935V4.504c0-.11-.037-.2-.11-.27a.372.372 0 0 0-.265-.103H8.16a.372.372 0 0 0-.265.104.354.354 0 0 0-.11.27V6.75l-.86-.935a.335.335 0 0 0-.264-.12.335.335 0 0 0-.263.12l-.22.24a.41.41 0 0 0-.11.287c0 .113.036.21.11.29L8.085 8.71a.333.333 0 0 0 .263.118c.106 0 .194-.04.267-.118l1.905-2.077a.428.428 0 0 0 .108-.29Z"/></g><defs><clipPath id="a"><path d="M6 8.827h4.696V4.131H6z"/></clipPath></defs></svg>`,
        },
        "mingcute:time-line": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><g fill="none"><path d="m12.593 23.258l-.011.002l-.071.035l-.02.004l-.014-.004l-.071-.035q-.016-.005-.024.005l-.004.01l-.017.428l.005.02l.01.013l.104.074l.015.004l.012-.004l.104-.074l.012-.016l.004-.017l-.017-.427q-.004-.016-.017-.018m.265-.113l-.013.002l-.185.093l-.01.01l-.003.011l.018.43l.005.012l.008.007l.201.093q.019.005.029-.008l.004-.014l-.034-.614q-.005-.018-.02-.022m-.715.002a.02.02 0 0 0-.027.006l-.006.014l-.034.614q.001.018.017.024l.015-.002l.201-.093l.01-.008l.004-.011l.017-.43l-.003-.012l-.01-.01z"/><path fill="currentColor" d="M12 2c5.523 0 10 4.477 10 10s-4.477 10-10 10S2 17.523 2 12S6.477 2 12 2m0 2a8 8 0 1 0 0 16a8 8 0 0 0 0-16m0 2a1 1 0 0 1 .993.883L13 7v4.586l2.707 2.707a1 1 0 0 1-1.32 1.497l-.094-.083l-3-3a1 1 0 0 1-.284-.576L11 12V7a1 1 0 0 1 1-1"/></g></svg>`,
        },
        "material-symbols:help": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24"><path fill="currentColor" d="M11.95 18q.525 0 .888-.363t.362-.887t-.362-.888t-.888-.362t-.887.363t-.363.887t.363.888t.887.362m-.9-3.85h1.85q0-.825.188-1.3t1.062-1.3q.65-.65 1.025-1.238T15.55 8.9q0-1.4-1.025-2.15T12.1 6q-1.425 0-2.312.75T8.55 8.55l1.65.65q.125-.45.563-.975T12.1 7.7q.8 0 1.2.438t.4.962q0 .5-.3.938t-.75.812q-1.1.975-1.35 1.475t-.25 1.825M12 22q-2.075 0-3.9-.787t-3.175-2.138T2.788 15.9T2 12t.788-3.9t2.137-3.175T8.1 2.788T12 2t3.9.788t3.175 2.137T21.213 8.1T22 12t-.788 3.9t-2.137 3.175t-3.175 2.138T12 22"/></svg>`,
        },
        "ant-design:bulb-outlined": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 1024 1024"><path fill="currentColor" d="M632 888H392c-4.4 0-8 3.6-8 8v32c0 17.7 14.3 32 32 32h192c17.7 0 32-14.3 32-32v-32c0-4.4-3.6-8-8-8M512 64c-181.1 0-328 146.9-328 328c0 121.4 66 227.4 164 284.1V792c0 17.7 14.3 32 32 32h264c17.7 0 32-14.3 32-32V676.1c98-56.7 164-162.7 164-284.1c0-181.1-146.9-328-328-328m127.9 549.8L604 634.6V752H420V634.6l-35.9-20.8C305.4 568.3 256 484.5 256 392c0-141.4 114.6-256 256-256s256 114.6 256 256c0 92.5-49.4 176.3-128.1 221.8"/></svg>`,
        },
        "qlementine-icons:sort-desc-16": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" d="M3 1.5a.5.5 0 0 0-1 0v11.8L.85 12.15a.5.5 0 0 0-.707.707l2 2a.5.5 0 0 0 .707 0l2-2a.5.5 0 0 0-.707-.707l-1.15 1.15V1.5zM7.5 12a.5.5 0 0 0 0 1h3a.5.5 0 0 0 0-1zm0-3a.5.5 0 0 0 0 1h4a.5.5 0 0 0 0-1zm0-3a.5.5 0 0 0 0 1h6a.5.5 0 0 0 0-1zm8-2a.5.5 0 0 0 0-1h-8a.5.5 0 0 0 0 1z"/></svg>`,
        },
        "mdi:hide": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M11.83 9L15 12.16V12a3 3 0 0 0-3-3zm-4.3.8l1.55 1.55c-.05.21-.08.42-.08.65a3 3 0 0 0 3 3c.22 0 .44-.03.65-.08l1.55 1.55c-.67.33-1.41.53-2.2.53a5 5 0 0 1-5-5c0-.79.2-1.53.53-2.2M2 4.27l2.28 2.28l.45.45C3.08 8.3 1.78 10 1 12c1.73 4.39 6 7.5 11 7.5c1.55 0 3.03-.3 4.38-.84l.43.42L19.73 22L21 20.73L3.27 3M12 7a5 5 0 0 1 5 5c0 .64-.13 1.26-.36 1.82l2.93 2.93c1.5-1.25 2.7-2.89 3.43-4.75c-1.73-4.39-6-7.5-11-7.5c-1.4 0-2.74.25-4 .7l2.17 2.15C10.74 7.13 11.35 7 12 7"/></svg>`,
        },
        "mdi:show": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M12 9a3 3 0 0 0-3 3a3 3 0 0 0 3 3a3 3 0 0 0 3-3a3 3 0 0 0-3-3m0 8a5 5 0 0 1-5-5a5 5 0 0 1 5-5a5 5 0 0 1 5 5a5 5 0 0 1-5 5m0-12.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5"/></svg>`,
        },
        "pajamas:scroll-down": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="currentColor" fill-rule="evenodd" d="M10 14a2 2 0 1 1-4 0a2 2 0 0 1 4 0m1.78-8.841a.75.75 0 0 0-1.06 0l-1.97 1.97V.75a.75.75 0 0 0-1.5 0v6.379l-1.97-1.97a.75.75 0 0 0-1.06 1.06l3.25 3.25L8 10l.53-.53l3.25-3.25a.75.75 0 0 0 0-1.061" clip-rule="evenodd"/></svg>`,
        },
        "material-symbols:grid-view-rounded": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M5 11q-.825 0-1.412-.587T3 9V5q0-.825.588-1.412T5 3h4q.825 0 1.413.588T11 5v4q0 .825-.587 1.413T9 11zm0 10q-.825 0-1.412-.587T3 19v-4q0-.825.588-1.412T5 13h4q.825 0 1.413.588T11 15v4q0 .825-.587 1.413T9 21zm10-10q-.825 0-1.412-.587T13 9V5q0-.825.588-1.412T15 3h4q.825 0 1.413.588T21 5v4q0 .825-.587 1.413T19 11zm0 10q-.825 0-1.412-.587T13 19v-4q0-.825.588-1.412T15 13h4q.825 0 1.413.588T21 15v4q0 .825-.587 1.413T19 21z"/></svg>`,
        },
        "mdi:information-off": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M22.1 21.5L2.4 1.7L1.1 3l3 3C2.8 7.6 2 9.7 2 12c0 5.5 4.5 10 10 10c2.3 0 4.4-.8 6-2.1l2.8 2.8zM13 17h-2v-4.1l2 2zm-2-9.2L6.7 3.5C8.3 2.6 10.1 2 12 2c5.5 0 10 4.5 10 10c0 1.9-.6 3.7-1.5 5.3L12.2 9h.8V7h-2z"/></svg>`,
        },
        "mynaui:one-circle-solid": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><rect width="24" height="24" fill="none"/><path fill="currentColor" d="M2.25 12c0-5.385 4.365-9.75 9.75-9.75s9.75 4.365 9.75 9.75s-4.365 9.75-9.75 9.75S2.25 17.385 2.25 12M13.5 8a.75.75 0 0 0-1.191-.607l-2.75 2a.75.75 0 1 0 .882 1.214l1.56-1.134V16a.75.75 0 1 0 1.5 0z"/></svg>`,
        },
        "mynaui:one-circle-solid-off": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><rect width="24" height="24" fill="none"/><path fill="currentColor" d="M2.281,1.05 L10.194,8.962 L10.192,8.964 L11.263,10.04 L11.268,10.037 L12.001,10.77 L12.001,10.781 L13.5,12.289 L13.5,12.269 L19.492,18.261 L19.469,18.287 L22,20.831 L20.7,22.031 L18.213,19.544 C16.406,21.034 14.302,21.665 12,21.781 C6.615,21.781 2.25,17.416 2.25,12.031 C2.225,9.514 3.125,7.829 4.487,5.818 L1,2.331 L2.281,1.05 z M12.001,13.332 L12.001,16.031 C12.001,16.445 12.337,16.781 12.751,16.781 C13.165,16.781 13.501,16.445 13.501,16.031 L13.501,14.832 L12.001,13.332 z M12,2.281 C10.656,2.312 9.594,2.563 7.326,3.514 L11.675,7.885 L12.309,7.424 L12.691,7.283 L13.091,7.363 C13.342,7.491 13.5,7.749 13.5,8.031 L13.5,9.719 L20.524,16.779 C21.355,15.336 21.671,13.68 21.75,12.031 C21.75,6.646 17.384,2.156 12,2.281 z"/></svg>`,
        },
        "heroicons-solid:sort-descending": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 20 20"><rect width="20" height="20" fill="none"/><path fill="currentColor" d="M3 3a1 1 0 0 0 0 2h11a1 1 0 1 0 0-2zm0 4a1 1 0 0 0 0 2h7a1 1 0 1 0 0-2zm0 4a1 1 0 1 0 0 2h4a1 1 0 1 0 0-2zm12-3a1 1 0 1 0-2 0v5.586l-1.293-1.293a1 1 0 0 0-1.414 1.414l3 3a1 1 0 0 0 1.414 0l3-3a1 1 0 0 0-1.414-1.414L15 13.586z"/></svg>`,
        },
        "iconamoon:settings-fill": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><rect width="24" height="24" fill="none"/><path fill="currentColor" fill-rule="evenodd" d="M9 3.223a1 1 0 0 1 .777-.975a10 10 0 0 1 4.445.002a1 1 0 0 1 .778.975v1.847a1 1 0 0 0 1.5.866l1.601-.925a1 1 0 0 1 1.234.187a10.1 10.1 0 0 1 2.221 3.848a1 1 0 0 1-.455 1.162l-1.601.924a1 1 0 0 0 0 1.732l1.6.923a1 1 0 0 1 .455 1.161a10 10 0 0 1-2.22 3.851a1 1 0 0 1-1.234.186l-1.601-.925a1 1 0 0 0-1.5.866v1.85a1 1 0 0 1-.778.974a10 10 0 0 1-4.445-.002A1 1 0 0 1 9 20.775v-1.847a1 1 0 0 0-1.5-.866l-1.601.925a1 1 0 0 1-1.234-.187A10 10 0 0 1 3.34 17a10 10 0 0 1-.896-2.048a1 1 0 0 1 .455-1.162l1.6-.924a1 1 0 0 0 0-1.732l-1.598-.923a1 1 0 0 1-.456-1.161a10 10 0 0 1 2.22-3.85a1 1 0 0 1 1.233-.187l1.602.925A1 1 0 0 0 9 5.072zM12 15a3 3 0 1 0 0-6a3 3 0 0 0 0 6" clip-rule="evenodd"/></svg>`,
        },
        "iconamoon:zoom-in": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="m21 21l-4.343-4.343m0 0A8 8 0 1 0 5.343 5.343a8 8 0 0 0 11.314 11.314M11 8v6m-3-3h6"/></svg>`,
        },
        "iconamoon:zoom-out": {
            svg: `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="m21 21l-4.343-4.343m0 0A8 8 0 1 0 5.343 5.343a8 8 0 0 0 11.314 11.314M8 11h6"/></svg>`,
        },
    };

    $: data = icon && icons[icon]?.svg;
    $: viewBox = data && extractViewBox(data);

    $: elements = data?.replace(/<svg[ \n]([^>]*)>/, "")?.replace("</svg>", "");
    function extractViewBox(svg) {
        if (!svg) return "0 0 20 20";
        const regex = /viewBox="([\d\- \.]+)"/;
        const res = regex.exec(svg);
        if (!res) return "0 0 20 20";
        return res[1];
    }
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<!-- svelte-ignore a11y-click-events-have-key-events -->
<div
    use:optionalTippy={tooltip}
    on:click={(e) => {
        if (onClick) {
            e.stopPropagation();
            onClick(e);
        }
    }}
    class={className}
    class:boxed
    class:clickable={onClick}
    class:disabled
    style={`cursor: ${cursor}`}
>
    <svg
        xmlns="http://www.w3.org/2000/svg"
        {width}
        {height}
        {viewBox}
        {stroke}
        {fill}
        style="color: {color};"
        {...$$restProps}
    >
        {#if elements}
            {@html elements}
        {/if}
    </svg>
</div>

<style lang="scss">
    div {
        display: flex;
        align-items: center;

        &.boxed {
            border-radius: 4px;
            padding: 6px;
            &:hover {
                background-color: var(--icon-boxed-hover-bg);
            }
        }

        &.clickable {
            color: var(--icon-secondary);

            // &:hover {
            //     color: var(--icon-secondary-hover);
            // }
            &:active {
                opacity: 0.9;
                transform: scale(0.95);
            }
        }

        &.disabled {
            pointer-events: none;
            color: var(--icon-disabled) !important;
        }

        svg {
            stroke: none;
        }
    }
</style>
