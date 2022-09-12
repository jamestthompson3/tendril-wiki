import { htmlToText } from "./parsing.js";

export function updateMRU(title) {
  const mru = document.getElementById("mru");
  const links = mru.querySelectorAll("a");
  let found = false;
  for (let i = 0; i < links.length; i++) {
    const link = links[i];
    if (link.href.includes(CURRENT_TITLE)) {
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

export function getFullBody() {
  let fullBody;
  const pageContent = document.getElementById("content-block");
  for (let i = 0; i < pageContent.children.length; i++) {
    const child = pageContent.children[i];
    const text = htmlToText(child);
    if (fullBody) {
      fullBody = `${fullBody}\n${text}`;
    } else {
      fullBody = text;
    }
  }
  return fullBody;
}

export function getTags() {
  const tags = document.querySelector(".tags");
  return Array.from(tags.querySelectorAll("a"))
    .map((el) => el.innerText.replace("#", ""))
    .join(",");
}
