/** @type {import('./$types').PageServerLoad} */

export async function load({ fetch, params }) {
	const filePath = params.filePath;
	
	const response = await fetch("/api/read?" + new URLSearchParams({
		filePath,
	}));
	const { contents } = await response.json();

	let tokens = filePath.split('/');
	const filename = tokens[tokens.length-1];
	return {
		root: filePath,
		filename,
		contents,
	}
}
