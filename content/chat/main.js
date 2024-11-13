async function fetchMessages() {
  try {
      const response = await fetch('http://localhost:8080/api/chat');
      const data = await response.json();
      displayMessages(data.messages);
  } catch (error) {
      console.error('Error fetching messages:', error);
  }
}

function displayMessages(messages) {
  const chatWindow = document.getElementById('chat-window');
  chatWindow.innerHTML = '';
  messages.forEach(msg => {
      const messageDiv = document.createElement('div');
      messageDiv.classList.add('message');
      const timestamp = new Date(msg.timestamp).toLocaleTimeString();
      messageDiv.textContent = `[${timestamp}] ${msg.user}: ${msg.content}`;
      chatWindow.appendChild(messageDiv);
  });
  chatWindow.scrollTop = chatWindow.scrollHeight;
}

async function sendMessage() {
  const username = document.getElementById('username').value.trim();
  const content = document.getElementById('message-input').value.trim();

  if (!username || !content) {
      alert('Please enter both username and message content.');
      return;
  }

  const message = {
      user: username,
      content: content
  };

  try {
      await fetch('http://localhost:8080/api/chat', {
          method: 'POST',
          headers: {
              'Content-Type': 'application/json'
          },
          body: JSON.stringify(message)
      });
      document.getElementById('message-input').value = '';
      fetchMessages();
  } catch (error) {
      console.error('Error sending message:', error);
  }
}

document.getElementById('send-button').addEventListener('click', sendMessage);

// Initial fetch of messages
fetchMessages();
setInterval(fetchMessages, 3000); // Refresh messages every 3 seconds
