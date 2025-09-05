import { writable } from 'svelte/store';

export interface Toast {
    id: number;
    message: string;
    type: 'info' | 'success' | 'error';
    duration: number;
}

const createToastStore = () => {
    const { subscribe, update } = writable<Toast[]>([]);

    const addToast = (
        message: string,
        type: Toast['type'] = 'info',
        duration: number = 3000
    ) => {
        const id = Date.now();
        update(toasts => [...toasts, { id, message, type, duration }]);
        setTimeout(() => removeToast(id), duration);
    };

    const removeToast = (id: number) => {
        update(toasts => toasts.filter(t => t.id !== id));
    };

    return {
        subscribe,
        info: (message: string, duration?: number) => addToast(message, 'info', duration),
        success: (message: string, duration?: number) => addToast(message, 'success', duration),
        error: (message: string, duration?: number) => addToast(message, 'error', duration),
    };
};

export const toasts = createToastStore();
