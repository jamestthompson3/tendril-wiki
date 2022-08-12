/* Unit Tests */
import { textToHtml } from "./utils/parsing.js";

// Parsing
// ==================

function testParsing() {
  // WikiText -> HTML
  const wikitext = [
    "[[A Schoolman’s Guide to Marshall McLuhan]]",
    "some [[linked-page]] https://github.com, only http.",
    "[[multi worded titles]]",
    "somebody@example.com wrote [[this article|article-link]] about cool things",
    "[[alias|actual-link]]",
  ];
  const html = [
    '<a href="/A%20Schoolman%E2%80%99s%20Guide%20to%20Marshall%20McLuhan">A Schoolman’s Guide to Marshall McLuhan</a>',
    'some <a href="/linked-page">linked-page</a> <a href="https://github.com">https://github.com</a>, only http.',
    '<a href="/multi%20worded%20titles">multi worded titles</a>',
    '<a href="mailto:somebody@example.com">somebody@example.com</a> wrote <a href="/article-link">this article</a> about cool things',
    '<a href="/actual-link">alias</a>',
  ];
  wikitext.forEach((str, idx) => {
    const parsed = textToHtml(str);
    if (parsed !== html[idx]) {
      const content = document.querySelector(".content");
      const errMsg = document.createElement("p");
      errMsg.innerHTML = `<strong style="color: red;">Test Failed.</strong><hr><br>Found:<br>  <pre id="parsed${idx}"></pre><br> Expected:<br>  <pre id="expected${idx}"></pre>`;
      content.appendChild(errMsg);
      const parseBlock = document.getElementById(`parsed${idx}`);
      parseBlock.innerText = parsed;
      const expectedBlock = document.getElementById(`expected${idx}`);
      expectedBlock.innerText = html[idx];
    }
  });
}

testParsing();
