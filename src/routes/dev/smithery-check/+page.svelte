<script lang="ts">
    const { data } = $props();
    let ok: boolean = data.ok;
    let details: string = data.details;
</script>

<div class="p-8">
    <h1 class="text-xl mb-4">Smithery Connectivity Check</h1>
    {#if ok}
        <div class="text-green">OK — client.test() succeeded.</div>
    {:else}
        <div class="text-red mb-4">FAIL — client.test() failed</div>
        {#if details}
            <pre class="bg-surface p-4 rounded">{details}</pre>
        {/if}
        <p class="mt-4 text-sm">If TLS or network errors appear, run from the host:</p>
        <pre class="bg-surface p-4 rounded">curl -v "https://registry.smithery.ai/servers?pageSize=1"</pre>
    {/if}
</div>
