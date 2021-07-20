/**
 * @fileoverview Prevent `import`s of items outside of the `src/` directory from being used
 * @author dataReachable Plugin Systems Team
 */

"use strict";

//------------------------------------------------------------------------------
// Rule Definition
//------------------------------------------------------------------------------

const {default: moduleVisitor} = require('eslint-module-utils/moduleVisitor');

const RELATIVE_IMPORT_ERROR_MSG = "relative imports outside of the sources folder is not allowed";
const PATH_SEPARATOR = /[\/\\]/;

module.exports = {
    meta: {
        type: "error",
        docs: {
            description: "prohibit relative imports of files outside of `src/`",
        },
    },
    create: function (context) {
        const filename = context.getFilename();
        if (filename === '<text>') return {}; // can't check a non-file

        const filePath = filename
            .substr(`${process.cwd()}/src/`.length)
            .split(PATH_SEPARATOR);
        filePath.pop(); // Remove the file name

        const reconstructImportPath = function (node) {
            const constructedPath = Array.from(filePath);
            const depPath = node.value.split(PATH_SEPARATOR);

            for (let elem of depPath) {
                if (elem === "..") {
                    if (constructedPath.length > 0) {
                        constructedPath.pop();
                    } else {
                        context.report({node, message: RELATIVE_IMPORT_ERROR_MSG})
                    }
                } else if (elem !== ".") {
                    constructedPath.push(elem);
                }
            }
        };

        return moduleVisitor(reconstructImportPath, context.options[0]);
    }
};
