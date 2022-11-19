<script lang="ts">
    import { fly } from "svelte/transition";
    import type { ArtistProject } from "../../App";
    import { db } from "../../data/db";
    import { autoWidth } from "../../utils/AutoWidth";
    import { flip } from "svelte/animate";
    import { quadInOut } from "svelte/easing";

    export let artist: ArtistProject;

    $: members = artist?.members ?? [];

    let newMember = "";
    let isEditEnabled = false;

    function addMember() {
        if (members.includes(newMember)) return;

        members.push(newMember);

        db.artistProjects.update(artist.name, {
            members: [...members]
        });
        newMember = "";
    }

    function removeMember(memberName) {
        members.splice(
            members.findIndex((m) => m === memberName),
            1
        );
        db.artistProjects.update(artist.name, {
            members
        });
    }
</script>

<container>
    <p class="label">project members:</p>
    {#if members.length}
        <div class="members">
            {#each members as member (member)}
                <div
                    animate:flip={{ duration: 180, easing: quadInOut }}
                    class="member"
                    class:editable={isEditEnabled}
                >
                    <p>{member}</p>
                    {#if isEditEnabled}
                        <iconify-icon
                            in:fly={{ duration: 150, x: -40 }}
                            icon="mingcute:close-circle-fill"
                            on:click={() => removeMember(member)}
                        />
                    {/if}
                </div>
            {/each}
        </div>
    {:else if !isEditEnabled}
    {/if}
    {#if isEditEnabled}
        <form on:submit|preventDefault={addMember}>
            <input
                use:autoWidth
                autofocus
                class="member-input"
                bind:value={newMember}
                placeholder="Add member"
            />
        </form>
    {/if}
    <div
        class="edit-button"
        on:click={() => {
            isEditEnabled = !isEditEnabled;
        }}
    >
        <iconify-icon icon="clarity:edit-solid" />
    </div>
</container>

<style lang="scss">
    container {
        display: flex;
        flex-direction: row;
        align-items: center;
        gap: 10px;
        justify-content: flex-start;
    }
    p {
        margin: 0;
    }

    input {
        padding: 0;
        font-size: 14px;
        outline: none;
        background: none;
        min-width: 120px;
        border: none;
        &::placeholder {
            color: rgb(105, 105, 105);
        }
    }

    .members {
        display: flex;
        flex-direction: row;
        gap: 10px;
        align-items: center;
    }

    .member {
        position: relative;
        display: flex;
        flex-direction: row;
        align-items: center;
        gap: 5px;
        padding: 0 1px;

        iconify-icon {
            display: none;
        }

        &.editable {
            iconify-icon {
                display: flex;
                &:hover {
                    opacity: 0.6;
                }
            }
            background: rgb(62, 61, 61);
            border-radius: 4px;
            border: 1px solid rgb(76, 72, 72);
            padding: 0 6px;
        }

        &:hover {
        }
        p {
            margin: 0;
        }

        &:not(:last-child) {
            p::after {
                content: ",";
                position: absolute;
                right: -6px;
                top: 0;
            }
        }
    }

    .edit-button {
        border-radius: 4px;
        width: 20px;
        height: 20px;
        display: flex;
        align-items: center;
        justify-content: center;
        color: grey;
        &:hover {
            background-color: rgba(170, 170, 170, 0.418);
            color: white;
        }
    }

    .label {
        opacity: 0.6;
    }
</style>
