// Genuine import (path traversal detected)
import pkg1 from "../package.json";
// Disingenuous import (webpack copies file)
import("../package.json");
// Disingenuous import (inline'd)
let pkg2 = import("../package.json");

console.log(JSON.stringify(pkg1));
console.log(JSON.stringify(pkg2));
