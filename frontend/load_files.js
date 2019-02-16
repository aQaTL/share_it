"use strict";

const el_id = (id) => document.getElementById(id);

function load_files(path) {
    let req = new XMLHttpRequest();
    req.open("GET", "/s/" + path, false);
    req.send();
    return JSON.parse(req.responseText);
}

function display_files(root, files) {
    let filesView = el_id("files_view");
    let text = files.reduce((accumulator, file) => {
        let file_name = file.name;
        if (file.e_type === "Dir") {
            return accumulator +
                `</li><a href="#" onclick="on_dir_click('${root + "/" + encodeURI(file_name)}')">${file_name}</a><br>`;
        } else {
            return accumulator + `</li><a href="s/${root + "/" + encodeURI(file_name)}">${file_name}</a><br>`;
        }
    }, "");
    filesView.innerHTML = text;
}

function on_dir_click(new_root) {
    display_files(new_root, load_files(new_root));
}

let files = load_files("");
window.onload = () => display_files("", files);
