<!-- SOURCE: https://codepen.io/tomhazledine/pen/RwxeVw -->

<link
    href="https://fonts.googleapis.com/css?family=Lato:900,400|Shadows+Into+Light"
    rel="stylesheet"
    type="text/css"
/>

<div class="cassetteOuter">
    <span class="screw screw1"><span class="screwInner"></span></span>
    <span class="screw screw2"><span class="screwInner"></span></span>
    <span class="screw screw3"><span class="screwInner"></span></span>
    <span class="screw screw4"><span class="screwInner"></span></span>

    <div class="stickerOuter">
        <div class="sticker">
            <span class="a-side">A</span>
            <p class="cursive">musicat mixtape</p>

            <!-- svelte-ignore a11y-missing-attribute -->
            <img class="logo" src="icon.png" />
            <span class="stripe"></span>
            <span class="c90">D-C90</span>
            <div class="middle clearfix">
                <span class="circle one">
                    <span class="teethBox">
                        <span class="teeth"></span>
                        <span class="teeth"></span>
                        <span class="teeth"></span>
                    </span>
                </span>
                <div class="window">
                    <span class="reelOne"></span>
                    <span class="reelTwo"></span>
                </div>
                <span class="circle two">
                    <span class="teethBox">
                        <span class="teeth"></span>
                        <span class="teeth"></span>
                        <span class="teeth"></span>
                    </span>
                </span>
            </div>
        </div>
    </div>

    <div class="cassetteBottom">
        <span class="screw screw5"><span class="screwInner"></span></span>
        <span class="bottomShadow">
            <span class="bottomHoles one"></span>
            <span class="bottomHoles two"></span>
            <span class="bottomHoles three"></span>
        </span>
        <span class="holes one"></span>
        <span class="holes two"></span>
    </div>
</div>

