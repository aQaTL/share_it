"use strict";

const el_id = (id) => document.getElementById(id);

let dir_stack = [];

function load_files(path) {
    let req = new XMLHttpRequest();
    req.open("GET", "/s/" + path, false);
    req.send();
    return JSON.parse(req.responseText);
}

const display_files = (root, files) => el_id("files_view").innerHTML =
    files.reduce((accumulator, file) => {
        let file_path = root + encodeURI(file.name);
        return accumulator + (file.e_type === "Dir" ?
            `</li><a href="#" onclick="on_dir_click('${file_path}/')">${file.name}</a><br>`
            :
            `</li><a href="s/${file_path}">${file.name}</a><br>`);
    }, "");

function on_dir_click(new_root) {
    dir_stack.push(new_root);
    display_files(new_root, load_files(new_root));
    el_id("curr_path").innerHTML = dir_stack[dir_stack.length - 1];
}

function load_previous_dir() {
    if (dir_stack.length <= 1) {
        return;
    }
    dir_stack.pop();
    on_dir_click(dir_stack.pop());
}

let files = load_files("");
dir_stack.push("");
window.onload = () => display_files("", files);
