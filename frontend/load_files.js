"use strict";

function load_files() {
    let req = new XMLHttpRequest();
    req.open("GET", "/index", false);
    req.send();
    return JSON.parse(req.responseText);
}

function display_files(files) {
    let filesView = document.getElementById("files_view");
    let text = files.reduce((accumulator, filename) =>
            accumulator += `</li><a href="s/${encodeURI(filename)}">${filename}</a><br>`,
        "");
    filesView.innerHTML = text;
}

let files = load_files();
window.onload = () => display_files(files);