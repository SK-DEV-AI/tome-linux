import type { PageLoad } from './$types';
import { Client } from '$lib/smithery/client';

export const load: PageLoad = async () => {
    const client = new Client();
    let ok = false;
    let details = '';
    try {
        ok = await client.test();
    } catch (e) {
        ok = false;
        details = e instanceof Error ? `${e.name}: ${e.message}` : String(e);
    }
    return { ok, details };
};
