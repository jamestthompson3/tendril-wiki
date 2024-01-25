const sidebar = document.querySelector("nav");
const additionalActionsDialog = document.getElementById("additional-actions");
const showAA = document.querySelector("#additional-actions + button");
const closeAA = document.querySelector("#additional-actions button");
const recentlyEditedDialog = document.getElementById("recently-edited");
const closeRecents = document.querySelector("#recently-edited button");

const openRecents = document.getElementById("get-recents");
openRecents.onclick = async () => {
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
    listItem.style.margin = "0.5rem 0";
    list.appendChild(listItem);
  }
  recentlyEditedDialog.append(list);
  recentlyEditedDialog.showModal();
};
closeRecents.addEventListener("click", () => {
  recentlyEditedDialog.close();
  recentlyEditedDialog.querySelector("ul").remove();
});

showAA.addEventListener("click", () => {
  additionalActionsDialog.showModal();
});

closeAA.addEventListener("click", () => {
  additionalActionsDialog.close();
});
