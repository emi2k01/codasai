export default function highlight(from, to) {
    let regexFrom = new RegExp(from, "m");
    let regexTo = new RegExp(to, "m");

    let codeCopyEl = document.querySelector(".file-viewer code.copy");
    let contents = codeCopyEl.innerText;

    let idxFrom = contents.search(regexFrom);
    if (idxFrom == -1) {
        return;
    }

    // We start looking after `idxFrom` but then we convert the regex match
    // index to absolute by adding `idxFrom + 1`
    let contentsAfterIdxFrom = contents.substring(idxFrom + 1);
    let relativeIdxTo = contentsAfterIdxFrom.search(regexTo);
    let idxTo = relativeIdxTo + idxFrom + 1;

    let contentsHtmlHighlighted = escapeHtml(contents.substring(0, idxFrom));
    contentsHtmlHighlighted += "<span class='highlight'>";
    contentsHtmlHighlighted += escapeHtml(contents.substring(idxFrom, idxTo + 1));
    contentsHtmlHighlighted += "</span>";
    contentsHtmlHighlighted += escapeHtml(contents.substring(idxTo + 1));

    codeCopyEl.innerHTML = contentsHtmlHighlighted;
}

function escapeHtml(unsafe) {
    return unsafe
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#039;");
}
