document.addEventListener('DOMContentLoaded', () => {
  const loginForm = document.getElementById('login-form');

  loginForm.addEventListener('submit', async (e) => {
      e.preventDefault();
      const user = document.getElementById('user').value.trim();
      const password = document.getElementById('password').value.trim();
      const actionType = document.querySelector('input[name="action-type"]:checked').value;

      try {
          const response = await fetch(`${window.location.origin}/api/chat/auth`, {
              method: 'POST',
              headers: {
                  'Content-Type': 'application/json'
              },
              body: JSON.stringify({
                  type: actionType,
                  user,
                  password
              })
          });

          const data = await response.json();
          if (response.ok) {
              console.log(actionType);
              if (actionType === 'login') {
                  document.cookie = `token=${data.token}; path=/`;
                  localStorage.setItem('user', user);
                  window.location.href = 'chat.html';
              } else {
                  alert('New user created successfully! Please log in.');
              }
          } else {
              alert(`Error: ${data.error}`);
          }
      } catch (error) {
          console.error('Error during authentication:', error);
      }
  });
});