import dot from "dot-object";
import {Config} from "./config";

// Communicate with the parent process to obtain information such as config
// and to share the state with the parent process
export class PluginComms<T> {
    // The instanceId as received from the parent process
    private _instanceId?: string;
    // The state as set by the caller, and sent to the parent process using the setter
    private _state?: T;
    // The config as received from the parent process
    private config: Config = {}
    // The defaultConfig can be provided by the plugin via the ConfigY(A)ML & ConfigJSON helper classes.
    // It can also be hard-coded by the caller.
    private readonly defaultConfig: Config;

    // Establishes the communications channels with the survey form.
    //
    // It's recommended that only one instance of this class is instantiated,
    // and that it's created as soon as practicable. As delaying instantiation
    // may impact on user experience, especially with custom configurations.
    constructor(defaultConfig: Config = {}) {
        this.defaultConfig = defaultConfig;
        this.listener();
        this.obtainInstanceId();
    }

    // Read a node of config using a dot-notation. For more information, please
    // consult the documentation at the npm package [dot-object](https://www.npmjs.com/package/dot-object)
    public getConfigNode(path): object {
        return dot.pick(path, this.config) ?? dot.pick(path, this.defaultConfig) ?? null;
    }

    // Binds an inter-window listener to this class. All received messages will
    // directly be fed into our fields.
    listener() {
        window.addEventListener("message", e => {
            this._instanceId = e.data.instanceId;
            this.config = e.data.config ?? {};
        });
    }

    // Acquires the instanceId from the parent process asynchronously by aggressively requesting it.
    obtainInstanceId() {
        window.parent.postMessage({}, '*');

        const interval = setInterval(() => {
            if (this._instanceId !== undefined) clearInterval(interval);

            window.parent.postMessage({}, '*');
        }, 50);
    }

    // Obtain the current state as cached by the communications SDK.
    public get state() {
        return this._state;
    }

    // Update the current state, placing it into the SDK's cache and sending it to the survey.
    public set state(state) {
        this._state = state;
        this.tryPostState();
    }

    // Attempt to send the state to the parent.
    tryPostState() {
        window.parent.postMessage({
            instanceId: this._instanceId,
            state: this._state ?? {}
        }, '*');
    }
}
