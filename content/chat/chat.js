window.onload = async function() {
  const token = localStorage.getItem('token');
  const username = localStorage.getItem('username');

  if (token && username) {
    const isValid = await checkToken(username, token);
    if (!isValid) {
      window.location.href = 'login.html';
    } else {
      loadMessages();
    }
  } else {
    window.location.href = 'login.html';
  }

  document.getElementById('logout-button').addEventListener('click', () => {
    localStorage.removeItem('token');
    localStorage.removeItem('username');
    window.location.href = 'login.html';
  });

  document.getElementById('send-button').addEventListener('click', sendMessage);
};

async function checkToken(username, token) {
  try {
    const response = await fetch('/api/chat/auth', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ type: "check", user: username, token })
    });

    return response.ok;
  } catch (error) {
    console.error('Error verifying token:', error);
    return false;
  }
}

function escapeHTML(str) {
  return str.replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/"/g, "&quot;")
            .replace(/'/g, "&#039;");
}


async function loadMessages() {
  try {
    const response = await fetch('/api/chat/messages', {
      method: 'GET',
      headers: {'Auth': `${localStorage.getItem("username")}:${localStorage.getItem("token")}`}
    });
    if (response.ok) {
      const data = await response.json();
      const messagesContainer = document.getElementById('messages');
      messagesContainer.innerHTML = '';
      if (data.messages != undefined) {
        data.messages.forEach(msg => {
          const messageDiv = document.createElement('div');
          messageDiv.classList.add('message');
          const timestamp = new Date(msg.timestamp).toLocaleString();
          const safeContent = escapeHTML(msg.content).replace(/\n/g, '<br>'); // Replace \n with <br>
          messageDiv.innerHTML = `<div class="timestamp">${timestamp}</div><div>${msg.user}: ${safeContent}</div>`;
          messagesContainer.appendChild(messageDiv);
        });
      } else {
        const noMessages = document.createElement('div');
        noMessages.classList.add('no-message');
        noMessages.innerHTML = `No messages...`;
        messagesContainer.appendChild(noMessages);
      }
    } else {
      console.log(response.body)
      alert('Failed to load messages');
    }
  } catch (error) {
    console.error('Error:', error);
    alert('Error loading messages');
  }
}


setInterval(loadMessages, 3000);

async function sendMessage() {
  const input = document.getElementById('chat-input');
  const content = input.value;
  const username = localStorage.getItem('username');
  const token = localStorage.getItem('token');

  if (content.trim() === '') return;

  try {
    const response = await fetch('/api/chat/send', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ user: username, content, token })
    });
    if (response.ok) {
      input.value = '';
      loadMessages();
    } else {
      const error = await response.json();
      alert(`Error: ${error.error}`);
    }
  } catch (error) {
    console.error('Error:', error);
    alert('Failed to send message');
  }
}

window.onkeydown = function(e) {
  if (e.key == "Enter") {
    sendMessage()
  }
}
