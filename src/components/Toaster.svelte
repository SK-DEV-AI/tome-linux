<script lang="ts">
    import { toasts } from '$lib/stores/toasts';
    import Toast from './Toast.svelte';
</script>

<div
    aria-live="assertive"
    class="pointer-events-none fixed inset-0 z-50 flex items-end px-4 py-6 sm:items-start sm:p-6"
>
    <div class="flex w-full flex-col items-center space-y-4 sm:items-end">
        {#each $toasts as toast (toast.id)}
            <Toast {toast} />
        {/each}
    </div>
</div>
