let open = $state(false);

export const settingsDialog = {
    get open() {
        return open;
    },
    show() {
        open = true;
    },
    hide() {
        open = false;
    },
    set(value: boolean) {
        open = value;
    },
};
