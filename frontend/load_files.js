"use strict";

const el_id = (id) => document.getElementById(id);

let dir_stack = [];

function load_files(path) {
    return new Promise((resolve, reject) => {
        let req = new XMLHttpRequest();
        req.open("GET", "/s/" + path, true);
        req.onload = () => {
            resolve(JSON.parse(req.responseText));
        };
        req.send();
    });
}

const display_files = (root, files) => el_id("files_view").innerHTML =
    files.reduce((accumulator, file) => {
        let file_path = root + encodeURI(file.name);
        return accumulator + (file.e_type === "Dir" ?
            `</li><a class="dir" href="#" onclick="on_dir_click('${file_path}/')">${file.name}</a><br>`
            :
            `</li><a class="file" href="s/${file_path}">${file.name}</a><br>`);
    }, "");

async function on_dir_click(new_root) {
    dir_stack.push(new_root);
    display_files(new_root, await load_files(new_root));
    el_id("curr_path").innerHTML = dir_stack[dir_stack.length - 1];
}

async function load_previous_dir() {
    if (dir_stack.length <= 1) {
        return;
    }
    dir_stack.pop();
    await on_dir_click(dir_stack.pop());
}

(async () => {
    let files = await load_files("");
    dir_stack.push("");
    window.onload = () => display_files("", files);
    if (document.readyState === "interactive" || document.readyState === "complete") {
        display_files("", files);
    }
})();
