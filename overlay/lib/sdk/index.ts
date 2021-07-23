/// This is the dR Plugins System Platform's SDK
/// Copyright (c) 2020- dataReachable Pty Limited

import ReactDOM from "react-dom";

// Helper function to render the passed JSX elements when the document loads
export function renderOnLoad(body: JSX.Element) {
    document.addEventListener("DOMContentLoaded", () => {
        const container = document.createElement("div");
        document.body.appendChild(container);

        ReactDOM.render(body, container)
    })
}
