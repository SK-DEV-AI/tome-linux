<script lang="ts">
    import type { PageProps } from './$types';

    import Button from '$components/Button.svelte';
    import Flex from '$components/Flex.svelte';
    import Input from '$components/Input.svelte';
    import Scrollable from '$components/Scrollable.svelte';
    import Card from '$components/Smithery/Card.svelte';
    import Configuration from '$components/Smithery/Configuration.svelte';
    import type { McpConfig } from '$lib/mcp';
    import McpServer from '$lib/models/mcp-server.svelte';
    import { Client } from '$lib/smithery/client';
    import type { CompactServer, ConfigSchema, Server } from '$lib/smithery/types';
    import { debounce } from '$lib/util.svelte';

    const { data }: PageProps = $props();

    let servers = $state(data.servers);
    let page: number = $state(1);
    let query: string = $state('');

    let serverToInstall: Server | null = $state(null);

    // New: expose load error if present
    let loadError: string | undefined = $state(data.loadError);

    function closeConfigure() {
        serverToInstall = null;
    }

    async function loadNextPage() {
        try {
            page += 1;
            const newServers = await new Client().servers(page);

            if (newServers.servers?.length > 0) {
                servers = servers.concat(newServers.servers);
            } else {
                // To prevent further requests if a page returns no servers.
                page -= 1;
                console.info('No more servers to load.');
            }
        } catch (error) {
            console.error('Failed to load more servers:', error);
            // Revert page increment on error
            page -= 1;
        }
    }

    async function search() {
        const client = new Client();

        if (query !== '') {
            const res = await client.search(query);
            servers = res.servers || [];
        } else {
            const res = await client.servers();
            servers = res.servers || [];
        }
    }

    function configSchemaFor(server: Server): ConfigSchema {
        return server.connections.findBy('type', 'stdio')?.configSchema || ({} as ConfigSchema);
    }

    async function configure(_server: CompactServer) {
        try {
            serverToInstall = await new Client().server(_server.qualifiedName);
            loadError = undefined;
        } catch (e) {
            console.error('failed to fetch server:', e);
            loadError = e instanceof Error ? e.message : String(e);
        }
    }

    async function install(config: McpConfig) {
        // HEURISTIC FIX: Some server stdioFunctions incorrectly split single
        // arguments (like a file path) into an array of single characters.
        // This detects that pattern (many single-character arguments) and
        // joins them back together.
        if (config.args.length > 1) {
            const looksSplit = config.args.every(arg => arg.length === 1);
            if (looksSplit) {
                console.warn(
                    `[MCP Installer] Detected potentially malformed arguments from server's configuration function. Re-joining into a single argument.`
                );
                config.args = [config.args.join('')];
            }
        }

        await McpServer.create(config);
        serverToInstall = null;
    }
</script>

{#if loadError}
    <Flex class="w-full items-center justify-center p-8">
        <div class="text-red max-w-xl text-center">
            <h2 class="text-light mb-2 text-xl font-semibold">Failed to load Smithery registry</h2>
            <p class="text-medium mb-4">{loadError}</p>
            <p class="text-dark text-sm">Try: check network, open DevTools, or run curl from the host.</p>
        </div>
    </Flex>
{:else}
    {#if serverToInstall}
        <Configuration
            server={serverToInstall}
            config={configSchemaFor(serverToInstall)}
            onCancel={closeConfigure}
            onInstall={install}
        />
    {:else}
        <Scrollable class="!h-content pr-2">
            <Flex class="w-full">
                <Input
                    bind:value={query}
                    class="placeholder:text-light mb-8"
                    label={false}
                    name="search"
                    onkeyup={debounce(search)}
                    placeholder="Search Smithery..."
                />
            </Flex>

            <Flex class="grid w-full auto-cols-max auto-rows-max grid-cols-3 items-start gap-4">
                {#each servers as server (server.qualifiedName)}
                    <Card {server} onInstall={configure} />
                {/each}
            </Flex>

            <Flex class="w-full justify-center">
                <Button onclick={loadNextPage} class="border-light text-medium m-auto mt-8">
                    Load More
                </Button>
            </Flex>
        </Scrollable>
    {/if}
{/if}
