import './style.scss';
import { invoke } from '@tauri-apps/api';
import { open } from '@tauri-apps/api/dialog';
import { homeDir } from '@tauri-apps/api/path';
import { watch } from "tauri-plugin-fs-watch-api";

const COMMAND_PREFIX = "command:";

// Taken from https://github.com/roniemartinez/tauri-plugin-htmx
const patchedSend = async function(params) {
    // Make readonly properties writable
    Object.defineProperty(this, "readyState", { writable: true })
    Object.defineProperty(this, "status", { writable: true })
    Object.defineProperty(this, "statusText", { writable: true })
    Object.defineProperty(this, "response", { writable: true })

    // Set response
    const query = new URLSearchParams(params);
    console.log(`Calling command ${this.command} with parameters ${params}`);
    this.response = await invoke(this.command, Object.fromEntries(query));
    this.readyState = XMLHttpRequest.DONE;
    this.status = 200;
    this.statusText = "OK";

    // We only need load event to trigger a XHR response
    this.dispatchEvent(new ProgressEvent("load"));
};

window.addEventListener("DOMContentLoaded", () => {
    document.body.addEventListener('htmx:beforeSend', (event) => {
        const path = event.detail.requestConfig.path;
        if (path.startsWith(COMMAND_PREFIX)) {
            event.detail.xhr.command = path.slice(COMMAND_PREFIX.length);
            event.detail.xhr.send = patchedSend;
        }
    });
    window.dispatchEvent(new Event("helmad:libs-ready"));
    setTimeout(() => { invoke('close_splashscreen'); }, 1000);
});

window.pickLocalChartDir = async function() {
    const selectedPath = await open({
        directory: true,
        multiple: false,
        defaultPath: await homeDir(),
    });
    const fileCallback = function(events) {
        events.map((event) => {
            const newEvent = new CustomEvent("helmad:templateFileChanged", { detail: event.path });
            console.log(newEvent);
            window.dispatchEvent(newEvent);
        });
    };
    const stopWatching = await watch(
        selectedPath,
        fileCallback,
        { recursive: true },
    );
    window.addEventListener("helmad:stopFileWatch", stopWatching);
    return selectedPath;
}

window.renderLocalChart = async function() {
    const path = await window.pickLocalChartDir();
    return await invoke("local_chart", { path });
}

// const remoteRepoButton = document.querySelector("button#remoteChart");
// if (remoteRepoButton) {
//     remoteRepoButton.addEventListener("click", () => {
//         console.log("stopping watch");
//         window.dispatchEvent(new CustomEvent("helmad:stopFileWatch"))
//     });
// }


let path = null;
document.body.addEventListener("htmx:confirm", async (evt) => {
    console.log(evt);
    if (evt.detail.elt.id == "localChart") {
        evt.preventDefault();
        path = await window.pickLocalChartDir();
        evt.detail.issueRequest();
    }
});
document.body.addEventListener("htmx:configRequest", async (evt) => {
    if (evt.detail.elt.id == "localChart") {
        evt.detail.parameters["chart"] = path;
        evt.detail.parameters["name"] = "helmad";
        evt.detail.parameters["local"] = true;
        evt.detail.parameters["values"] = "";
        evt.detail.parameters["resources"] = await invoke("template", {chart: path, values: "", name: "helmad"});
    }
});

