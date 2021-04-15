import React from 'react';
import './index.scss';
import "../package.json";
import * as ReactDOM from "react-dom";
import includeme from "./includeme.png";

const TestyMcTestface = () => <div className="red-text">Test</div>

document.addEventListener("DOMContentLoaded", () => {
    ReactDOM.render(<div>
        <TestyMcTestface/>
        <img src={includeme} alt="webpack logo, experimenting with loading images atm"/>
    </div>, document.body)
})
