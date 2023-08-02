import { api } from '$lib';

/** @type {import('./$types').PageLoad} */
export async function load({ fetch, params }) {
	const filePath = params.filePath;

	const response = await api.get('notedag/list', { filePath })
	const files = await response.json();

	return {
		root: filePath,
		files,
	}
}
