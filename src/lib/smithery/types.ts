/* eslint-disable @typescript-eslint/no-explicit-any */

export interface ServerList {
    servers: CompactServer[];
    pagination: {
        currentPage: number;
        pageSize: number;
        totalPages: number;
        totalCount: number;
    };
}

export interface CompactServer {
    qualifiedName: string;
    displayName: string;
    description: string;
    homepage: string;
    useCount: number; // Corrected from string
    isDeployed: boolean;
    createdAt: string;
}

export interface Server {
    qualifiedName: string;
    displayName: string;
    description: string; // Added
    remote: boolean; // Added
    iconUrl: string | null;
    deploymentUrl: string | null; // Corrected from string
    connections: Connection[];
    security?: {
        scanPassed: boolean;
    };
    tools?: Array<{
        name: string;
        description: string | null;
        inputSchema: {
            type: 'object';
            properties?: object;
        };
    }>;
}

export interface Connection {
    type: 'stdio' | 'ws' | 'http';
    configSchema: ConfigSchema;
    stdioFunction: string;
}

export interface ConfigSchema {
    type: 'object';
    required: string[];
    properties: {
        [key: string]: {
            type: 'string';
            default?: any;
            description: string;
        };
    };
}

export interface Config {
    name: string;
    required: boolean;
    description: string;
    value: string;
    valid: boolean;
}
