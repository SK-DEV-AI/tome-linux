<script lang="ts">
    import Button from '$components/Button.svelte';
    import Flex from '$components/Flex.svelte';
    import Input from '$components/Input.svelte';
    import Svg from '$components/Svg.svelte';
    import { Client } from '$lib/smithery/client';
    import Setting from '$lib/models/setting.svelte';
    import { onMount } from 'svelte';
    import { toasts } from '$lib/stores/toasts';

    let apiKey = $state('');
    let status: 'unknown' | 'testing' | 'valid' | 'invalid' = $state('unknown');
    let { saving }: { saving: boolean } = $props();

    onMount(() => {
        apiKey = Setting.SmitheryApiKey || '';
    });

    async function testConnection() {
        status = 'testing';
        const client = new Client({ apiKey });
        if (await client.test()) {
            status = 'valid';
            toasts.success('Connection successful!');
        } else {
            status = 'invalid';
            toasts.error('Connection failed. Please check your API key.');
        }
    }

    function onApiKeyChange() {
        saving = true;
        Setting.SmitheryApiKey = apiKey;
        status = 'unknown'; // Reset status when key changes
        // A short timeout to show the "saving" checkmark
        setTimeout(() => (saving = false), 1000);
    }
</script>

<Flex class="w-full flex-col items-start">
    <Input
        bind:value={apiKey}
        label={false}
        name="smithery_api_key"
        type="password"
        class="w-full"
        onchange={onApiKeyChange}
        placeholder="smithery_xxxxxxxxxxxx"
    />
    <Flex class="mt-4 w-full items-center justify-start gap-4">
        <Button onclick={testConnection} class="border-purple text-purple">
            Test Connection
        </Button>
        {#if status === 'testing'}
            <p class="text-medium">Testing...</p>
        {:else if status === 'valid'}
            <Flex class="text-green items-center gap-2">
                <Svg name="Check" class="h-4 w-4" />
                <p>Valid Key</p>
            </Flex>
        {:else if status === 'invalid'}
            <Flex class="text-red items-center gap-2">
                <Svg name="Warning" class="h-4 w-4" />
                <p>Invalid Key</p>
            </Flex>
        {/if}
    </Flex>
</Flex>
