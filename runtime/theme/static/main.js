import registerOffscreens from "./offscreen.js";
import registerExplorers from "./explorer.js";
import StateObserver from "./state.js";

function main() {
    registerOffscreens();
    registerExplorers();
    let stateObserver = new StateObserver();
    stateObserver.onAction("open_file", ["file"], ([file]) => {
        alert(file);
    });
    stateObserver.onError((e) => window.alert(`State error:\n${e}`));
    stateObserver.trigger();
}

main();
