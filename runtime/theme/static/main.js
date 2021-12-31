import { parseState } from "./state_parser.js";

function addLinkEvents() {
    window.addEventListener("hashchange", () => {
        closeAllOffscreens();
        applyStateFromHash();
    });
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
            // We use a copy of the `code#code-block` tag that has transparent font
            // color. This way we can highlight the background of the code
            // without disturbing the syntax highlighting.
            let codeBgEl = document.getElementById("code-block-bg");
            let code = codeBgEl.innerText;

            let fromIdx = code.search(fromRegex);
            if (fromIdx == -1) {
                return;
            }

            // we start searching from `fromIdx+1` but we need the index to be
            // absolute with respect to `code`, so we add `fromIdx+1` to the
            // resulting index.
            let toIdx = code.substring(fromIdx + 1).search(toRegex) + fromIdx + 1;
            if (toIdx == -1) {
                return;
            }

            // from start to first highlight span
            let codeHtmlHighlighted = escapeHtml(code.substring(0, fromIdx));
            codeHtmlHighlighted += "<span class='highlight'>";
            // from first highlight span to second highlight span
            codeHtmlHighlighted += escapeHtml(code.substring(fromIdx, toIdx + 1));
            codeHtmlHighlighted += "</span>";
            // we don't need the rest of the code since we only use the code
            // for alignment.

            codeBgEl.innerHTML = codeHtmlHighlighted;
        });
    }
}

function addDirectoryButtonsEvents() {
    let dirButtons = document.getElementsByClassName("directory-button");

    for (let i = 0; i < dirButtons.length; i++) {
        const dirButton = dirButtons[i];
        dirButton.addEventListener("click", function(event) {
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
        .getElementById("file-name")
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

function addOffscreenTogglerEvents() {
    let offscreenTogglers = document.getElementsByClassName("offscreen-open");

    for (let i = 0; i < offscreenTogglers.length; i++) {
        const offscreenToggler = offscreenTogglers[i];
        const targetId = offscreenToggler.getAttribute("data-offscreen-target");
        const target = document.getElementById(targetId);
        const closeButtons = target.getElementsByClassName("offscreen-close");

        for (let j = 0; j < closeButtons.length; j++) {
            closeButtons[j].addEventListener("click", () => {
                target.classList.toggle("open");
            });
        }

        offscreenToggler.addEventListener("click", () => {
            target.classList.toggle("open");
        });
    }
}

function closeAllOffscreens() {
    let offscreens = document.querySelectorAll(".offscreen.open");
    for (let i = 0; i < offscreens.length; i++) {
        offscreens[i].classList.remove("open");
    }
}

function escapeHtml(unsafe) {
    return unsafe
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#039;");
}

function main() {
    applyStateFromHash();
    addLinkEvents();
    addDirectoryButtonsEvents();
    disableDisabledAnchors();
    addOffscreenTogglerEvents();
}

main();
