let dirty = $state(false);

export const formGuard = {
	get dirty() {
		return dirty;
	},
	set dirty(v: boolean) {
		dirty = v;
	}
};
