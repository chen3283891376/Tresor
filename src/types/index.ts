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

export type TwoFAEntryPreview = {
    entry_id: string;
    issuer: string;
    account: string;
    created_at: number;
};

export type DecryptedTwoFAEntry = {
    entry_id: string;
    issuer: string;
    account: string;
    secret_base32: string;
    created_at: number;
    updated_at: number;
};

export type NewTwoFAParams = {
    issuer: string;
    account: string;
    secret: string;
};

export type UpdateTwoFAParams = {
    entry_id: string;
    issuer?: string;
    account?: string;
    secret?: string;
};

export type QrScanResult = {
    secret: string;
    issuer: string;
    account: string;
    algorithm?: string;
    digits?: number;
    period?: number;
};

export type PasswordGeneratorConfig = {
    length: number;
    include_uppercase: boolean;
    include_lowercase: boolean;
    include_digits: boolean;
    include_symbols: boolean;
    exclude_ambiguous: boolean;
    custom_symbols?: string;
};
