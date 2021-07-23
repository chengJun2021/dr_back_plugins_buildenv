import React from 'react';
import './index.scss';
import "../package.json";
import includeme from "./includeme.png";
import includemewebp from "./includeme.webp";
import thicc from "./thicc.png";
import {PluginComms} from "dr-plugin-sdk/pluginComms";
import {renderOnLoad} from "dr-plugin-sdk";
import {ConfigYML} from "dr-plugin-sdk/config/yml";

const state = new PluginComms(new ConfigYML());

console.log(state.getConfigNode("invalid"))

const TestyMcTestface = () => <div className="red-text">{state.getConfigNode("owo")}</div>

renderOnLoad(<div>
    <TestyMcTestface/>
    <img src={includeme} alt="webpack logo, experimenting with loading images atm"/>
    <img src={includemewebp} alt="webpack logo, experimenting with loading images, webp edition"/>
    <img src={thicc} alt="a very big image to stress test webpack"/>
</div>);
