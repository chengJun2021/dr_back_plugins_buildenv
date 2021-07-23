// Type bound on configs
export interface Config {
    [key: string]: object
}

// Base class for ConfigYML / ConfigJSON, not for end users.
export class ConfigFile implements Config {
    [key: string]: object

    // Extracts the fields specified in the params into itself.
    constructor(object: object) {
        for (let key of Object.keys(object)) {
            this[key] = object[key];
        }
    }
}
