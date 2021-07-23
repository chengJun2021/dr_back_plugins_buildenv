import {ConfigFile} from "./index";

// Helper class to load config.yaml from the source directory
export class ConfigYAML extends ConfigFile {
    constructor() {
        let yaml = require("../../../../src/config.yaml");

        super(yaml)
    }
}
