export type VaultMeta = {
    schema_version: number;
    created_at: number;
    last_modified: number;
};

export type EntryMetaPreview = {
    entry_id: string;
    url: string | null;
    created_at: number;
};

export type PasswordLeak = {
    entry_id: string;
    compromised: boolean;
};

export type DecryptedEntry = {
    entry_id: string;
    account: string;
    password: string;
    url: string | null;
    note: string | null;
    created_at: number;
    updated_at: number;
};

export type NewEntryParams = {
    account: string;
    password: string;
    url?: string;
    note?: string;
};

export type UpdateEntryParams = {
    entry_id: string;
    account?: string;
    password?: string;
    url?: string;
    note?: string;
};
