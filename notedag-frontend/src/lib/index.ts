// place files you want to import through the `$lib` alias in this folder.

const ENDPOINT = "http://127.0.0.1:8080/";

// server proxies REST api calls
export const api = {
	get: (fn: string, params: Record<string, string>) => fetch(`${ENDPOINT}${fn}?` + new URLSearchParams(params)),
	post: (fn: string, params: Record<string, string>) => fetch(`${ENDPOINT}${fn}`, {
		method: "POST",
		body: JSON.stringify(params),
		headers: {
			"Content-Type": "application/json",
		},
	})
}

export enum EditorMode {
	NORMAL = "NORMAL",
	INSERT = "INSERT",
}
