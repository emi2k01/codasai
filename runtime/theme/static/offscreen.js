export default function register() {
    window.addEventListener("hashchange", () => {
        closeAll();
    });

    let dimmer = document.getElementById("dimmer");
    dimmer.addEventListener("click", closeAll);

    let offscreens = document.getElementsByClassName("offscreen");

    for (let i = 0; i < offscreens.length; i++) {
        const offscreen = offscreens[i];
        registerOne(offscreen, offscreens);
    }
}

function registerOne(offscreen, offscreens) {
    let id = offscreen.getAttribute("id");
    let triggersDataAttr = `button[data-offscreen-id="${id}"]`;
    let buttons = document.querySelectorAll(triggersDataAttr);
    let dimmer = document.getElementById("dimmer");

    for (let i = 0; i < buttons.length; i++) {
        const button = buttons[i];
        button.addEventListener("click", () => {
            // close all other offscreens
            for (let i = 0; i < offscreens.length; i++) {
                if (offscreen != offscreens[i]) {
                    offscreens[i].classList.remove("open");
                }
            }

            // toggle this offscreen
            offscreen.classList.toggle("open");

            // open or close dimmer depending on the current state of this
            // offscreen
            if (!offscreen.classList.contains("open")) {
                dimmer.classList.remove("open");
            } else {
                dimmer.classList.add("open");
            }
        });
    }
}

function closeAll() {
    document.querySelectorAll(".offscreen").forEach((el) => {
        el.classList.remove("open");
    });
    dimmer.classList.remove("open");
}
