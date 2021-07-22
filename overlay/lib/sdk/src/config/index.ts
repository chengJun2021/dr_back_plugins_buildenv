export interface Config {
    [key: string]: object
}

export class ConfigFile implements Config {
    [key: string]: object

    constructor(object: object) {
        for (let key of Object.keys(object)) {
            this[key] = object[key];
        }
    }
}
