const API_BASE = (import.meta.env.VITE_API_URL ?? 'http://localhost:8000') + '/api';

export interface AuthResponse {
    token: string;
    user_id: string;
    username: string;
    two_factor_enabled: boolean;
    force_password_change: boolean;
}

export interface UserProfile {
    id: string;
    username: string;
    email: string | null;
    two_factor_enabled: boolean;
    force_password_change: boolean;
}

async function fetchPublicKeyPem(): Promise<string> {
    const res = await fetch(`${API_BASE}/public-key`);
    if (!res.ok) throw new Error(`public-key fetch failed: ${res.status}`);
    const data = await res.json();
    return data.public_key as string;
}

function pemToArrayBuffer(pem: string): ArrayBuffer {
    const b64 = pem
        .replace('-----BEGIN RSA PUBLIC KEY-----', '')
        .replace('-----END RSA PUBLIC KEY-----', '')
        .replace(/\s/g, '');
    const binary = atob(b64);
    const buf = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) buf[i] = binary.charCodeAt(i);
    return buf.buffer;
}

async function importRsaKey(pem: string): Promise<CryptoKey> {
    const keyData = pemToArrayBuffer(pem);
    return crypto.subtle.importKey(
        'spki',
        keyData,
        { name: 'RSA-OAEP', hash: 'SHA-256' },
        false,
        ['encrypt'],
    );
}

async function encryptPassword(password: string, publicKey: CryptoKey): Promise<string> {
    const encoded = new TextEncoder().encode(password);
    const encrypted = await crypto.subtle.encrypt({ name: 'RSA-OAEP' }, publicKey, encoded);
    const bytes = new Uint8Array(encrypted);
    let binary = '';
    for (let i = 0; i < bytes.length; i++) binary += String.fromCharCode(bytes[i]);
    return btoa(binary);
}

export async function login(
    username: string,
    password: string,
    twoFactorCode?: string,
): Promise<AuthResponse> {
    const pem = await fetchPublicKeyPem();
    const publicKey = await importRsaKey(pem);
    const encryptedPassword = await encryptPassword(password, publicKey);

    const res = await fetch(`${API_BASE}/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            username,
            encrypted_password: encryptedPassword,
            two_factor_code: twoFactorCode ?? null,
        }),
    });

    if (res.status === 403) {
        throw new TwoFactorRequiredError(username, password);
    }

    if (!res.ok) {
        throw new Error(`login failed: ${res.status}`);
    }

    return res.json();
}

export async function changePassword(
    token: string,
    oldPassword: string,
    newPassword: string,
): Promise<void> {
    const pem = await fetchPublicKeyPem();
    const publicKey = await importRsaKey(pem);
    const encryptedOld = await encryptPassword(oldPassword, publicKey);
    const encryptedNew = await encryptPassword(newPassword, publicKey);

    const res = await fetch(`${API_BASE}/change-password`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            Authorization: `Bearer ${token}`,
        },
        body: JSON.stringify({
            old_password: encryptedOld,
            new_password: encryptedNew,
        }),
    });

    if (!res.ok) throw new Error(`change-password failed: ${res.status}`);
}

export async function validateSession(token: string): Promise<UserProfile> {
    const res = await fetch(`${API_BASE}/validate-session`, {
        headers: { Authorization: `Bearer ${token}` },
    });
    if (!res.ok) throw new Error('session invalid');
    return res.json();
}

export class TwoFactorRequiredError extends Error {
    constructor(
        public readonly username: string,
        public readonly password: string,
    ) {
        super('2fa_required');
    }
}
