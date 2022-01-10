export default function highlight(from, to) {
    let regexFrom = new RegExp(from, "m");
    let regexTo = new RegExp(to, "m");

    let codeEl = document.querySelector(".file-viewer code");
    let codeCopyEl = document.querySelector(".file-viewer code.copy");
    let contents = codeEl.innerText;

    let fromResult = regexFrom.exec(contents);
    if (fromResult == null) {
        return;
    }

    // We start looking after `offset` but then we convert the regex match
    // index to absolute by adding `offset + 1`
    let offset = fromResult.index + fromResult[0].length;
    let contentsAfterFromIndex = contents.substring(offset);
    let toResult = regexTo.exec(contentsAfterFromIndex);
    if (toResult == null) {
        return;
    }
    let toIndex = toResult.index + offset + 1;

    let contentsHtmlHighlighted = escapeHtml(contents.substring(0, fromResult.index));
    contentsHtmlHighlighted += "<span class='highlight'>";
    contentsHtmlHighlighted += escapeHtml(contents.substring(fromResult.index, toIndex + toResult[0].length));
    contentsHtmlHighlighted += "</span>";
    contentsHtmlHighlighted += escapeHtml(contents.substring(toIndex + toResult[0].length));

    codeCopyEl.innerHTML = contentsHtmlHighlighted;

    let highlightEl = codeCopyEl.querySelector(".highlight");
    highlightEl.scrollIntoView({
        behavior: "smooth",
        inline: "nearest",
        block: "center",
    });
}

function escapeHtml(unsafe) {
    return unsafe
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#039;");
}
