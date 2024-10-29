const chat_window = document.querySelector('#chat-window');

async function get_new_message() {
    try {
        const response = await fetch(`${window.location.origin}/new-message`);
        const message_json = response.json()
            .then((value) => {
                let new_p = document.createElement('p');

                let new_node;

                if (document.cookie.includes(value.sender)) {
                    new_node = document.createTextNode(`You: ${value.message}`);
                } else {
                    new_node = document.createTextNode(`${value.sender}: ${value.message}`);
                }

                new_p.setAttribute("class", "message-bubble");

                new_p.appendChild(new_node);
                chat_window.appendChild(new_p);

                new_p.scrollIntoView();
            });

        new Promise(resolve => setTimeout(resolve, 50));

        get_new_message();
    } catch (error) {
        console.error("Error encountered: ", error);
    }
}

get_new_message();


const inputArea = document.querySelector('#user-message');

document.querySelector('#input-area button').addEventListener('click', e => {
    e.preventDefault();

    fetch(`${window.location.origin}/message`, {
        method: "post",
        body: inputArea.textContent,
    });

    inputArea.textContent = "";
});