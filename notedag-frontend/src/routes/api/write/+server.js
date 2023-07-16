import { json } from '@sveltejs/kit';
import { api } from '$lib';

/** @type {import('./$types').RequestHandler} */
export async function POST({ request, cookies }) {
	const { filePath, contents } = await request.json();

	const _ = await api.post("notedag/write", { filePath, contents });

    return json({ status: 200 });
}
