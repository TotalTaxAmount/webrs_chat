document.getElementById('login-button').addEventListener('click', async () => {
  const username = document.getElementById('username').value;
  const password = document.getElementById('password').value;
  
  if (!username || !password) {
    document.getElementById('error-message').textContent = 'Username and password are required';
    return;
  }

  try {
    const response = await fetch('/api/chat/auth', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ type: 'login', user: username, password })
    });

    if (response.ok) {
      const data = await response.json();
      localStorage.setItem('token', data.token);
      localStorage.setItem('username', username);
      window.location.href = '/chat/';
    } else {
      const error = await response.json();
      document.getElementById('error-message').textContent = error.error;
    }
  } catch (error) {
    console.error('Error:', error);
    document.getElementById('error-message').textContent = 'Login failed';
  }
});

// Signup button logic

document.getElementById('signup-button').addEventListener('click', async () => {
  const username = document.getElementById('username').value;
  const password = document.getElementById('password').value;

  if (!username || !password) {
    document.getElementById('error-message').textContent = 'Username and password are required';
    return;
  }

  try {
    const response = await fetch('/api/chat/auth', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ type: 'new', user: username, password })
    });

    if (response.ok) {
      const data = await response.json();
      localStorage.setItem('token', data.token);
      localStorage.setItem('username', username);
      window.location.href = '/chat/';
    } else {
      const error = await response.json();
      document.getElementById('error-message').textContent = error.error;
    }
  } catch (error) {
    console.error('Error:', error);
    document.getElementById('error-message').textContent = 'Signup failed';
  }
});
