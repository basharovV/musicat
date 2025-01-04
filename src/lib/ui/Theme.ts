export let themeSettings = {
    font: {
        family: "Merriweather, serif"
    },
    main: {
        background: "white"
    },
    colors: {
        primary: { background: "#1b4738", color: "white" }
    }
};

export function encodeB64(str) {
    const buff = Buffer.from(str, "utf-8");
    return buff.toString("base64");
}

export function makeCSSVars(settings, prefix = "-") {
    return Object.entries(settings)
        .flatMap(([key, value]) => {
            const path = prefix + "-" + key;
            if (typeof value === "object") return makeCSSVars(value, path);
            else return `${path}:${value};`;
        })
        .join("\n");
}
