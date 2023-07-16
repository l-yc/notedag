import { json } from '@sveltejs/kit';
import { api } from '$lib';

/** @type {import('./$types').RequestHandler} */
export async function GET({ request }) {
	const url = new URL(request.url);
	const filePath = url.searchParams.get('filePath') ?? '';

	const response = await api.get('notedag/read', { filePath });
	const contents = await response.json();

    return json({ contents, status: 200 });
}
