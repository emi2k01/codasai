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
    } else if (state.action == "highlight_regex") {
        let file = state.argument("file");
        let from = state.argument("from");
        let to = state.argument("to");

        let fromRegex = new RegExp(from, "m");
        let toRegex = new RegExp(to, "m");

        openFile(file, function() {
            // we use a duplicate of the code block with transparent font color
            // so that we highlight blocks align with the code since we don't
            // want to lose the syntax highlighting and we want to avoid mixing
            // the HTML tags for line highlighting and syntax highlighting
            let codeBgEl = document.getElementById("code-block-bg");
            let code = codeBgEl.innerText;

            let fromIdx = code.search(fromRegex);
            if (fromIdx == -1) {
                return;
            }
            // we start searching after `fromIdx` but we need the index to be
            // based on the whole code so we add the `fromIdx` (plus 1 'cause
            // 0-based)
            let toIdx = code.substring(fromIdx+1).search(toRegex) + fromIdx+1;
            if (toIdx == -1) {
                return;
            }

            let codeHtmlHighlighted = code.substring(0, fromIdx);
            codeHtmlHighlighted += "<span class='highlight'>";
            codeHtmlHighlighted += code.substring(fromIdx, toIdx+1);
            codeHtmlHighlighted += "</span>";
            // we don't need the rest of the code since we only use the code
            // for alignment

            codeBgEl.innerHTML = codeHtmlHighlighted;
        });
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

function openFile(filePath, callback) {
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
            if (callback != undefined) {
                callback();
            }
        })
        .catch((error) => console.error("fetch error: ", error));
}

function updateCodeView(fileName, code) {
    document.getElementById("file-viewer").classList.remove("closed");
    // - content
    let codeBlock = document.getElementById("code-block");
    codeBlock.innerHTML = code;
    let codeBlockBg = document.getElementById("code-block-bg");
    codeBlockBg.innerHTML = code;

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

function disableDisabledAnchors() {
    let anchors = document.querySelectorAll("a.disabled");
    for (let i = 0; i < anchors.length; i++) {
        anchors[i].addEventListener("click", function(e) {
            e.preventDefault();
        });
    }
}

function main() {
    applyStateFromHash();
    addLinkEvents();
    addDirectoryButtonsEvents();
    disableDisabledAnchors();
}

main();
