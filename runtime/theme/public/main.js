import { parseState } from "./state_parser.js";

function addLinkEvents() {
    window.addEventListener("hashchange", applyStateFromHash);
}

function applyStateFromHash() {
    let state = decodeURIComponent(window.location.hash);

    const magic_prefix = "#csai:";

    if (state.startsWith(magic_prefix)) {
        state = parseState(state.substring(magic_prefix.length));
        applyState(state);
    }
}

function applyState(state) {
    if (state.action == "open_file") {
        let file = state.argument("file");
        openFile(file);
    }
}

function addDirectoryButtonsEvents() {
    let dirButtons = document.getElementsByClassName("directory-button");

    for (let i = 0; i < dirButtons.length; i++) {
        const dirButton = dirButtons[i];
        dirButton.addEventListener("click", function (event) {
            event.currentTarget.parentNode.classList.toggle("open");
        });
    }
}

function openFile(filePath) {
    let prefix = document.body.getAttribute("data-workspace-url");
    let url = prefix + "/" + filePath + ".html";

    //TODO: Add error alert
    fetch(url)
        .then((response) => {
            if (!response.ok) {
                throw new Error("request error");
            }
            return response.text();
        })
        .then((code) => {
            updateCodeView(filePath, code);
        })
        .catch((error) => console.error("fetch error: ", error));
}

function updateCodeView(fileName, code) {
    // - content
    let codeBlock = document.getElementById("code-block");
    codeBlock.innerHTML = code;

    // - file name
    document
        .getElementsByClassName("file-name")[0]
        .getElementsByTagName("span")[0].innerText = fileName;

    // - line numbers
    let lineNumbersHtml = "";
    let line = 1;
    for (let char of code) {
        if (char == "\n") {
            // is this way of constructing a string wrong?
            // I doubt it matters since the average case should be in the hundreds of lines.
            lineNumbersHtml += lineNumberTemplate(line);
            line += 1;
        }
    }

    // empty files look weird without any line number so at least add one.
    if (lineNumbersHtml == "") {
        lineNumbersHtml = lineNumberTemplate(1);
    }

    let lineNumbers = document.getElementsByClassName("line-numbers")[0];
    lineNumbers.innerHTML = lineNumbersHtml;
}

function lineNumberTemplate(line) {
    return `<a href=\"#\">${line}</a>\n`;
}

function main() {
    applyStateFromHash();
    addLinkEvents();
    addDirectoryButtonsEvents();
}

main();
