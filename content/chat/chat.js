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

async function loadMessages() {
  try {
    const response = await fetch('/api/chat/messages');
    if (response.ok) {
      const data = await response.json();
      const messagesContainer = document.getElementById('messages');
      messagesContainer.innerHTML = '';
      data.messages.forEach(msg => {
        const messageDiv = document.createElement('div');
        messageDiv.classList.add('message');
        const timestamp = new Date(msg.timestamp).toLocaleTimeString();
        messageDiv.innerHTML = `<div class="timestamp">${timestamp}</div><div>${msg.user}: ${msg.content}</div>`;
        messagesContainer.appendChild(messageDiv);
      });
    } else {
      alert('Failed to load messages');
    }
  } catch (error) {
    console.error('Error:', error);
    alert('Error loading messages');
  }
}

setInterval(loadMessages, 3000)

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