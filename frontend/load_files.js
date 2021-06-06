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
            `</li><a class="dir" href="javascript: void(0)" onclick="on_dir_click('${file_path}/')">${file.name}</a><br>`
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

function dragstartHandler(ev) {
    console.debug("dragstart");
    ev.stopPropagation();
    ev.preventDefault();
}

function dragoverHandler(ev) {
    console.debug("dragover");
    ev.stopPropagation();
    ev.preventDefault();
}

async function updateProgress(percentage) {
    console.debug(`Upload progress ${percentage}%`);
    let bar = document.getElementById("uploadBoxProgress").ldBar;
    bar.set(percentage);
}

async function dropHandler(ev) {
    ev.stopPropagation();
    ev.preventDefault();

    console.debug("drop");
    for (let i = 0; i < ev.dataTransfer.files.length; i++) {
        let file = ev.dataTransfer.files[i];
        console.log(`Uploading file ${file.name} weighing ${file.size}`);

        openUploadBox(file.name);

        let filenameParam = encodeURIComponent(file.name);
        try {
            let uploadRequest = new Promise(((resolve, reject) => {
                let req = new XMLHttpRequest();
                req.open("POST", `/upload/${filenameParam}`, true);
                req.upload.addEventListener("progress", async (e) => {
                    await updateProgress(i, Math.round((e.loaded * 100.0 / e.total) || 100));
                });
                req.addEventListener("readystatechange", (e) => {
                    if (req.readyState !== 4) {
                        return;
                    }

                    if (req.status === 200) {
                        resolve(undefined);
                    } else {
                        reject(req.response);
                    }
                });
                req.send(file);
            }));

            await uploadRequest;
            console.debug("File upload succeeded");

            dir_stack = [""];
            display_files("", await load_files(""));

        } catch (e) {
            console.error("Failed to upload file: ", e);
        } finally {
            closeUploadBox();
        }
    }
}

window.addEventListener("DOMContentLoaded", () => {
    document.addEventListener("dragstart", dragstartHandler);
    document.addEventListener("dragover", dragoverHandler);
    document.addEventListener("drop", dropHandler);
});

function openUploadBox(filename) {
    let uploadBox = document.getElementById("uploadBox");
    let uploadFilename = uploadBox.querySelector("#uploadFilename");

    uploadFilename.textContent = filename;
    uploadBox.style.display = "block";

    updateProgress(0, 0);
}

function closeUploadBox() {
    let uploadBox = document.getElementById("uploadBox");
    uploadBox.style.display = "none";
}

// When the user clicks anywhere outside of the modal, close it
window.onclick = function (event) {
    let uploadBox = document.getElementById("uploadBox");
    if (event.target === uploadBox) {
        uploadBox.style.display = "none";
    }
}

window.addEventListener("keypress", ev => {
    if (ev.key === "Escape") {
        closeUploadBox();
    }
});