// Safer frontend updater helper that works even if the updater plugin
// is not registered. It will return null when the updater is unavailable.

export async function availableUpdate(): Promise<any | null> {
    try {
        const plugin = await import('@tauri-apps/plugin-updater').catch(() => null);
        if (!plugin || typeof plugin.check !== 'function') {
            return null;
        }
        try {
            return await plugin.check();
        } catch {
            return null;
        }
    } catch {
        return null;
    }
}

export async function isUpToDate(): Promise<boolean> {
    try {
        const update = await availableUpdate();
        if (!update) {
            return true;
        }

        // Config may be DB-backed; keep original behavior to ignore skipped versions.
        const { Config } = await import('$lib/models').catch(() => ({ Config: null }));
        if (Config && Config.skippedVersions && Config.skippedVersions.includes(update.version)) {
            return true;
        }

        return false;
    } catch {
        // If something goes wrong, treat as up-to-date so app usage is not blocked.
        return true;
    }
}
