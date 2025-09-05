import type { ToSqlRow } from './base.svelte';

import { startupError } from '$lib/stores/error';
import Gemini from '$lib/engines/gemini/client';
import Ollama from '$lib/engines/ollama/client';
import OpenAI from '$lib/engines/openai/client';
import type { Client, ClientOptions } from '$lib/engines/types';
import { Model } from '$lib/models';
import Base from '$lib/models/base.svelte';

const AVAILABLE_MODELS: Record<EngineType, 'all' | string[]> = {
    'openai-compat': 'all',
    ollama: 'all',
    openai: ['gpt-4o', 'o4-mini', 'gpt-4.5-preview', 'gpt-4.1', 'gpt-4.1-mini'],
    gemini: ['gemini-2.5-pro', 'gemini-2.5-flash', 'gemini-2.0-flash-lite', 'gemini-1.5-pro'],
};

type EngineType = 'ollama' | 'openai' | 'gemini' | 'openai-compat';

interface Row {
    id: number;
    name: string;
    type: string;
    options: string;
}

export default class Engine extends Base<Row>('engines') {
    id?: number = $state();
    name: string = $state('');
    type: EngineType = $state('openai-compat');
    options: ClientOptions = $state({ url: '', apiKey: '' });
    models: Model[] = $state([]);

    static async sync() {
        await super.sync();
        await Model.sync();
    }

    get client(): Client | undefined {
        const Client = {
            ollama: Ollama,
            openai: OpenAI,
            gemini: Gemini,
            'openai-compat': OpenAI,
        }[this.type];

        if (Client) {
            try {
                return new Client({ ...this.options, engineId: Number(this.id) });
            } catch {
                return undefined;
            }
        }
    }

    protected async afterSave(): Promise<void> {
        await Model.sync();
    }

    protected static async fromSql(row: Row): Promise<Engine> {
        const engine = Engine.new({
            id: row.id,
            name: row.name,
            type: row.type as EngineType,
            options: JSON.parse(row.options),
            models: [],
        });

        if (engine.client) {
            try {
                engine.models = (await engine.client.models())
                    .filter(
                        m =>
                            AVAILABLE_MODELS[engine.type] == 'all' ||
                            AVAILABLE_MODELS[engine.type].includes(m.name)
                    )
                    .sortBy('name');
            } catch (e) {
                if (engine.type === 'ollama') {
                    startupError.set(
                        'Ollama server not found. Please ensure it is running and accessible.'
                    );
                }
                // Log other errors for debugging, but don't block the UI
                console.error(`Failed to fetch models for engine ${engine.name}:`, e);
            }
        }

        return engine;
    }

    protected async toSql(): Promise<ToSqlRow<Row>> {
        return {
            name: this.name,
            type: this.type,
            options: JSON.stringify(this.options),
        };
    }
}
