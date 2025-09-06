import type { PageLoad } from './$types';

import { Client } from '$lib/smithery/client';
import type { CompactServer } from '$lib/smithery/types';

interface Response {
    servers: CompactServer[];
    loadError?: string;
}

export const load: PageLoad = async (): Promise<Response> => {
    const client = new Client();

    try {
        const res = await client.servers();
        // client.servers() returns a ServerList { servers, pagination }
        return { servers: res.servers || [] };
    } catch (err) {
        // Surface the actual error message to the page so users can see cause
        let msg = 'Failed to load Smithery registry.';
        if (err instanceof Error) {
            msg = `${err.name}: ${err.message}`;
        } else if (typeof err === 'string') {
            msg = err;
        } else {
            try {
                msg = JSON.stringify(err);
            } catch {
                // leave default
            }
        }
        console.error('[Smithery] load error:', err);
        return {
            servers: [],
            loadError: msg,
        };
    }
};
