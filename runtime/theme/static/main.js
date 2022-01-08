import registerOffscreens from "./offscreen.js";
import registerExplorers from "./explorer.js";

function main() {
    let offscreens = document.getElementsByClassName("offscreen");
    registerOffscreens(offscreens);
    registerExplorers();
}

main();
