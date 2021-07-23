import {ConfigFile} from "./index";

// Helper class to load config.json from the source directory
export class ConfigJSON extends ConfigFile {
    constructor() {
        let json = require("../../../../src/config.json");

        super(json)
    }
}
