let lastOpenFile = null;

export default function openFile(path, callback) {
    if (lastOpenFile == path) {
        if (callback) {
            callback();
        }
        return;
    }

    lastOpenFile = path;

    let workspaceUrl = document.body.getAttribute("data-workspace-url");
    let url = workspaceUrl + "/" + path + ".html";

    fetch(url)
        .then(response => {
            if (!response.ok) {
                throw new Error("request error");
            }
            return response.text();
        })
        .then(contents => {
            updateFileView(path, contents);
            if (callback) {
                callback();
            }
        })
        .catch(error => console.error(`fetch error: ${error}`));
}

function updateFileView(fileName, contents) {
    let fileViewerEl = document.getElementById("file-viewer");
    let fileNameEl = fileViewerEl.querySelector(".file-name");
    let lineNumbersEl = fileViewerEl.querySelector(".line-numbers");
    let codeEl = fileViewerEl.querySelector("code.original");
    let codeCopyEl = fileViewerEl.querySelector("code.copy");

    fileNameEl.innerText = fileName;

    // # Line numbers
    let lineNumbersHtml = "";
    let line = 1;
    for (let char of contents) {
        if (char == "\n") {
            lineNumbersHtml += lineNumberTemplate(line);
            line += 1;
        }
    }

    // we want to show at least one line number
    if (lineNumbersHtml == "") {
        lineNumbersHtml = lineNumberTemplate(1);
    }

    lineNumbersEl.innerHTML = lineNumbersHtml;

    // # Contents
    // `contents` is already escaped by Codasai at build time
    codeEl.innerHTML = contents;
    codeCopyEl.innerHTML = contents;
}

function lineNumberTemplate(line) {
    return `<a href="#">${line}</a>`;
}
