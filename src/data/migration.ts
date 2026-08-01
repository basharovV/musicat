import { invoke } from "@tauri-apps/api/core";
import { exportDB } from "dexie-export-import";
import { writable } from "svelte/store";
import type { UserSettings } from "../App";
import { db } from "./db";

/**
 * One-time Dexie -> SQLite migration, driven from the frontend at startup.
 * The Rust side (db::migration) only shadow-migrates and verifies when
 * asked -- nothing calls it until now. Every read/write path in the app was
 * already cut over to SQLite during frontend integration, so this isn't
 * "migrate when convenient": if it doesn't run (or fails) for someone
 * upgrading from a Dexie-based version, their library is just empty, with
 * no Dexie fallback left to quietly serve it instead.
 */
export type MigrationState =
    | { status: "idle" }
    | { status: "checking" }
    | { status: "migrating" }
    | { status: "failed"; mismatches: string[] }
    | { status: "done" };

export const migrationState = writable<MigrationState>({ status: "idle" });

interface MigrationReport {
    tracks: number;
    albums: number;
    playlists: number;
    smartQueries: number;
    songProjects: number;
    artistProjects: number;
    scrapbookItems: number;
    warnings: string[];
}

interface VerificationReport {
    ok: boolean;
    mismatches: string[];
}

interface MigrationOutcome {
    report: MigrationReport;
    verification: VerificationReport;
}

async function markMigrated(settings: UserSettings) {
    await invoke("update_settings", {
        settings: { ...settings, migratedFromDexieAt: Date.now() },
    });
}

/**
 * Call once at startup, after settings have loaded. `settings` is whatever
 * `get_settings` just returned -- if `migratedFromDexieAt` is already set,
 * this is a no-op.
 */
export async function migrateFromDexieIfNeeded(settings: UserSettings) {
    if (settings.migratedFromDexieAt) {
        migrationState.set({ status: "done" });
        return;
    }

    migrationState.set({ status: "checking" });

    let songCount: number;
    try {
        songCount = await db.songs.count();
    } catch (e) {
        // No Dexie database on disk at all (a genuinely fresh install) --
        // nothing to migrate.
        console.error("Error checking Dexie for a migration source", e);
        songCount = 0;
    }

    if (songCount === 0) {
        await markMigrated(settings);
        migrationState.set({ status: "done" });
        return;
    }

    migrationState.set({ status: "migrating" });
    try {
        const blob = await exportDB(db);
        const exportJson = await blob.text();

        const outcome = await invoke<MigrationOutcome>(
            "run_shadow_migration_from_dexie",
            { exportJson },
        );

        if (!outcome.verification.ok) {
            console.error(
                "Dexie migration verification failed",
                outcome.verification.mismatches,
            );
            migrationState.set({
                status: "failed",
                mismatches: outcome.verification.mismatches,
            });
            return;
        }

        await invoke("cutover_migrated_database");
        await markMigrated(settings);
        migrationState.set({ status: "done" });
    } catch (e) {
        console.error("Error migrating library from Dexie", e);
        migrationState.set({ status: "failed", mismatches: [String(e)] });
    }
}
