document.addEventListener('DOMContentLoaded', () => {
    document.getElementById('urlForm').addEventListener('submit', handle_button_click);
});

function handle_button_click(event) {
    event.preventDefault();
    const url = document.getElementById('urlInput').value;
    fetch('/shorten', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: `url=${encodeURIComponent(url)}`
    })
    .then(response => response.json())
    .then(data => {
        document.getElementById('result').textContent = data.msg; // Ensure the key 'msg' matches the response
    })
    .catch(err => {
        console.error('Error:', err);
        document.getElementById('result').textContent = 'Error occurred';
    });
}