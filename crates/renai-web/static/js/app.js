var message = 'Hello, World!';
var para = 'This is a test.';
var heading = document.createElement('h1');
heading.textContent = message;
var paragraph = document.createElement('p');
paragraph.textContent = para;
document.body.appendChild(heading);
document.body.appendChild(paragraph);
