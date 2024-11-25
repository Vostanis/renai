let message: string = 'Hello, World!';
let para: string = 'This is a test.';

let heading = document.createElement('h1');
heading.textContent = message;

let paragraph = document.createElement('p');
paragraph.textContent = para;

document.body.appendChild(heading);
document.body.appendChild(paragraph);
