document.addEventListener('DOMContentLoaded', () => {
  const token = document.cookie.split('; ').find(row => row.startsWith('token='))?.split('=')[1];

  if (!token) {
      alert('Session expired or not logged in. Redirecting to login page.');
      window.location.href = 'index.html';
      return;
  }

  async function fetchMessages() {
      try {
          const response = await fetch(`${window.location.origin}/api/chat/messages`);
          const data = await response.json();
          displayMessages(data.messages);
      } catch (error) {
          console.error('Error fetching messages:', error);
      }
  }

  function displayMessages(messages) {
      const chatWindow = document.getElementById('chat-window');
      chatWindow.innerHTML = '';
      if (messages && messages.length > 0) {
          messages.forEach(msg => {
              const messageDiv = document.createElement('div');
              messageDiv.classList.add('message');
              const timestamp = new Date(msg.timestamp).toLocaleTimeString();
              messageDiv.textContent = `[${timestamp}] ${msg.user}: ${msg.content}`;
              chatWindow.appendChild(messageDiv);
          });
          chatWindow.scrollTop = chatWindow.scrollHeight;
      } else {
          chatWindow.textContent = 'No messages to display.';
      }
  }

  async function sendMessage() {
      const content = document.getElementById('message-input').value.trim();

      if (!content) {
          alert('Please enter a message.');
          return;
      }

      try {
          const data = await fetch(`${window.location.origin}/api/chat/send`, {
              method: 'POST',
              headers: {
                  'Content-Type': 'application/json',
                  'Authorization': `Bearer ${token}`
              },
              body: JSON.stringify({
                  user: localStorage.getItem('user'),
                  content: content,
                  token: token
              })
          });
          document.getElementById('message-input').value = '';
          fetchMessages();
      } catch (error) {
          console.error('Error sending message:', error);
      }
  }

  document.getElementById('send-button').addEventListener('click', sendMessage);
  fetchMessages();
  setInterval(fetchMessages, 3000);
});