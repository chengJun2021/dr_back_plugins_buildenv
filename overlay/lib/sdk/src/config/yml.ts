import {ConfigFile} from "./index";

export class ConfigYML extends ConfigFile {
    constructor() {
        let yml = require("../../../../src/config.yml");

        super(yml)
    }
}
