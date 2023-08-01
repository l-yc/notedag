import Convert from 'ansi-to-html';
import type {CellState} from "$lib/notedag";

// for now, we'll connect directly
const KERNEL_URI = 'ws://127.0.0.1:8080/kernel/socket';

export class KernelManager {
	connection = {
		ws: null as WebSocket | null,
		status: 'disconnected',
	};

	callbacks: Record<string, (json: any) => void> = {};
	refresh?: () => void;

	constructor(refresh?: () => void) {
		this.refresh = refresh;
	}

	async connect() {
		await new Promise<void>((resolve, reject) => {
			if (this.connection.ws !== null) {
				resolve();
				return;
			} 

			let ws = new WebSocket(KERNEL_URI);
			this.connection = {
				ws,
				status: 'connecting',
			}

			ws.onopen = () => {
				this.connection.status = 'connected';
				console.log('connected');
				if (this.refresh) this.refresh();
				resolve();
			};
			ws.onclose = () => {
				this.connection = {
					ws: null,
					status: 'disconnected'
				};
				console.log('disconnected');
				if (this.refresh) this.refresh();
			};

			ws.onmessage = (msg) => this.handleMessage(msg);
		});
	}

	handleMessage(msg: MessageEvent<any>) {
		//console.log('received', msg.data);
		console.log('received message');

		try {
			const json = JSON.parse(msg.data);

			if (json.id === undefined) {
				this.connection.status = json.status;
			} else {
				this.callbacks[json.id](json);
			}
		} catch (e) {
			console.error('failed to parse ws message');
			console.log(e);
		}
	};

	updateCell(cell: CellState, json: any): [bool, CellState] {
		const { id, name, value, status } = json;

		const contentTypeHandler: Record<string, ((s: string) => string)> = {
			'text/plain': (s: string) => {
				let pre = document.createElement('pre');
				pre.innerText = s;
				return pre.outerHTML;
			},
			'text/html': (s: string) => {
				let div = document.createElement('div');
				div.innerHTML = s;
				return div.outerHTML;
			},
			'image/png': (s: string) => {
				let img = document.createElement('img');
				img.src = 'data:image/png;base64,' + s;
				return img.outerHTML;
			}
		};

		switch (name) {
			case 'output':
			case 'error':
				//console.log(value);
				let escaped = new Option(value).innerHTML;
				let convert = new Convert();
				let html = convert.toHtml(escaped);
				if (name == 'output') cell.output.value = html;
				else cell.output.error = html;
				break;
			case 'result':
				{
					let json: Record<string, string> = JSON.parse(value);
					cell.output.result = '';
					for (const k of ['text/html', 'text/plain']) {
						if (k in json) {
							const v = json[k];	
							cell.output.result += contentTypeHandler[k](v);
							break;
						}
					}
				}
				break;
			case 'data':
				{
					let json: Record<string, string> = JSON.parse(value);
					for (let [k, v] of Object.entries(json)) {
						cell.output.result += contentTypeHandler[k](v);
					}
				}
				break;
			case 'queued':
			case 'running':
				cell.output.status = value;
				break;
			case 'count':
				cell.output.executionCount = value;
				break;
			case 'complete':
				return [true, cell]
		}

		return [false, cell]
	}

	async submit(cell: CellState, callback: (cell: CellState) => void) {
		return new Promise((resolve, reject) => {
			const ws = this.connection.ws;
			if (!ws) reject('Not connected!');

			console.log('sending', cell.id);
			this.callbacks[cell.id] = (json: any) => {
				console.log('updating', json.name, 'from', JSON.stringify(cell));
				const [done, res] = this.updateCell(cell, json);
				//delete this.callbacks[cell.id];
				callback(res);
				if (done) resolve(res);
			}

			ws.send(JSON.stringify({
				id: cell.id,
				value: cell.code.value,
			}));
		});
	}
}
