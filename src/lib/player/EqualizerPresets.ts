import type { EqualizerBand, EqualizerPreset } from "../../App";

const WINAMP_FREQS = [60, 170, 310, 600, 1000, 3000, 6000, 12000, 14000, 16000];

export const EQ_LABELS = [
    "60",
    "170",
    "310",
    "600",
    "1K",
    "3K",
    "6K",
    "12K",
    "14K",
    "16K",
];

/**
 * Converts Winamp's 0-64 scale to decibels (-12 to +12)
 */
const winampToDb = (val: number): number => {
    const db = (val - 33) * 0.375;
    return Math.round(db * 10) / 10; // Round to 1 decimal place
};

const createBandsFromRaw = (rawGains: number[]): EqualizerBand[] => {
    return rawGains.map((raw, index) => ({
        id: index,
        freq: WINAMP_FREQS[index],
        gain: winampToDb(raw),
        q: 1.41,
        type: "peaking",
        label: `${WINAMP_FREQS[index] < 1000 ? WINAMP_FREQS[index] : WINAMP_FREQS[index] / 1000 + "k"}`,
    }));
};

export const DEFAULT_EQ = createBandsFromRaw([
    33, 33, 33, 33, 33, 33, 33, 33, 33, 33,
]);
export const EQ_PRESETS: EqualizerPreset[] = [
    {
        name: "Rock",
        bands: createBandsFromRaw([45, 40, 26, 23, 26, 39, 47, 50, 50, 50]),
    },
    {
        name: "Techno",
        bands: createBandsFromRaw([45, 42, 33, 23, 24, 33, 45, 48, 48, 47]),
    },
    {
        name: "Classical",
        bands: createBandsFromRaw([33, 33, 33, 33, 33, 33, 20, 20, 20, 16]),
    },
    {
        name: "Full Bass",
        bands: createBandsFromRaw([48, 48, 48, 42, 35, 25, 18, 15, 14, 14]),
    },
    {
        name: "Dance",
        bands: createBandsFromRaw([48, 44, 36, 32, 32, 22, 20, 20, 32, 32]),
    },
    {
        name: "Laptop/Headphones",
        bands: createBandsFromRaw([40, 50, 41, 26, 28, 35, 40, 48, 53, 56]),
    },
    {
        name: "Large Hall",
        bands: createBandsFromRaw([49, 49, 42, 42, 33, 24, 24, 24, 33, 33]),
    },
];

export const EQ_PRESET_OPTIONS = EQ_PRESETS.map((p) => ({
    label: p.name,
    value: p.name,
}));
