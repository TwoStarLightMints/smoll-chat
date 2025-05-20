const chat_window = document.querySelector('#chat-window');
const inputArea = document.querySelector('#user-message');

const socket = new WebSocket("ws://{{}}/socket");

document.querySelector('#input-area button').addEventListener('click', e => {
    e.preventDefault();

    const message = {
	name: document.cookie.split("=")[1],
	message: inputArea.textContent,
    }

    socket.send(JSON.stringify(message));

    let new_p = document.createElement('p');

    let new_node = document.createTextNode(`You: ${inputArea.textContent}`);

    new_p.setAttribute("class", "message-bubble");

    new_p.appendChild(new_node);
    chat_window.appendChild(new_p);

    new_p.scrollIntoView();

    inputArea.textContent = "";
});

socket.onmessage = e => {
    let new_p = document.createElement('p');

    let message_data = JSON.parse(e.data);

    let new_node = document.createTextNode(`${message_data.name}: ${message_data.message}`);

    new_p.setAttribute("class", "message-bubble");

    new_p.appendChild(new_node);
    chat_window.appendChild(new_p);

    new_p.scrollIntoView();

    console.log(e.data);
}
