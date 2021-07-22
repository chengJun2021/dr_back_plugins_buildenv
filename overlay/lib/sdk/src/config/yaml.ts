import {ConfigFile} from "./index";

export class ConfigYAML extends ConfigFile {
    constructor() {
        let yaml = require("../../../../src/config.yaml");

        super(yaml)
    }
}
