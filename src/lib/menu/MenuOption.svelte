<script>
  import { onMount, getContext } from "svelte";

  export let isDisabled = false;
  export let isDestructive = false;
  export let text = "";
  export let confirmText = "";
  export let isConfirming = false;

  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();

  const { dispatchClick } = getContext("menu");

  const handleClick = (e) => {
    if (isDisabled) return;

    dispatch("click");
    dispatchClick();
  };
</script>

<div
  class:disabled={isDisabled}
  class:destructive={isDestructive}
  class:confirming={isConfirming}
  on:click={handleClick}
>
  {#if isConfirming}
    {confirmText}
  {:else if text}
    {text}
  {:else}
    <slot />
  {/if}
</div>

<style lang="scss">
  div {
    padding: 0px 10px;
    cursor: default;
    font-size: 13.5px;
    display: flex;
    align-items: center;
    grid-gap: 5px;
    color: rgb(255, 255, 255);
    border-radius: 5px;
    &:hover {
      background: #0e6dd2;
      color: white;
    }
    &.disabled {
      color: rgba(174, 174, 174, 0.4);
      &:hover {
        background: transparent;
      }
    }
    &.confirming {
      background: #d2630e;
      &:hover {
        background: #d2630e !important;
      }
    }
    &.destructive {
      &:hover {
        background: #d20e32;
      }
    }
  }
</style>
