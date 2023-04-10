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
    if (links.length >= 8) {
      // Trim off the last link. List should only be 8 entries long.
      mru.removeChild(links[7].parentNode);
    }
  }
  if (CURRENT_TITLE !== title || window.location.pathname === "/new") {
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

export function triggerModal(content) {
  const modal = document.getElementById("modal");
  const bodyContent = document.querySelector(".flex-container");
  const modalBody = modal.querySelector(".modal-body");
  modalBody.appendChild(content);
  modal.style.display = "flex";
  modal.style.zIndex = 2;
  modal.addEventListener("click", teardownModal);
  bodyContent.style.filter = "blur(4px)";
  document.addEventListener("keydown", tearDownOnEscape);
}

function teardownModal(e) {
  const modal = document.getElementById("modal");
  const modalBody = modal.querySelector(".modal-body");
  const content = document.querySelector(".flex-container");
  if (e.target === modalBody) return;
  content.style.filter = "";
  modalBody.innerHTML = "";
  modal.style.display = "none";
  modal.removeEventListener("click", teardownModal);
}

function tearDownOnEscape(e) {
  if (e.key === "Escape") {
    teardownModal(e);
    document.removeEventListener("keydown", tearDownOnEscape);
  }
}
