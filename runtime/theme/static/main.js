import registerOffscreens from "./offscreen.js";
import registerExplorers from "./explorer.js";
import StateObserver from "./state.js";
import openFile from "./actions/open_file.js";
import highlight from "./actions/highlight.js";

function main() {
    registerOffscreens();
    registerExplorers();

    let stateObserver = new StateObserver();
    stateObserver.onAction("open_file", ["file"], ([file]) => {
        openFile(file);
    });
    stateObserver.onAction("highlight", ["file", "from", "to"], ([file, from, to]) => {
        openFile(file, () => {
            highlight(from, to);
        });
    });
    stateObserver.onError((e) => alert(`State error:\n${e}`));
    stateObserver.trigger();
}

main();
