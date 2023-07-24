import { v4 as uuidv4 } from 'uuid';

export type UUID = string;

export interface CellInputState {
	value: string;
	syntax: string;
}

function defaultCellInput(): CellInputState {
	return {
		value: '',
		syntax: 'code',
	}
}

export interface CellOutputState {
	value: string;
	error: string;
	result: string;
	executionCount: string;
}

function defaultCellOutput(): CellOutputState {
	return {
		value: '',
		error: '',
		result: '',
		executionCount: ' ',
	}
}

export interface CellState {
	id: UUID;
	code: CellInputState;
	meta: object;
	output: CellOutputState;
}

function defaultCell(): CellState {
	return {
		id: uuidv4() as UUID,
		code: defaultCellInput(),
		meta: {},
		output: defaultCellOutput(),
	}
}

function clearOutput(cell: CellState): void {
	cell.output = defaultCellOutput();
}

export interface GroupState {
	id: UUID;
	name: string;
	cells: UUID[];
	children: UUID[];

	/// User state
	nextChild: UUID | null;
}

function defaultGroup(): GroupState {
	return {
		id: uuidv4() as UUID,
		name: 'untitled group',
		cells: [],
		children: [],
		nextChild: null,
	}
}

export class NoteDAGState {
	_refresh?: () => void;

	constructor(
		/// Smallest unit of code
		public cells: Record<UUID, CellState>,

		/// Groups cells together with some metadata
		public groups: Record<UUID, GroupState>,

		/// Entry point for execution
		public root: UUID,

		/// User state
		public focusedGroup: UUID = root,
		public focusedCell: UUID = groups[root].cells[0],
		public activeGroupChain: GroupState[] = [],
	) {
		this.rebuildActiveGroupChain();
	}

	static default(): NoteDAGState {
		let cells: Record<UUID, CellState> = {};
		let groups: Record<UUID, GroupState> = {};

		let cell = defaultCell();
		cells[cell.id] = cell;

		let group = defaultGroup();
		group.cells.push(cell.id);
		groups[group.id] = group;

		const root = group.id;

		let ret = new NoteDAGState(cells, groups, root);
		return ret;
	}

	static load(jsonStr: string, _refresh?: () => void): NoteDAGState {
		try {
			const json = JSON.parse(jsonStr);
			let ret = new NoteDAGState(json.cells, json.groups, json.root);
			ret._refresh = _refresh;
			return ret;
		} catch (e) {
			//console.error('failed to parse NoteDag from JSON:', e);
			console.log('failed to parse NoteDAG from JSON'); 
			return NoteDAGState.default();
		}
	}

	refresh() {
		if (!this._refresh) return false;
		this._refresh.call(this);
		return true;
	}

	/// handlers:focus
	focusGroup(groupId: UUID) {
		console.log('focusing', groupId);
		const group = this.groups[groupId];
		console.log(this, this.groups, 'focusing', group);
		const cellId = group.cells.indexOf(this.focusedCell) === -1 ? group.cells[0] : this.focusedCell;
		this.focusCell(groupId, cellId);
	}

	focusCell(groupId: UUID, cellId: UUID) {
		this.focusedGroup = groupId;
		this.focusedCell = cellId;
		this.rebuildActiveGroupChain();
	}

	focusCellBefore(groupId: UUID = this.focusedGroup, cellId: UUID = this.focusedCell) {
		const group = this.groups[groupId];
		const idx = group.cells.indexOf(cellId);
		if (idx > 0) this.focusCell(groupId, group.cells[idx - 1])
	}

	focusCellAfter(groupId: UUID = this.focusedGroup, cellId: UUID = this.focusedCell) {
		const group = this.groups[groupId];
		const idx = group.cells.indexOf(cellId);
		if (idx < group.cells.length - 1) this.focusCell(groupId, group.cells[idx + 1])
	}

	/// handlers:add
	addNewCell(groupId: UUID, idx?: number) {
		const newCell = defaultCell();
		this.cells[newCell.id] = newCell;
		if (idx === undefined) this.groups[groupId].cells.push(newCell.id);
		else this.groups[groupId].cells.splice(idx, 0, newCell.id);
		this.focusCell(groupId, newCell.id);
	}

	addNewCellBefore(groupId: UUID = this.focusedGroup, cellId: UUID = this.focusedCell) {
		const group = this.groups[groupId];
		const idx = group.cells.indexOf(cellId);
		this.addNewCell(groupId, idx);
	}

	addNewCellAfter(groupId: UUID = this.focusedGroup, cellId: UUID = this.focusedCell) {
		const group = this.groups[groupId];
		const idx = group.cells.indexOf(cellId);
		this.addNewCell(groupId, idx+1);
	}

	addNewGroup(groupId?: UUID) {
		const parent = this.groups[groupId ?? this.focusedGroup];

		const newGroup = defaultGroup();
		this.groups[newGroup.id] = newGroup;
		this.addNewCell(newGroup.id);
		
		parent.children.push(newGroup.id);
		parent.nextChild = newGroup.id;

		this.focusGroup(newGroup.id);
	}

	/// handlers:delete
	deleteCell(cellId: UUID, groupId: UUID) {
		delete this.cells[cellId];

		alert('deleting ' + cellId);
		let group = this.groups[groupId];
		const idx = group.cells.indexOf(cellId);
		group.cells.splice(idx, 1);
		alert(JSON.stringify(group.cells));

		if (this.focusedCell === cellId) {
			alert('updating focused cell');
			this.focusCell(group.id, group.cells[Math.min(idx, group.cells.length-1)]);
		} else this.refresh();
	}

	deleteGroup(groupId: UUID, parentGroupId: UUID) {
		if (groupId == this.root) {
			alert('cannot delete root group');
			return;
		}

		const group = this.groups[groupId];
		if (group.children.length > 0) {
			alert('cannot delete group with dependent groups');
			return;
		}
		delete this.groups[groupId];

		const parent = this.groups[parentGroupId];
		const idx = parent.children.indexOf(groupId);
		parent.children.splice(idx, 1);
		if (parent.nextChild === groupId) parent.nextChild = parent.children[Math.min(idx, parent.children.length-1)] || null;

		if (this.focusedGroup === groupId) {
			if (parent.nextChild === null) this.focusGroup(parent.id);
			else this.focusGroup(parent.nextChild);
		} else this.refresh();
	}

	/// handlers:clear
	clearCell(cellId: string) {
		console.log('clearing', cellId);
		clearOutput(this.cells[cellId]);
	}

	clearGroup(groupId: string) {
		const group = this.groups[groupId];
		console.log(groupId, group.cells);
		for (const id of group.cells) {
			this.clearCell(id);
		}
	}

	clearOutput() {
		for (const id in this.cells) {
			clearOutput(this.cells[id]);
		}
	}

	setNextGroup(groupId: string, nextGroupId: string) {
		this.groups[groupId].nextChild = nextGroupId;
		this.focusGroup(nextGroupId);
		this.rebuildActiveGroupChain();
		console.log('focused', this.focusedGroup);
	}

	rebuildActiveGroupChain() {
		let ret = [];
		let id: string | null = this.root;
		//console.log('starting from', id);
		while (id !== null) {
			const group: GroupState = this.groups[id];
			ret.push(group);
			id = group.nextChild;
		}
		this.activeGroupChain = ret;
		console.log('rebuilding');
		this.refresh();
	}
	

}
