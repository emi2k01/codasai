function main() {
    addFileButtonsEvents();
    addDirectoryButtonsEvents();
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

function addFileButtonsEvents() {
    let fileButtons = document.getElementsByClassName("file-button");

    for (let i = 0; i < fileButtons.length; i++) {
        const fileButton = fileButtons[i];
        fileButton.addEventListener("click", function (event) {
            let url = fileButton.getAttribute("data-url");
            let fileName = event.currentTarget
                .getElementsByClassName("file-name")[0]
                .getAttribute("data-file-name");

            //TODO: Add error alert
            fetch(url)
                .then((response) => {
                    if (!response.ok) {
                        throw new Error("request error");
                    }
                    return response.text();
                })
                .then((code) => {
                    updateCodeView(fileName, code);
                })
                .catch((error) => console.error("fetch error: ", error));
        });
    }
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

main();
