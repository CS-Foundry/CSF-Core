declare module "@novnc/novnc" {
    export default class RFB extends EventTarget {
        constructor(target: HTMLElement, urlOrChannel: string, options?: Record<string, unknown>);
        scaleViewport: boolean;
        resizeSession: boolean;
        disconnect(): void;
    }
}
