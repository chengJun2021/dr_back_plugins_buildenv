import {ConfigFile} from "./index";

// Helper class to load config.yml from the source directory
export class ConfigYML extends ConfigFile {
    constructor() {
        let yml = require("../../../src/config.yml");

        super(yml)
    }
}
