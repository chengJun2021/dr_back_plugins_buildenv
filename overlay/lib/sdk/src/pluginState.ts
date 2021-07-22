import dot from "dot-object";
import {Config} from "./config";
import ReactDOM from "react-dom";

export class PluginState<T> {
    public instanceId?: string;
    public pluginState?: T;
    private config: Config = {}

    constructor(private defaultConfig: Config = {}) {
        this.listener();
        this.obtainInstanceId();
    }

    public getConfigNode(path): object {
        return dot.pick(path, this.config) ?? dot.pick(path, this.defaultConfig) ?? null;
    }

    listener() {
        window.addEventListener("message", e => {
            this.instanceId = e.data.instanceId;
            this.config = e.data.config ?? {};
        });
    }

    obtainInstanceId() {
        const interval = setInterval(() => {
            if (this.instanceId !== undefined) clearInterval(interval);

            window.parent.postMessage({}, '*');
        }, 500);
    }

    public get state() {
        return this.pluginState;
    }

    public set state(state) {
        this.pluginState = state;
        this.tryPostState();
    }

    tryPostState() {
        window.parent.postMessage({
            instanceId: this.instanceId,
            state: this.pluginState ?? {}
        }, '*');
    }

    public renderOnLoad(body: JSX.Element) {
        document.addEventListener("DOMContentLoaded", () => {
            const container = document.createElement("div");
            document.body.appendChild(container);

            ReactDOM.render(body, container)
        })
    }
}
