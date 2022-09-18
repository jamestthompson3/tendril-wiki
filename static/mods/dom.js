export function updateMRU(title) {
  const mru = document.getElementById("mru");
  const links = mru.querySelectorAll("a");
  let found = false;
  for (let i = 0; i < links.length; i++) {
    const link = links[i];
    const href = link.href;
    if (href.includes(encodeURIComponent(CURRENT_TITLE))) {
      found = true;
      if (CURRENT_TITLE !== title) {
        link.href = link.href.replace(CURRENT_TITLE, title);
        link.innerText = link.innerText.replace(CURRENT_TITLE, title);
      }
      // use parent node here since we're targeting the list items that contain the link, not the link itself.
      mru.insertBefore(link.parentNode, links[0].parentNode);
      break;
    }
  }
  if (!found) {
    const newEntry = document.createElement("li");
    newEntry.innerHTML = `<a href="${title}">${title}</a>`;
    mru.insertBefore(newEntry, mru.firstChild);
    // Trim off the last link. List should only be 8 entries long.
    mru.removeChild(links[7].parentNode);
  }
  if (CURRENT_TITLE !== title) {
    history.pushState({ name: "edit page title" }, "", title);
    document.title = title;
    CURRENT_TITLE = title;
  }
}

export function moveCaretToEnd(el) {
  if (typeof el.selectionStart == "number") {
    el.selectionStart = el.selectionEnd = el.value.length;
  } else if (typeof el.createTextRange != "undefined") {
    el.focus();
    var range = el.createTextRange();
    range.collapse(false);
    range.select();
  }
}

export function moveCaretToStart(el) {
  if (typeof el.selectionStart == "number") {
    el.selectionStart = el.selectionEnd = 0;
  } else if (typeof el.createTextRange != "undefined") {
    el.focus();
    var range = el.createTextRange();
    range.collapse(false);
    range.select();
  }
}
