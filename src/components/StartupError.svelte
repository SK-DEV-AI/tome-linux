<script lang="ts">
    import Flex from './Flex.svelte';
    import Svg from './Svg.svelte';

    export let message: string;
</script>

<Flex class="h-screen w-screen flex-col items-center justify-center bg-medium p-8 text-center">
    <div class="text-red mb-4 h-16 w-16">
        <Svg name="Error" />
    </div>
    <h1 class="text-light mb-2 text-2xl font-semibold">Application Error</h1>
    <p class="text-medium max-w-md">{message}</p>
    <p class="text-dark mt-8 text-sm">
        Once you have resolved the issue, you may need to restart Tome.
    </p>
</Flex>
