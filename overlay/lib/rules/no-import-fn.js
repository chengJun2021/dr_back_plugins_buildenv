/**
 * @fileoverview Prevent `import()` from being used
 * @author dataReachable Plugin Systems Team
 */

"use strict";

//------------------------------------------------------------------------------
// Rule Definition
//------------------------------------------------------------------------------

const IMPORT_FUNCTION_MESSAGE = "use of `import()` is not allowed";

module.exports = {
    meta: {
        type: "error",
        docs: {
            description: "disallow functional style use of `import`",
        },
    },
    create: function (context) {
        const analyze = function (node, subject) {
            if (subject === 'ImportExpression') {
                context.report({
                    node: node,
                    message: IMPORT_FUNCTION_MESSAGE
                })
            }
        };

        return {
            VariableDeclarator: node => analyze(node.init, node.init.type),
            ExpressionStatement: node => analyze(node, node.expression.type)
        };
    }
};
