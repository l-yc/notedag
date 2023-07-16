/** @type {import('./$types').PageServerLoad} */

export async function load({ fetch, params }) {
	const filePath = params.filePath;
	
	const response = await fetch("/api/list?" + new URLSearchParams({
		filePath,
	}));
	const { files } = await response.json();

	return {
		root: filePath,
		files,
	}
}
