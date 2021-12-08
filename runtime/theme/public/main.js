function main() {
    let fileButtons = document.getElementsByClassName("file-button");
    let codeBlock = document.getElementById("code-block");

    for (let i = 0; i < fileButtons.length; i++) {
        const fileButton = fileButtons[i];
        fileButton.addEventListener("click", function() {
            let url = fileButton.getAttribute("data-url");

            //TODO: Add error alert
            fetch(url)
                .then(response => {
                    if (!response.ok) {
                        throw new Error('request error');
                    }
                    return response.text();
                })
                .then(code => codeBlock.innerHTML = code)
                .catch(error => console.error('fetch error: ', error));
        });
    }
}

main();