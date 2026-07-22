import { goto } from '$app/navigation';
import { auth } from '$lib/auth/store.svelte';

export async function authedFetch(input: string, init?: RequestInit): Promise<Response> {
    const res = await fetch(input, init);
    if (res.status === 401) {
        auth.clearSession();
        goto('/login');
    }
    return res;
}
