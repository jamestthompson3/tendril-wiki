import { textToHtml, htmlToText } from "./parsing.js";
import { updateMRU, getFullBody, getTags } from "./dom.js";

let currentFocusedElement;

export function deleteBlock(el) {
  el.remove();
}

export function savePage() {
  const body = getFullBody();
  const title = document.getElementsByClassName("title")[0].innerText;
  const tags = getTags();
  // TODO parse out and save metadata, tags
  fetch("/edit", {
    method: "POST",
    body: JSON.stringify({
      body,
      title,
      old_title: CURRENT_TITLE,
      tags,
    }),
    headers: {
      "content-type": "application/json",
    },
  })
    .then((res) => {
      if (res.status < 400) {
        updateMRU(title);
      }
    })
    .catch(console.error);
}

export function setAsFocused(el) {
  if (currentFocusedElement) {
    currentFocusedElement.blur();
  }
  currentFocusedElement = el;
  el.focus();
  el.selectionStart = el.textContent?.length;
  updateInputHeight(el);
}

export function updateInputHeight(el) {
  // Let's handle text overflows
  const scrollHeight = el.scrollHeight;
  const clientHeight = el.clientHeight;
  const scrollTop = el.scrollTop;
  const heightDiff = scrollHeight - scrollTop - clientHeight;
  if (heightDiff > 0 || scrollHeight > clientHeight) {
    el.style.height = `${scrollHeight}px`;
  }
}
