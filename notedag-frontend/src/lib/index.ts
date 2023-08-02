// place files you want to import through the `$lib` alias in this folder.
import { dev } from '$app/environment';
import { env } from '$env/dynamic/public'

const ENDPOINT = dev ? (env.PUBLIC_API_URL ?? "http://127.0.0.1:8080") : window.location.origin;

// server proxies REST api calls
export const api = {
	get: (fn: string, params: Record<string, string>) => fetch(`${ENDPOINT}/${fn}?` + new URLSearchParams(params), {
		mode: 'cors',
	}),
	post: (fn: string, params: Record<string, string>) => fetch(`${ENDPOINT}/${fn}`, {
		method: "POST",
		body: JSON.stringify(params),
		headers: {
			"Content-Type": "application/json",
		},
		mode: 'cors',
	})
}

export enum EditorMode {
	NORMAL = "NORMAL",
	INSERT = "INSERT",
}
