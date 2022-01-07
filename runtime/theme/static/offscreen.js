let dimmer = document.getElementById("dimmer");
dimmer.addEventListener("click", closeAll);

export default function register(elements) {
    for (let i = 0; i < elements.length; i++) {
        const element = elements[i];
        registerOne(element, elements);
    }
}

function registerOne(element, elements) {
    let id = element.getAttribute("id");
    let triggersDataAttr = `button[data-offscreen-id="${id}"]`;
    let buttons = document.querySelectorAll(triggersDataAttr);
    let dimmer = document.getElementById("dimmer");

    for (let i = 0; i < buttons.length; i++) {
        const button = buttons[i];
        button.addEventListener("click", () => {
            // close all other offscreens
            for (let i = 0; i < elements.length; i++) {
                if (element != elements[i]) {
                    elements[i].classList.remove("open");
                }
            }

            // toggle this offscreen
            element.classList.toggle("open");

            // open or close dimmer depending on the current state of this
            // offscreen
            if (!element.classList.contains("open")) {
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
