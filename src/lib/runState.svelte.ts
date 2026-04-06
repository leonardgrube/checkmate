let active = $state(false);
let dirty = $state(false);
let title = $state('');

export const runState = {
	get active() {
		return active;
	},
	set active(v: boolean) {
		active = v;
	},
	get dirty() {
		return dirty;
	},
	set dirty(v: boolean) {
		dirty = v;
	},
	get title() {
		return title;
	},
	set title(v: string) {
		title = v;
	}
};
