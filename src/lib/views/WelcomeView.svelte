<script>
    import { fade } from "svelte/transition";
    import { isWelcomeSeen } from "../../data/store";
</script>

<container in:fade={{ delay: 200, duration: 400 }}>
    <img class="bg" src="/images/welcome-cartwheel-galaxy.jpg" />
    <div class="content">
        <h1 in:fade={{ delay: 100, duration: 400 }}>
            A few words about this app
        </h1>
        <div in:fade={{ delay: 1000, duration: 400 }}>~~~</div>
        <br />
        <p style="margin: 0 auto" in:fade={{ delay: 1000, duration: 400 }}>
            <span>Musicat</span> is all about helping listeners connect more with
            their music collection.
        </p>
        <p style="font-size: 1.2em" in:fade={{ delay: 4800, duration: 400 }}>
            It's a hybrid app born out of necessity and curiosity. In the age of
            streaming, how can I learn to love my music collection again? <br/><br/>What
            if I see my whole music library on a world map? <br/><br/>What if my music
            player is also where I organize my own music ideas? 
        </p>
        <div in:fade={{ delay: 9800, duration: 400 }}>~~~</div>
        <p
            style="font-size: 1.2em;max-width: 550px;opacity: 0.8;"
            in:fade={{ delay: 9800, duration: 400 }}
        >
            This is a first version of that idea. <br /> It's constantly
            evolving, and I invite you to share feedback and suggestions to help
            improve it. Thanks for being curious and trying it out.
            <br />
            - Slav
        </p>

        <div in:fade={{ delay: 10800, duration: 400 }} class="button-container">
            <img class="logo" src="icon.png" />
            <button
                on:click={() => {
                    $isWelcomeSeen = true;
                }}>Very well then!</button
            >
        </div>
    </div>
</container>

<style lang="scss">
    container {
        width: 100%;
        height: 100%;
        position: absolute;
    }

    @keyframes move {
        0% {
            transform: scale(1);
            transform-origin: bottom left;
        }
        50% {
            transform: scale(1.1);
        }
        100% {
            transform: scale(1);
        }
    }
    @keyframes rotate {
        0% {
            transform: rotate(0deg);
        }
        25% {
            transform: rotate(5deg);
        }
        50% {
            transform: rotate(0deg);
        }
        75% {
            transform: rotate(-5deg);
        }
        100% {
            transform: rotate(0deg);
        }
    }
    @keyframes title {
        0% {
            color: white;
        }
        25% {
            color: rgb(207, 123, 123);
        }
        50% {
            color: rgb(207, 214, 204);
        }
        75% {
            color: rgb(230, 167, 167);
        }
        100% {
            color: white;
        }
    }
    .bg {
        position: absolute;
        width: 100%;
        min-width: 100%;
        height: auto;
        min-height: 100%;
        top: 0;
        bottom: 0;
        left: 0;
        right: 0;
        opacity: 0.2;
        object-fit: cover;
        animation: move 15s alternate-reverse infinite ease-in-out;
    }

    .content {
        z-index: 10;
        position: relative;
        margin: auto;
        width: 100%;
        height: 100%;
        padding: 2em;
        overflow: auto;
    }

    span {
        font-family: "2Peas";
        font-size: 1.6em;
        vertical-align: middle;
        margin-right: 10px;
    }

    h1 {
        margin: 0;
        margin-top: 0.2em;
        font-family: "Snake";
        font-size: 5em;
        animation: title 6s alternate-reverse infinite ease-in-out;
    }

    p {
        font-size: 1.7em;
        line-height: 1.4em;
        max-width: 620px;
        margin: 1em auto;
    }

    small {
        position: absolute;
        bottom: 0;
        right: 0;
        padding: 1em;
        opacity: 0.5;
    }

    .button-container {
        position: relative;
        margin-top: 8em;
        button {
            /* border-color: #5123DD; */
            font-size: 1.2em;
            z-index: 2;
            position: relative;
        }
        .logo {
            position: absolute;
            width: 150px;
            z-index: 0;
            top: -106px;
            animation: rotate 2s forwards infinite
                cubic-bezier(0.39, 0.575, 0.565, 1);
        }
    }
</style>
