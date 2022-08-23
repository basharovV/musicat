<script lang="ts">
  import { isInfoPopupOpen } from "../data/store";

  let version = "0.1.0";

  function onPageClick() {
    $isInfoPopupOpen = false;
  }
</script>

<container on:click={onPageClick}>
  <div>
    <!-- <img src="images/cd6.gif" /> -->
    <h1>Musicat</h1>
    <p>🎹 for the love of music 🥁</p>
    <small>built by Vyacheslav Basharov</small>
    <br/>
    <small>version {version}</small>
  </div>
</container>

<style lang="scss">
  container {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: rgba(0, 0, 0, 0.187);
  }
  div {
    border-radius: 5px;
    position: relative;
    padding: 2em 4em;
    background: rgb(55, 55, 55);
    color: white;
    border: 1px solid rgb(114, 114, 114);

    h1 {
      font-family: "2Peas";
    }

    small {
        opacity: 0.5;
    }
  }
</style>
