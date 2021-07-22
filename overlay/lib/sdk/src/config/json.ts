import {ConfigFile} from "./index";

export class ConfigJSON extends ConfigFile {
    constructor() {
        let json = require("../../../../src/config.json");

        super(json)
    }
}
