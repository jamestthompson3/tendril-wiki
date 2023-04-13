/* Unit Tests */
import { textToHtml, htmlToText } from "./mods/parsing.js";
import { LinkedList } from "./mods/utils.js";

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
    "http://127.0.0.1:6683/files/image-1660379904659.png",
    "# some topic",
    "testing #again",
    "# a title\nsome text",
    "> testing a quote-that ends",
    "https://www.youtube.com/embed/cf72gMBrsI0",
  ];
  const html = [
    '<a href="/A%20Schoolman%E2%80%99s%20Guide%20to%20Marshall%20McLuhan">A Schoolman’s Guide to Marshall McLuhan</a>',
    'some <a href="/linked-page">linked-page</a> <a href="https://github.com">https://github.com</a>, only http.',
    '<a href="/multi%20worded%20titles">multi worded titles</a>',
    '<a href="mailto:somebody@example.com">somebody@example.com</a> wrote <a href="/article-link">this article</a> about cool things',
    '<a href="/actual-link">alias</a>',
    '<img src="http://127.0.0.1:6683/files/image-1660379904659.png">',
    "<h2>some topic</h2>",
    "testing #again",
    "<h2>a title</h2><br>some text",
    "<blockquote>testing a quote-that ends</blockquote>",
    '<iframe title="Video player" frameborder="0" allow="autoplay;" allowfullscreen src="https://www.youtube.com/embed/cf72gMBrsI0"></iframe>',
  ];
  wikitext.forEach((str, idx) => {
    const parsed = textToHtml(str);
    if (parsed !== html[idx]) {
      displayErrorMessage("wikitext", idx, parsed, html[idx]);
    }
  });

  html.forEach((str, idx) => {
    const parsedContainer = document.createElement("div");
    parsedContainer.innerHTML = str;
    const parsed = htmlToText(parsedContainer);
    if (parsed !== wikitext[idx]) {
      displayErrorMessage("htmlToText", parsed, wikitext[idx]);
    }
  });
}

testParsing();
testList();

function testList() {
  // Utils
  // ===============
  const list = new LinkedList();
  list
    .append({ id: "123", content: "hello" })
    .insertAfter({ content: "goodbye", id: "456" }, "123");

  if (list.head.value.content !== "hello") {
    displayErrorMessage("LinkedList", list.head.value.content, "hello");
  }
  if (list.tail.value.content !== "goodbye") {
    displayErrorMessage("LinkedList", list.tail.value.content, "goodbye");
  }
  list.append({ id: "abc", content: "def" });

  if (list.tail.value.id !== "abc") {
    displayErrorMessage("LinkedList - append", list.tail.value.id, "abc");
  }
  list.delete("456");
  if (list.tail.value.id !== "abc") {
    displayErrorMessage("LinkedList - delete", list.tail.value.id, "abc");
  }
  const content = list.toContentString();
  if (content !== "hello\ndef") {
    displayErrorMessage("LinkedList - toContentString", content, "hello\ndef");
  }
}

// Testing Utils
// ===================
function displayErrorMessage(test, found, expected) {
  const content = document.querySelector(".content");
  const errMsg = document.createElement("p");
  errMsg.innerHTML = `<pre>${test}</pre><strong style="color: red;">Test Failed.</strong><hr><br>Found:<br>  <pre>${found}</pre><br> Expected:<br>  <pre >${expected}</pre>`;
  content.appendChild(errMsg);
}
