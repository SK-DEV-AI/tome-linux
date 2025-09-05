import { HttpClient, type HttpOptions } from '$lib/http';
import type { CompactServer, Server, ServerList } from '$lib/smithery/types';

interface ClientOptions extends HttpOptions {
    apiKey?: string;
}

export class Client extends HttpClient {
    constructor(options: ClientOptions = {}) {
        const apiKey = options.apiKey;
        delete options.apiKey;

        if (apiKey && apiKey.trim() !== '') {
            options.headers = {
                ...options.headers,
                Authorization: `Bearer ${apiKey}`,
            };
        }

        super(options);
    }

    get url() {
        return 'https://registry.smithery.ai';
    }

    async test(): Promise<boolean> {
        try {
            const result = await this.get('/servers?pageSize=1');
            return !!result; // Will be falsy if get returns undefined, truthy otherwise
        } catch (e) {
            console.error("Smithery connection test failed:", e);
            return false;
        }
    }

    async servers(page: number = 1): Promise<ServerList> {
        const query = this.options.headers?.Authorization ? '' : 'is:local';
        const response = (await this.get(
            `/servers?q=${query}&pageSize=24&page=${page}`
        )) as ServerList;
        return response || { servers: [], pagination: { currentPage: 1, pageSize: 24, totalPages: 0, totalCount: 0 } };
    }

    async server(name: string): Promise<Server> {
        return (await this.get(`/servers/${name}`)) as Server;
    }

    async search(query: string): Promise<ServerList> {
        const baseQuery = this.options.headers?.Authorization ? '' : 'is:local';
        const q = encodeURIComponent(query).replace(/%20/g, '+');
        const finalQuery = baseQuery ? `${baseQuery}+${q}` : q;
        const response = (await this.get(`/servers?q=${finalQuery}`)) as ServerList;
        return response || { servers: [], pagination: { currentPage: 1, pageSize: 24, totalPages: 0, totalCount: 0 } };
    }
}
