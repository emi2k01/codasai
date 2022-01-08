export default function register() {
    registerOpeners();
}

function registerOpeners() {
    let openers = document.querySelectorAll(".explorer-entries button");
    for (let i = 0; i < openers.length; i++) {
        const opener  = openers[i];

        opener.addEventListener("click", () => {
            let container = opener.parentNode;
            container.classList.toggle("open");
        })
    }
}
