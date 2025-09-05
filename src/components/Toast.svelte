<script lang="ts">
    import { fly } from 'svelte/transition';
    import type { Toast } from '$lib/stores/toasts';
    import Flex from './Flex.svelte';
    import Svg from './Svg.svelte';

    export let toast: Toast;

    const typeClasses = {
        info: 'bg-blue text-white',
        success: 'bg-green text-white',
        error: 'bg-red text-white',
    };

    const icons = {
        info: 'Info', // Assuming an 'Info' icon exists
        success: 'Check',
        error: 'Warning',
    }
</script>

<div
    in:fly={{ y: 20, duration: 300 }}
    out:fly={{ y: 20, duration: 300 }}
    class="pointer-events-auto m-2 w-full max-w-sm overflow-hidden rounded-lg shadow-lg ring-1 ring-black ring-opacity-5 {typeClasses[toast.type]}"
>
    <Flex class="p-4">
        <div class="flex-shrink-0">
            <Svg name={icons[toast.type]} class="h-6 w-6 text-white" />
        </div>
        <div class="ml-3 w-0 flex-1 pt-0.5">
            <p class="text-sm font-medium">{toast.message}</p>
        </div>
    </Flex>
</div>
