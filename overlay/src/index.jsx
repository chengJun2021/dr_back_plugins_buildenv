import React from 'react';
import './index.scss';
import "../package.json";
import includeme from "./includeme.png";
import includemewebp from "./includeme.webp";
import thicc from "./thicc.png";
import {PluginState} from "dr-plugin-sdk/src/pluginState";
import {ConfigYML} from "dr-plugin-sdk/src/config/yml";

const state = new PluginState(new ConfigYML());

console.log(state.getConfigNode("invalid"))

const TestyMcTestface = () => <div className="red-text">{state.getConfigNode("owo")}</div>

state.renderOnLoad(<div>
    <TestyMcTestface/>
    <img src={includeme} alt="webpack logo, experimenting with loading images atm"/>
    <img src={includemewebp} alt="webpack logo, experimenting with loading images, webp edition"/>
    <img src={thicc} alt="a very big image to stress test webpack"/>
</div>);
