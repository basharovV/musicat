<script lang="ts">
  import { isDraggingFiles, isInfoPopupOpen } from "./data/store";

  import Dropzone from "./lib/Dropzone.svelte";
  import InfoPopup from "./lib/InfoPopup.svelte";
  import Library from "./lib/Library.svelte";
  import Sidebar from "./lib/Sidebar.svelte";
  import { startMenuListener } from "./window/EventListener";

  startMenuListener();

  function onDragEnter(e) {
    e.preventDefault();

    e.dataTransfer.dropEffect = "copy";
    isDraggingFiles.set(true);
    console.log("drag enter");
  }

  function onPageClick() {
    $isInfoPopupOpen = false;
  }
</script>

<!-- <svelte:body on:click={onPageClick} /> -->

{#if $isInfoPopupOpen}
  <div class="info">
    <InfoPopup />
  </div>
{/if}

{#if $isDraggingFiles}
  <Dropzone />
{/if}

<main on:dragenter={onDragEnter}>
  <Sidebar />
  <Library />
</main>

<style lang="scss">
  main {
    display: grid;
    grid-template-columns: auto 1fr;
    width: 100vw;
    height: 100vh;
    opacity: 1;
    position: relative;
  }

  .info {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 2;
  }
</style>
