import { invoke } from "@tauri-apps/api/core";

const FN_NAMES = ["", "error", "warn", "info", "debug", "log"];
const IS_DEV = process.env.NODE_ENV === "development";

enum Level {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

let maxLevel: Level;

function forwardConsole(level: Level) {
    const fnName = FN_NAMES[level];
    const original = console[fnName];

    if (IS_DEV) {
        if (level <= maxLevel) {
            console[fnName] = (...args) => {
                original(...args);
                log(level, args);
            };
        } else {
            console[fnName] = (...args) => {
                original(...args);
            };
        }
    } else {
        if (level <= maxLevel) {
            console[fnName] = (...args) => {
                original(...args);
                log(level, args);
            };
        } else {
            console[fnName] = nop;
        }
    }
}

async function log(level: Level, args: any[]): Promise<void> {
    const traces = new Error().stack
        ?.split("\n")
        .map((line) => line.split("@"))
        .filter(
            ([name, location]) =>
                location !== "[native code]" && !location.includes("Logger.ts"),
        );

    let file = null;
    let line = null;
    let caller = "";

    if (traces?.[0]) {
        const [_caller, location] =
            traces[0].length === 2 ? traces[0] : ["", traces[0][0]];

        if (_caller) {
            caller = _caller;
        }

        if (location) {
            const [_file, _line] = location
                .replace(/http:\/\/localhost:\d+\//, "")
                .split(":");

            file = _file;
            line = Number(_line);
        }
    }

    await invoke("write_log", {
        level,
        message: args.toString(),
        file,
        line,
        caller,
    });
}

function nop() {}

export async function setup() {
    maxLevel = await invoke("max_log_level", {});

    for (let level = 1; level <= 5; level += 1) {
        forwardConsole(level);
    }
}