<style lang="scss">
    * {
        box-sizing: border-box;
    }

    // Colours
    $black: #4d4d4d;
    $white: #ffffff;
    $cream: #efeeeb;
    $grey: #dad8d2;
    $primary: #38323c;

    $brown: #4d3b3b;
    $c90grey: #bbbbbb;
    $c90red: #dd342f; //#881615;
    $c90green: #4b9121;
    $c90black: #141311;
    $screwGrey: #5e5f5a;
    $red: #fd6139;
    $blue: #424ba8;
    $paper: #f8f1e1; //#d1cfb6;

    // Typography
    $bold: 900;
    $book: 400;
    $sans: "Lato", sans-serif;
    $cursive: "Shadows Into Light", cursive;

    // Animation
    $dur: 1s;

    // Clearfix
    .clearfix:before,
    .clearfix:after {
        content: " "; // 1
        display: table; // 2
    }
    .clearfix:after {
        clear: both;
    }
    // For IE 6/7 only – Include this rule to trigger hasLayout and contain floats.
    .clearfix {
        *zoom: 1;
    }

    // Variables
    $cassetteShadow1: -0.9em;
    $cassetteShadow2: 2em;
    $cassetteShadow3: 0;
    $cassetteShadow4: 0.5em;

    // Cassette

    .cassetteOuter {
        position: relative;
        margin: 0em auto;
        background: $c90grey;
        border-radius: 0.7em;
        scale: 0.7;
        width: 20em;
        height: 12em;
        box-shadow:
            darken($c90grey, 15%) 0 0.6em 0 0,
            darken($primary, 10%) $cassetteShadow1 $cassetteShadow2 0
                $cassetteShadow4;
        padding: 1em;
        border-top: lighten($c90grey, 10%) solid 0.1em;
        border-bottom: darken($c90grey, 10%) solid 0.1em;
        &:before {
            position: absolute;
            bottom: 1em;
            right: 100%;
            content: "";
            height: 3.5em;
            border-right: darken($c90grey, 15%) solid 0.2em;
            border-top: transparent solid 0.2em;
            border-bottom: transparent solid 0.2em;
            box-shadow: darken($primary, 10%) $cassetteShadow1 $cassetteShadow2
                0 $cassetteShadow4;
        }
        &:after {
            position: absolute;
            z-index: -9000;
            bottom: 1em;
            left: 100%;
            content: "";
            height: 3.5em;
            border-left: darken($c90grey, 5%) solid 0.2em;
            border-top: transparent solid 0.2em;
            border-bottom: transparent solid 0.2em;
            /* box-shadow:darken($primary,10%) $cassetteShadow1 $cassetteShadow2 0 $cassetteShadow4; */
        }
    }
    .screw {
        position: absolute;
        display: block;
        width: 0;
        height: 0;
        border-left: $screwGrey solid 0.4em;
        border-bottom: $screwGrey solid 0.3em;
        border-top: lighten($screwGrey, 40%) solid 0.4em;
        border-right: lighten($screwGrey, 40%) solid 0.4em;
        border-radius: 50%;
        box-shadow: darken($c90grey, 25%) 0 -0.2em 0 0;
        &:before {
            position: absolute;
            top: -0.25em;
            right: -0.35em;
            display: block;
            content: "";
            width: 0.3em;
            height: 0.2em;
            border-radius: 50%;
            background: $white;
            transform: rotate(45deg);
        }
        &.screw1 {
            top: 0.5em;
            left: 0.3em;
            .screwInner {
                transform: rotate(20deg);
            }
        }
        &.screw2 {
            top: 0.5em;
            right: 0.3em;
            .screwInner {
                transform: rotate(30deg);
            }
        }
        &.screw3 {
            bottom: 0.3em;
            left: 0.3em;
            .screwInner {
                transform: rotate(10deg);
            }
        }
        &.screw4 {
            bottom: 0.3em;
            right: 0.3em;
            .screwInner {
                transform: rotate(-10deg);
            }
        }
        .screwInner {
            position: absolute;
            top: -0.15em;
            left: -0.225em;
            display: block;
            width: 0.45em;
            height: 0.2em;
            border-radius: 0.1em;
            background: darken($screwGrey, 15%);
            &:after {
                position: absolute;
                top: 0;
                left: 0;
                display: block;
                content: "";
                width: 0.45em;
                height: 0.2em;
                border-radius: 0.1em;
                background: darken($screwGrey, 15%);
                transform: rotate(90deg);
            }
        }
    }
    .stickerOuter {
        width: 100%;
        height: 8em;
        background: $c90grey;
        border-radius: 0.7em;
        border-top: darken($c90grey, 10%) solid 0.1em;
        border-bottom: lighten($c90grey, 10%) solid 0.1em;
        position: relative;
    }
    .sticker {
        position: absolute;
        top: 0.2em;
        right: 0.2em;
        left: 0.2em;
        bottom: 0.2em;
        background: $cream;
        border-radius: 0.7em;
        border-bottom: darken($c90grey, 20%) solid 1px;
        .a-side {
            position: absolute;
            border-radius: 0.1em;
            top: 0.8em;
            left: 1em;
            font-weight: $bold;
            display: block;
            background: $black;
            color: $cream;
            width: 1.1em;
            height: 1.1em;
            padding-top: 0.1em;
            line-height: 1em;
            text-align: center;
        }
        .cursive {
            font-family: $cursive;
            display: block;
            padding: 0.5em 0.5em 0.5em 5em;
            transform: rotate(-3deg);
            color: $blue;
        }
        .logo {
            width: 50px;
            top: 0;
            right:15px;
            position: absolute;
            filter: sepia(0.8) saturate(0) opacity(0.7);
        }
        
        .stripe {
            display: block;
            position: absolute;
            top: 40%;
            width: 100%;
            border-top: $red solid 0.3em;
            border-bottom: $black solid 0.3em;
        }
        .c90 {
            position: absolute;
            bottom: -0.1em;
            left: 3.5em;
            color: $c90green;
            font-weight: $bold;
        }
    }
    .middle {
        position: relative;
        width: 60%;
        margin: 0.5em auto 0;
        background: $c90grey;
        padding: 0.2em;
        border-radius: 0.4em;
        border-top: lighten($c90grey, 10%) solid 0.1em;
        border-bottom: darken($c90grey, 15%) solid 0.1em;
        box-shadow:
            $c90grey 0 0 0 0.5em,
            darken($c90grey, 20%) 0 -1px 0 0.5em;
    }
    .circle {
        display: block;
        position: relative;
        float: left;
        width: 2.3em;
        height: 2.4em;
        border-radius: 50%;
        background: $primary;
        border-top: darken($c90grey, 10%) solid 0.1em;
        border-bottom: lighten($c90grey, 15%) solid 0.1em;
        box-shadow:
            inset $c90grey 0 0 0 0.2em,
            inset darken($c90grey, 15%) 0 0.1em 0 0.2em,
            inset $cream 0 0.1em 0 0.4em,
            inset darken($cream, 25%) 0 0.2em 0 0.4em,
            inset darken($primary, 10%) -0.7em 1.5em 0 0;
        &.two {
            float: right;
        }
    }
    .teethBox {
        position: absolute;
        top: 50%;
        margin-top: -0.75em;
        left: 50%;
        margin-left: -0.85em;
        width: 1.7em;
        height: 1.7em;
        transition: transform;
        transition-duration: $dur;
        //background:$red;
        animation: spin 2s linear infinite forwards alternate;
    }
    .teeth {
        display: block;
        position: absolute;
        top: 50%;
        margin-top: -0.1em;
        left: 50%;
        width: 1.7em;
        margin-left: -0.85em;
        height: 0.2em;
        box-shadow:
            inset $cream 0.25em 0 0 0,
            //inset darken($cream,25%) .3em 0 0 0,
            inset $cream -0.25em 0 0 0,
            //inset darken($cream,25%) -.3em 0 0 0,
;
        &:nth-of-type(2) {
            transform: rotate(60deg);
        }
        &:nth-of-type(3) {
            transform: rotate(-60deg);
        }
    }
    .window {
        position: absolute;
        z-index: 2;
        top: 0.2em;
        left: 50%;
        margin-left: -2.5em;
        margin-top: 0.3em;
        width: 5em;
        height: 1.8em;
        border-radius: 0.4em;
        overflow: hidden;
        background: darken($primary, 10%);
        border-top: darken($c90grey, 10%) solid 0.1em;
        border-bottom: lighten($c90grey, 15%) solid 0.1em;
        box-shadow:
            inset darken($c90grey, 15%) 0 0.2em 0 0,
            $c90grey 0 0 0 0.25em,
            darken($c90grey, 15%) 0 -0.1em 0 0.2em,
            lighten($c90grey, 15%) 0 0.1em 0 0.2em;
        .reelOne {
            position: absolute;
            left: -1.4em;
            top: 0.9em;
            display: block;
            margin-left: -4.5em;
            margin-top: -4.5em;
            width: 9em;
            height: 9em;
            background: $brown;
            border-radius: 50%;
            transition: width, height, margin;
            transition-duration: $dur;
            animation: spinReel1 2s linear infinite alternate;

            &:after {
                position: absolute;
                top: 50%;
                left: 50%;
                content: "";
                width: 4em;
                height: 4em;
                margin: -2em 0 0 -2em;
                border-radius: 50%;
                background: $cream;
                border: $cream solid 0.2em;
                box-shadow: inset $grey 0 0 0 0.2em;
            }
        }
        .reelTwo {
            position: absolute;
            right: -1.4em;
            top: 0.9em;
            display: block;
            margin-right: -3em;
            margin-top: -3em;
            width: 6em;
            height: 6em;
            background: $brown;
            border-radius: 50%;
            transition: width, height, margin;
            transition-duration: $dur;
            animation: spinReel2 2s linear infinite alternate;

            &:after {
                position: absolute;
                top: 50%;
                left: 50%;
                content: "";
                width: 4em;
                height: 4em;
                margin: -2em 0 0 -2em;
                border-radius: 50%;
                background: $cream;
                border: $cream solid 0.2em;
                box-shadow: inset $grey 0 0 0 0.2em;
            }
        }
    }
    .cassetteBottom {
        position: absolute;
        left: 50%;
        bottom: 0;
        height: 2.2em;
        width: 10em;
        margin-left: -5em;
        background: lighten($c90grey, 5%);
        border-top: lighten($c90grey, 15%) solid 0.1em;
        .screw5 {
            bottom: 1em;
            left: 50%;
            margin-left: -0.5em;
        }
        &:before {
            content: "";
            position: absolute;
            right: 100%;
            border-right: lighten($c90grey, 5%) solid 0.5em;
            border-top: transparent solid 2.2em;
        }
        &:after {
            content: "";
            position: absolute;
            left: 100%;
            border-left: lighten($c90grey, 5%) solid 0.5em;
            border-top: transparent solid 2.2em;
        }
    }
    .bottomShadow {
        position: absolute;
        z-index: 100;
        left: 50%;
        margin-left: -5.5em;
        bottom: -0.7em;
        width: 11em;
        height: 0.7em;
        background: darken($c90grey, 15%);
        &:before {
            content: "";
            display: block;
            position: absolute;
            z-index: 100;
            bottom: 100%;
            left: 50%;
            margin-left: -5.5em;
            width: 10.4em;
            border-bottom: darken($c90grey, 15%) solid 0.1em;
            border-right: transparent solid 0.3em;
            border-left: transparent solid 0.3em;
        }
        &:after {
            content: "";
            display: block;
            position: absolute;
            z-index: 100;
            top: 100%;
            left: 50%;
            margin-left: -5.5em;
            width: 10.4em;
            border-top: darken($c90grey, 15%) solid 0.1em;
            border-right: transparent solid 0.3em;
            border-left: transparent solid 0.3em;
            box-shadow: darken($primary, 10%) $cassetteShadow1
                ($cassetteShadow2 - 0.6em) 0 $cassetteShadow4;
        }
    }
    .holes {
        display: block;
        position: absolute;
        bottom: 0.8em;
        width: 0.8em;
        height: 0.6em;
        border-radius: 0.2em;
        background: darken($c90grey, 35%);
        box-shadow: inset darken($c90grey, 15%) 0 0.1em 0 0;
        &:after {
            position: absolute;
            top: 0.5em;
            content: "";
            width: 0.6em;
            height: 0.6em;
            border-radius: 50%;
            background: darken($c90grey, 35%);
            box-shadow: inset darken($c90grey, 15%) 0 0.1em 0 0;
        }
        &.one {
            left: 20%;
            &:after {
                right: 200%;
            }
        }
        &.two {
            right: 20%;
            &:after {
                left: 200%;
            }
        }
    }
    .bottomHoles {
        position: absolute;
        top: 0.05em;
        width: 2em;
        height: 0.6em;
        background: darken($c90grey, 35%);
        box-shadow:
            inset $brown 0 0.3em 0 0,
            inset darken($c90grey, 25%) 0 -0.1em 0 0;
        &.one {
            left: 0.5em;
        }
        &.two {
            left: 50%;
            margin-left: -1em;
            &:before {
                content: "";
                display: block;
                position: absolute;
                top: 0.1em;
                right: 140%;
                width: 0.4em;
                height: 0.4em;
                background: darken($c90grey, 35%);
                box-shadow:
                    inset $brown 0 0.2em 0 0,
                    inset darken($c90grey, 25%) 0 -0.05em 0 0;
            }
            &:after {
                content: "";
                display: block;
                position: absolute;
                top: 0.1em;
                left: 140%;
                width: 0.4em;
                height: 0.4em;
                background: darken($c90grey, 35%);
                box-shadow:
                    inset $brown 0 0.2em 0 0,
                    inset darken($c90grey, 25%) 0 -0.05em 0 0;
            }
        }
        &.three {
            right: 0.5em;
        }
    }

    // Hover Animation
    @keyframes spin {
        from {
            transform: rotate(0deg);
        }
        to {
            transform: rotate(-360deg);
        }
    }
    @keyframes spinReel1 {
        from {
            margin-left: -4.5em;
            margin-top: -4.5em;
            width: 9em;
            height: 9em;
        }
        to {
            margin-left: -4em;
            margin-top: -4em;
            width: 8em;
            height: 8em;
        }
    }
    @keyframes spinReel2 {
        to {
            margin-right: -3.5em;
            margin-top: -3.5em;
            width: 7em;
            height: 7em;
        }
        from {
            margin-right: -3em;
            margin-top: -3em;
            width: 6em;
            height: 6em;
        }
    }
    .cassetteOuter:hover {
        .teethBox {
            transform: rotate(369deg);
        }
        .reelOne {
            margin-left: -4em;
            margin-top: -4em;
            width: 8em;
            height: 8em;
        }
        .reelTwo {
            margin-right: -3.5em;
            margin-top: -3.5em;
            width: 7em;
            height: 7em;
        }
    }

    // Liner Notes
    .linerNotes {
        width: 21em;
        margin: 6em auto;
    }
    @mixin linerSpan {
        font-size: 0.6em;
        display: block;
    }
    .linerNotesTop {
        transform: rotate(180deg) skewX(10deg);
        box-shadow: darken($primary, 10%) 1.1em -1em 0 0;
        position: relative;
        left: 0.3em;
        height: 3em;
        background: $paper;
        padding: 0.4em;
        .boxDesc {
            @include linerSpan;
            font-size: 0.5em;
            float: left;
            //padding-top:.6em;
        }
        .infoBoxes {
            @include linerSpan;
            float: right;
            width: 40%;
            @mixin infoBox {
                display: block;
                text-transform: uppercase;
                text-align: center;
                border: $black solid 0.2em;
                width: 33.33%;
                height: 2.4rem;
                float: left;
                line-height: 1rem;
            }
            .box1 {
                @include infoBox;
                font-size: 0.6em;
                padding-top: 0.1em;
                //background:red;
            }
            .box2 {
                @include infoBox;
                font-size: 0.8em;
                background: $black;
                color: $paper;
                font-weight: $bold;
                padding-top: 0.7em;
            }
            .box3 {
                @include infoBox;
                font-size: 0.6em;
                padding-top: 0.6em;
            }
            .boxCopyright {
                display: block;
                width: 100%;
                height: 0.8em;
                font-size: 0.6em;
                text-align: center;
                position: relative;
                top: -0.4em;
            }
        }
    }
    .linerNotesSpine {
        transform: skewX(-10deg) rotate(180deg);
        position: relative;
        left: 0.3em;
        height: 3em;
        background: $red;
        border-top: $black solid 0.8em;
        border-bottom: $black solid 0.8em;
        text-align: center;
        font-weight: $bold;
        line-height: 1em;
        box-shadow:
            inset darken($paper, 25%) 0 0.3em 0 0,
            inset darken($paper, 25%) 0 -0.3em 0 0,
            inset $c90red 0 -0.7em 0 0,
            darken($primary, 10%) 0.6em -1em 0 0.2em;
        padding-top: 0.3em;
        .spineTitle {
            display: block;
            background: darken($paper, 25%);
            width: 8em;
            margin: 0 auto;
        }
    }
    .green {
        color: $c90green;
    }
    .linerNotesMainContent {
        position: relative;
        height: 12em;
        background: $paper;
        box-shadow: darken($primary, 10%) -1em 1em 0 0;
        z-index: 1;
        &:before {
            content: "";
            position: absolute;
            z-index: 2;
            top: 0;
            left: 50%;
            height: 12em;
            border-left: $black solid 0.1em;
        }
        &:after {
            content: "";
            position: absolute;
            top: 0;
            right: 0;
            width: 20.5em;
            border-top: darken($paper, 10%) solid 2em;
            border-right: transparent solid 0.5em;
            //display:none;
        }
        .side {
            float: left;
            position: relative;
            display: block;
            width: 50%;
            padding-left: 0.5em;
            border-bottom: $black solid 0.2em;
            z-index: 2;
        }
        .linerNotesMainContentInner {
            position: absolute;
            z-index: 100;
            top: 3em;
            height: 10em;
            left: 2em;
            width: 16em;
            font-family: $cursive;
            color: $blue;
            transform: rotate(3deg);
            h1 {
                font-size: 1.5em;
                line-height: 1em;
            }
        }
    }
    .linerNotesBottom {
        transform: rotate(180deg);
        position: relative;
        z-index: 2;
        height: 12em;
        background: $paper;
        box-shadow:
            inset $c90red 0 1.5em 0 0,
            inset $c90red 0 -1.5em 0 0,
            inset $red 0 3em 0 0,
            inset $red 0 -3em 0 0,
            inset $black 0 4.5em 0 0,
            inset $black 0 -4.5em 0 0,
            darken($primary, 10%) 1em -1em 0 0;
        padding-top: 1.5em;
        text-align: center;
        //p{line-height:1.5em;}
        .black {
            text-transform: uppercase;
            font-weight: $bold;
            font-size: 0.6em;
        }
        .white {
            color: $cream;
            font-size: 0.8em;
        }
        .title {
            font-weight: $bold;
            font-size: 2em;
            line-height: 2em;
            padding-bottom: 0.35em;
        }
    }
</style>
