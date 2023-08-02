import { api } from '$lib';

/** @type {import('./$types').PageLoad} */
export async function load({ fetch, params }) {
	const filePath = params.filePath;
	
	const response = await api.get('notedag/read', { filePath });
	const contents = await response.json();

	let tokens = filePath.split('/');
	const filename = tokens[tokens.length-1];
	return {
		root: filePath,
		filename,
		contents,
	}
}
