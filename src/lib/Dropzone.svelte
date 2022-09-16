<script lang="ts">
  import { listenForFileDrop } from "../window/EventListener";

  import { addFolder, addSong } from "../data/LibraryImporter";

  import type { Event } from "@tauri-apps/api/event";
  import { isAudioFile } from "../utils/FileUtils";
  import { isDraggingExternalFiles } from "../data/store";

  async function startDragListener() {
    const filedropEvent: Event<any> = await listenForFileDrop();
    filedropEvent.payload.forEach((entry) => {
      // check dir
      if (isAudioFile(entry)) {
        const file = entry.split("/").pop();
        addSong(entry, file, true);
      } else {
        addFolder(entry);
      }
    });
    isDraggingExternalFiles.set(false);
  }
  startDragListener();

  function onDragLeave(e) {
    e.preventDefault();
    isDraggingExternalFiles.set(false);
    console.log("drag leave");
  }
</script>

<div class="dropzone" on:dragleave={onDragLeave}>
  <img alt="Drop the mic" src="images/drop-the-mic.gif" />
</div>

<style lang="scss">
  .dropzone {
    display: flex;
    align-items: center;
    justify-content: center;
    position: absolute;
    width: 80vw;
    height: 80vh;
    left: 0;
    right: 0;
    top: 0;
    bottom: 0;
    margin: auto;
    background-color: #00a4ba;
    z-index: 20;
    border: 3px dashed rgb(211, 57, 57);
    border-radius: 5px;
    opacity: 1;
    transition: opacity 0.2s ease-in-out;
    * {
      pointer-events: none;
    }
    img {
      width: 300px;
    }
  }
</style>
