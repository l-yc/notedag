import { v4 as uuidv4 } from 'uuid';

export type UUID = string;

export class CellInput {
	constructor(
		public value: string,
		public syntax: string,
	) {}

	static default(): CellInput {
		return new CellInput('', 'code');
	}
}

export class CellOutput {
	constructor(
		public value: string,
		public error: string,
		public result: string,
		public executionCount: string,
	) {}

	static default(): CellOutput {
		return {
			value: '',
			error: '',
			result: '',
			executionCount: ' ',
		}
	}
}

export class Cell {
	constructor(
		public id: UUID,
		public code: CellInput,
		public meta: object,
		public output: CellOutput,
	) {}

	static default(): Cell {
		return new Cell(
			uuidv4() as UUID,
			CellInput.default(),
			{},
			CellOutput.default(),
		);
	}

	public clearOutput(): void {
		this.output = CellOutput.default();
	}
}

export class Group {
	constructor(
		public id: UUID,
		public name: string,
		public cells: UUID[],
		public children: UUID[],

		/// User state
		public nextChild: UUID | null,
	) {}

	static default(): Group {
		return {
			id: uuidv4() as UUID,
			name: 'untitled group',
			cells: [],
			children: [],
			nextChild: null,
		}
	}
}

export class NoteDAG {
	constructor(
		/// Smallest unit of code
		public cells: Record<UUID, Cell>,

		/// Groups cells together with some metadata
		public groups: Record<UUID, Group>,

		/// Entry point for execution
		public root: UUID,

		/// User state
		public focusedGroup: UUID = root,
		public focusedCell: UUID = groups[root].cells[0],
	) {}

	static default(): NoteDAG {
		let cells: Record<UUID, Cell> = {};
		let groups: Record<UUID, Group> = {};

		let cell = Cell.default();
		cells[cell.id] = cell;

		let group = Group.default();
		group.cells.push(cell.id);
		groups[group.id] = group;

		const root = group.id;

		let ret = new NoteDAG(cells, groups, root);
		return ret;
	}

	static from_file_data(jsonStr: string): NoteDAG {
		try {
			const json = JSON.parse(jsonStr);
			return new NoteDAG(json.cells, json.groups, json.root);
		} catch (e) {
			//console.error('failed to parse NoteDag from JSON:', e);
			console.log('failed to parse NoteDAG from JSON'); 
			return NoteDAG.default();
		}
	}

	/// handlers:focus
	focusGroup(groupId: UUID) {
		const group = this.groups[groupId];
		const cellId = group.cells.indexOf(this.focusedCell) === -1 ? group.cells[0] : this.focusedCell;
		this.focusCell(groupId, cellId);
	}

	focusCell(groupId: UUID, cellId: UUID) {
		this.focusedGroup = groupId;
		this.focusedCell = cellId;
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
		const newCell = Cell.default();
		this.cells[newCell.id] = newCell;
		if (idx === undefined) this.groups[groupId].cells.push(newCell.id);
		else this.groups[groupId].cells.splice(idx, 0, newCell.id);
		this.focusedCell = newCell.id;
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
		const newGroup = Gruop.default();
		this.groups[newGroup.id] = newGroup;
		this.addNewCell(newGroup.id);
		
		const parent = this.groups[groupId ?? this.focusedGroup];
		parent.children.push(newGroup.id);
		parent.nextChild = newGroup.id;

		this.focusGroup(newGroup.id);
	}

	/// handlers:delete
	deleteCell(cellId: UUID, groupId: UUID) {
		delete this.cells[cellId];

		let group = this.groups[groupId];
		const idx = group.cells.indexOf(cellId);
		group.cells.splice(idx, 1);

		if (this.focusedCell === cellId) {
			this.focusedCell = group.cells[Math.min(idx, group.cells.length-1)];
		}
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
		}
	}

	/// handlers:clear
	clearGroup(groupId: string) {
		let group = this.groups[groupId];
		for (let id in group.cells) {
			this.cells[id].clearOutput();
		}
	}

	clearOutput() {
		for (let id in this.cells) {
			this.cells[id].clearOutput()
		}
	}

	setNextGroup(groupId: string, nextGroupId: string) {
		this.groups[groupId].nextChild = nextGroupId;
		this.focusGroup(nextGroupId);
		console.log('focused', this.focusedGroup);
	}

	get activeGroupChain(): Group[] {
		let ret = [];
		let id: string | null = this.root;
		console.log('starting from', id);
		while (id !== null) {
			const group: Group = this.groups[id];
			ret.push(group);
			id = group.nextChild;
		}
		return ret;
	}
	

}
