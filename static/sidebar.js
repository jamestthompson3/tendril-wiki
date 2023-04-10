import { triggerModal } from "./mods/dom.js";

const sidebar = document.querySelector("aside");

const recentlyEdited = sidebar.querySelector("#recently-edited");
recentlyEdited.onclick = async () => {
  const response = await fetch("/api/mru");
  const data = await response.json();
  const list = document.createElement("ul");
  list.style.listStyle = "none";
  for (const item of data) {
    const listItem = document.createElement("li");
    const link = document.createElement("a");
    link.href = `/${item}`;
    link.textContent = item;
    listItem.appendChild(link);
    list.appendChild(listItem);
  }
  triggerModal(list);
};

let autohide = setAutohide();
sidebar.addEventListener("mouseenter", () => {
  sidebar.style.opacity = "1";
  clearTimeout(autohide);
});

sidebar.addEventListener("mouseleave", () => {
  autohide = setAutohide(300);
});

function setAutohide(time = 2000) {
  return setTimeout(() => {
    sidebar.style.opacity = "0";
  }, time);
}
